#[cfg(feature = "debug")]
pub(crate) mod debug;
#[cfg(feature = "display")]
pub(crate) mod display;
mod parsing;

use std::{iter, mem};

use proc_macro2::{Ident, Span, TokenStream, TokenTree};
use quote::ToTokens;
use syn::{
    buffer::Cursor,
    parse::{Parse, ParseBuffer, ParseStream},
    punctuated::Punctuated,
    spanned::Spanned as _,
    Error, Result,
};

/// Additional trait bounds attribute.
#[derive(Debug, Default)]
struct BoundsAttribute(Punctuated<syn::WherePredicate, syn::token::Comma>);

impl BoundsAttribute {
    /// Errors in case legacy syntax is encountered: `bound = "..."`.
    fn check_legacy_fmt(input: &ParseBuffer<'_>, error_span: Span) -> Result<()> {
        let fork = input.fork();

        let path = fork
            .parse::<syn::Path>()
            .and_then(|path| fork.parse::<syn::token::Eq>().map(|_| path));

        match path {
            Ok(path) if path.is_ident("bound") => fork
                .parse::<syn::Lit>()
                .ok()
                .and_then(|lit| match lit {
                    syn::Lit::Str(s) => Some(s.value()),
                    _ => None,
                })
                .map_or(Ok(()), |bound| {
                    Err(Error::new(
                        error_span,
                        format!("Legacy syntax, use: `bound({bound})`"),
                    ))
                }),
            Ok(_) | Err(_) => Ok(()),
        }
    }
}

impl Parse for BoundsAttribute {
    fn parse(input: ParseStream) -> Result<Self> {
        let _ = input.parse::<syn::Path>().and_then(|p| {
            if ["bound", "bounds"].into_iter().any(|i| p.is_ident(i)) {
                Ok(p)
            } else {
                Err(Error::new(
                    p.span(),
                    "Unknown attribute. Expected `bound(...)`",
                ))
            }
        })?;

        let content;
        syn::parenthesized!(content in input);

        content
            .parse_terminated(syn::WherePredicate::parse)
            .map(Self)
    }
}

/// [`fmt`] attribute.
///
/// [`fmt`]: std::fmt
#[derive(Debug)]
struct FmtAttribute {
    /// Interpolation [`syn::LitStr`].
    lit: syn::LitStr,

    /// Interpolation arguments.
    args: Vec<FmtArgument>,
}

impl FmtAttribute {
    /// Returns [`Iterator`] over bounded [`syn::Type`]s and trait names.
    fn bounded_types<'a>(
        &'a self,
        fields: &'a syn::Fields,
    ) -> impl Iterator<Item = (&'a syn::Type, &'static str)> {
        let placeholders = Placeholder::parse_fmt_string(&self.lit.value());

        // We ignore unknown fields, as compiler will produce better error messages.
        placeholders.into_iter().filter_map(move |placeholder| {
            let name = match placeholder.arg {
                Parameter::Named(name) => self
                    .args
                    .iter()
                    .find_map(|a| (a.alias.as_ref()? == &name).then_some(&a.expr))
                    .map_or(Some(name), |expr| expr.ident().map(ToString::to_string))?,
                Parameter::Positional(i) => self
                    .args
                    .get(i)
                    .and_then(|a| a.expr.ident().filter(|_| a.alias.is_none()))?
                    .to_string(),
            };

            let unnamed = name.strip_prefix('_').and_then(|s| s.parse().ok());
            let ty = match (&fields, unnamed) {
                (syn::Fields::Unnamed(f), Some(i)) => {
                    f.unnamed.iter().nth(i).map(|f| &f.ty)
                }
                (syn::Fields::Named(f), None) => f.named.iter().find_map(|f| {
                    f.ident.as_ref().filter(|s| **s == name).map(|_| &f.ty)
                }),
                _ => None,
            }?;

            Some((ty, placeholder.trait_name))
        })
    }

    /// Errors in case legacy syntax is encountered: `fmt = "...", (arg),*`.
    fn check_legacy_fmt(input: &ParseBuffer<'_>, error_span: Span) -> Result<()> {
        let fork = input.fork();

        let path = fork
            .parse::<syn::Path>()
            .and_then(|path| fork.parse::<syn::token::Eq>().map(|_| path));

        match path {
            Ok(path) if path.is_ident("fmt") => (|| {
                let args = fork
                    .parse_terminated::<_, syn::token::Comma>(syn::Lit::parse)
                    .ok()?
                    .into_iter()
                    .enumerate()
                    .filter_map(|(i, lit)| match lit {
                        syn::Lit::Str(str) => Some(if i == 0 {
                            format!("\"{}\"", str.value())
                        } else {
                            str.value()
                        }),
                        _ => None,
                    })
                    .collect::<Vec<_>>();
                (!args.is_empty()).then_some(args)
            })()
            .map_or(Ok(()), |fmt| {
                Err(Error::new(
                    error_span,
                    format!("Legacy syntax, use: `{}`", fmt.join(", ")),
                ))
            }),
            Ok(_) | Err(_) => Ok(()),
        }
    }
}

impl Parse for FmtAttribute {
    fn parse(input: ParseStream) -> Result<Self> {
        let fmt_lit = input.parse()?;

        if input.peek(syn::token::Comma) {
            let _ = input.parse::<syn::token::Comma>()?;
        }

        let fmt_args = input.step(|cursor| {
            let mut cursor = *cursor;
            let arguments = iter::from_fn(|| {
                let (arg, c) = FmtArgument::parse(cursor)?;
                cursor = c;
                Some(arg)
            })
            .collect();

            Ok((arguments, cursor))
        })?;

        Ok(Self {
            lit: fmt_lit,
            args: fmt_args,
        })
    }
}

impl ToTokens for FmtAttribute {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.lit.to_tokens(tokens);
        for arg in &self.args {
            syn::token::Comma(arg.span()).to_tokens(tokens);
            arg.to_tokens(tokens);
        }
    }
}

/// [Named parameter][1]: `identifier '=' expression`.
///
/// [1]: https://doc.rust-lang.org/stable/std/fmt/index.html#named-parameters
#[derive(Debug, Default)]
struct FmtArgument {
    /// `identifier` [`Ident`].
    alias: Option<Ident>,

    /// `expression` [`FmtExpr`].
    expr: FmtExpr,
}

impl FmtArgument {
    /// Parses this [`FmtArgument`] and returns a new [`Cursor`], where parsing
    /// can be continued.
    ///
    /// Returns [`None`] in case of eof or trailing comma.
    fn parse(mut cursor: Cursor<'_>) -> Option<(FmtArgument, Cursor<'_>)> {
        let mut arg = FmtArgument::default();

        if let Some((ident, c)) = cursor.ident() {
            if let Some((_eq, c)) = c.punct().filter(|(p, _)| p.as_char() == '=') {
                arg.alias = Some(ident);
                cursor = c;
            }
        }

        if let Some((ident, c)) = cursor.ident() {
            if let Some(c) = c
                .punct()
                .and_then(|(p, c)| (p.as_char() == ',').then_some(c))
                .or_else(|| c.eof().then_some(c))
            {
                arg.expr = FmtExpr::Ident(ident);
                return Some((arg, c));
            }
        }

        let (rest, c) = Self::parse_rest(cursor)?;
        arg.expr.extend(rest);

        Some((arg, c))
    }

    /// Parses the rest, until the end of this [`FmtArgument`] (comma or eof),
    /// in case simplest case of `(ident =)? ident(,|eof)` wasn't parsed.
    fn parse_rest(c: Cursor<'_>) -> Option<(TokenStream, Cursor<'_>)> {
        use crate::parsing::{
            alt, balanced_pair, colon2, punct, qself, seq, take_until1, token_tree,
        };

        take_until1(
            alt([
                &mut seq([&mut colon2, &mut qself]),
                &mut seq([&mut qself, &mut colon2]),
                &mut balanced_pair(punct('|'), punct('|')),
                &mut token_tree,
            ]),
            punct(','),
        )(c)
    }
}

impl ToTokens for FmtArgument {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        if let Some(alias) = &self.alias {
            alias.to_tokens(tokens);
            syn::token::Eq::default().to_tokens(tokens);
        }
        self.expr.to_tokens(tokens);
    }
}

/// Expression of a [`FmtArgument`].
///
/// This type is used instead of a [`syn::Expr`] to avoid using [`syn`]'s
/// `full` feature increasing compilation times.
#[derive(Debug)]
enum FmtExpr {
    /// [`Ident`].
    Ident(Ident),

    /// Plain [`TokenStream`].
    TokenStream(TokenStream),
}

impl FmtExpr {
    /// Returns an [`Ident`] in case this [`FmtExpr`] contains only it, or [`None`]
    /// otherwise.
    fn ident(&self) -> Option<&Ident> {
        match self {
            Self::Ident(i) => Some(i),
            Self::TokenStream(_) => None,
        }
    }
}

impl Default for FmtExpr {
    fn default() -> Self {
        Self::TokenStream(TokenStream::new())
    }
}

impl ToTokens for FmtExpr {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::Ident(ident) => ident.to_tokens(tokens),
            Self::TokenStream(ts) => ts.to_tokens(tokens),
        }
    }
}

impl Extend<TokenTree> for FmtExpr {
    fn extend<T: IntoIterator<Item = TokenTree>>(&mut self, iter: T) {
        *self = iter
            .into_iter()
            .fold(mem::take(self), |this, tt| match (this, tt) {
                (Self::TokenStream(stream), TokenTree::Ident(ident))
                    if stream.is_empty() =>
                {
                    Self::Ident(ident)
                }
                (Self::TokenStream(mut stream), tt) => {
                    stream.extend([tt]);
                    Self::TokenStream(stream)
                }
                (Self::Ident(ident), tt) => {
                    let mut stream = ident.into_token_stream();
                    stream.extend([tt]);
                    Self::TokenStream(stream)
                }
            });
    }
}

/// [Parameter][1] used in [`Placeholder`].
///
/// [1]: https://doc.rust-lang.org/stable/std/fmt/index.html#formatting-parameters
#[derive(Debug, Eq, PartialEq)]
enum Parameter {
    /// [Positional parameter][1].
    ///
    /// [1]: https://doc.rust-lang.org/stable/std/fmt/index.html#positional-parameters
    Positional(usize),

    /// [Named parameter][1].
    ///
    /// [1]: https://doc.rust-lang.org/stable/std/fmt/index.html#named-parameters
    Named(String),
}

impl<'a> From<parsing::Argument<'a>> for Parameter {
    fn from(arg: parsing::Argument<'a>) -> Self {
        match arg {
            parsing::Argument::Integer(i) => Parameter::Positional(i),
            parsing::Argument::Identifier(i) => Parameter::Named(i.to_owned()),
        }
    }
}

/// Representation of formatting placeholder.
#[derive(Debug, PartialEq, Eq)]
struct Placeholder {
    /// Formatting argument (either named or positional) to be used by this placeholder.
    arg: Parameter,

    /// [Width parameter][1], if present.
    ///
    /// [1]: https://doc.rust-lang.org/stable/std/fmt/index.html#width
    width: Option<Parameter>,

    /// [Precision parameter][1], if present.
    ///
    /// [1]: https://doc.rust-lang.org/stable/std/fmt/index.html#precision
    precision: Option<Parameter>,

    /// Name of [`std::fmt`] trait to be used for rendering this placeholder.
    trait_name: &'static str,
}

impl Placeholder {
    /// Parses [`Placeholder`]s from a given formatting string.
    fn parse_fmt_string(s: &str) -> Vec<Self> {
        let mut n = 0;
        parsing::format_string(s)
            .into_iter()
            .flat_map(|f| f.formats)
            .map(|format| {
                let (maybe_arg, ty) = (
                    format.arg,
                    format.spec.map(|s| s.ty).unwrap_or(parsing::Type::Display),
                );
                let position = maybe_arg.map(Into::into).unwrap_or_else(|| {
                    // Assign "the next argument".
                    // https://doc.rust-lang.org/stable/std/fmt/index.html#positional-parameters
                    n += 1;
                    Parameter::Positional(n - 1)
                });

                Self {
                    arg: position,
                    width: format.spec.and_then(|s| match s.width {
                        Some(parsing::Count::Parameter(arg)) => Some(arg.into()),
                        _ => None,
                    }),
                    precision: format.spec.and_then(|s| match s.precision {
                        Some(parsing::Precision::Count(parsing::Count::Parameter(
                            arg,
                        ))) => Some(arg.into()),
                        _ => None,
                    }),
                    trait_name: ty.trait_name(),
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod fmt_attribute_spec {
    use itertools::Itertools as _;
    use quote::ToTokens;
    use syn;

    use super::FmtAttribute;

    fn assert<'a>(input: &'a str, parsed: impl AsRef<[&'a str]>) {
        let parsed = parsed.as_ref();
        let attr = syn::parse_str::<FmtAttribute>(&format!("\"\", {}", input)).unwrap();
        let fmt_args = attr
            .args
            .into_iter()
            .map(|arg| arg.into_token_stream().to_string())
            .collect::<Vec<String>>();
        fmt_args.iter().zip_eq(parsed).enumerate().for_each(
            |(i, (found, expected))| {
                assert_eq!(
                    *expected, found,
                    "Mismatch at index {i}\n\
                             Expected: {parsed:?}\n\
                             Found: {fmt_args:?}",
                );
            },
        );
    }

    #[test]
    fn cases() {
        let cases = [
            "ident",
            "alias = ident",
            "[a , b , c , d]",
            "counter += 1",
            "async { fut . await }",
            "a < b",
            "a > b",
            "{ let x = (a , b) ; }",
            "invoke (a , b)",
            "foo as f64",
            "| a , b | a + b",
            "obj . k",
            "for pat in expr { break pat ; }",
            "if expr { true } else { false }",
            "vector [2]",
            "1",
            "\"foo\"",
            "loop { break i ; }",
            "format ! (\"{}\" , q)",
            "match n { Some (n) => { } , None => { } }",
            "x . foo ::< T > (a , b)",
            "x . foo ::< T < [T < T >; if a < b { 1 } else { 2 }] >, { a < b } > (a , b)",
            "(a + b)",
            "i32 :: MAX",
            "1 .. 2",
            "& a",
            "[0u8 ; N]",
            "(a , b , c , d)",
            "< Ty as Trait > :: T",
            "< Ty < Ty < T >, { a < b } > as Trait < T > > :: T",
        ];

        assert("", []);
        for i in 1..4 {
            for permutations in cases.into_iter().permutations(i) {
                let mut input = permutations.clone().join(",");
                assert(&input, &permutations);
                input.push(',');
                assert(&input, &permutations);
            }
        }
    }
}

#[cfg(test)]
mod placeholder_parse_fmt_string_spec {
    use super::*;

    #[test]
    fn indicates_position_and_trait_name_for_each_fmt_placeholder() {
        let fmt_string = "{},{:?},{{}},{{{1:0$}}}-{2:.1$x}{par:#?}{:width$}";
        assert_eq!(
            Placeholder::parse_fmt_string(&fmt_string),
            vec![
                Placeholder {
                    arg: Parameter::Positional(0),
                    width: None,
                    precision: None,
                    trait_name: "Display",
                },
                Placeholder {
                    arg: Parameter::Positional(1),
                    width: None,
                    precision: None,
                    trait_name: "Debug",
                },
                Placeholder {
                    arg: Parameter::Positional(1),
                    width: Some(Parameter::Positional(0)),
                    precision: None,
                    trait_name: "Display",
                },
                Placeholder {
                    arg: Parameter::Positional(2),
                    width: None,
                    precision: Some(Parameter::Positional(1)),
                    trait_name: "LowerHex",
                },
                Placeholder {
                    arg: Parameter::Named("par".to_owned()),
                    width: None,
                    precision: None,
                    trait_name: "Debug",
                },
                Placeholder {
                    arg: Parameter::Positional(2),
                    width: Some(Parameter::Named("width".to_owned())),
                    precision: None,
                    trait_name: "Display",
                },
            ],
        );
    }
}
