use std::mem;

use proc_macro2::{Ident, TokenStream, TokenTree};
use quote::{format_ident, quote, ToTokens};
use syn::{
    parse::{Parse, ParseStream, Parser},
    parse_quote,
    punctuated::Punctuated,
    spanned::Spanned as _,
    Error, Result,
};

use crate::parsing;

fn trait_name_to_attribute_name(trait_name: &str) -> &'static str {
    match trait_name {
        "Display" => "display",
        "Binary" => "binary",
        "Octal" => "octal",
        "LowerHex" => "lower_hex",
        "UpperHex" => "upper_hex",
        "LowerExp" => "lower_exp",
        "UpperExp" => "upper_exp",
        "Pointer" => "pointer",
        "Debug" => "debug",
        _ => unimplemented!(),
    }
}

/// Provides the hook to expand `#[derive(Display)]` into an implementation of `From`
pub fn expand(input: &syn::DeriveInput, trait_name: &str) -> Result<TokenStream> {
    let trait_name = if trait_name == "DebugCustom" {
        "Debug"
    } else {
        trait_name
    };

    let attrs = Attributes::parse_attrs(&input.attrs, trait_name)?;
    let trait_ident = format_ident!("{trait_name}");
    let ident = &input.ident;

    let ctx = (&attrs, ident, &trait_ident, trait_name);
    let (bounds, fmt) = match &input.data {
        syn::Data::Struct(s) => expand_struct(s, ctx),
        syn::Data::Enum(e) => expand_enum(e, ctx),
        syn::Data::Union(u) => expand_union(u, ctx),
    }?;

    let (impl_gens, ty_gens, where_clause) = {
        let (impl_gens, ty_gens, where_clause) = input.generics.split_for_impl();
        let mut where_clause =
            where_clause.cloned().unwrap_or_else(|| parse_quote!(where));
        where_clause.predicates.extend(bounds);
        (impl_gens, ty_gens, where_clause)
    };

    let res = quote! {
        #[automatically_derived]
        impl #impl_gens ::core::fmt::#trait_ident for #ident #ty_gens
             #where_clause
        {
            fn fmt(
                &self, __derive_more_f: &mut ::core::fmt::Formatter<'_>
            ) -> ::core::fmt::Result {
                #fmt
            }
        }
    };

    Ok(res)
}

type ExpansionCtx<'a> = (&'a Attributes, &'a Ident, &'a Ident, &'a str);

fn expand_struct(
    s: &syn::DataStruct,
    (attrs, ident, trait_ident, _): ExpansionCtx<'_>,
) -> Result<(Vec<syn::WherePredicate>, TokenStream)> {
    let s = StructOrEnumVariant {
        attrs,
        fields: &s.fields,
        trait_ident,
        ident,
    };
    let bounds = s.generate_bounds();
    let fmt = s.generate_fmt();

    let vars = s.fields.iter().enumerate().map(|(i, f)| {
        let var = f.ident.clone().unwrap_or_else(|| format_ident!("_{i}"));
        let member = f
            .ident
            .clone()
            .map_or_else(|| syn::Member::Unnamed(i.into()), syn::Member::Named);
        quote! { let #var = &self.#member; }
    });

    let fmt = quote! {
        #( #vars )*
        #fmt
    };

    Ok((bounds, fmt))
}

// TODO: top-level attribute on enum.
fn expand_enum(
    e: &syn::DataEnum,
    (attrs, ident, trait_ident, trait_name): ExpansionCtx<'_>,
) -> Result<(Vec<syn::WherePredicate>, TokenStream)> {
    let (bounds, fmt) = e.variants.iter().try_fold(
        (Vec::new(), TokenStream::new()),
        |(mut bounds, mut fmt), variant| {
            let attrs = Attributes::parse_attrs(&variant.attrs, trait_name)?;
            let ident = &variant.ident;

            let v = StructOrEnumVariant {
                attrs: &attrs,
                fields: &variant.fields,
                trait_ident,
                ident,
            };
            let fmt_inner = v.generate_fmt();
            bounds.extend(v.generate_bounds());

            let fields_idents =
                variant.fields.iter().enumerate().map(|(i, f)| {
                    f.ident.clone().unwrap_or_else(|| format_ident!("_{i}"))
                });
            let matcher = match variant.fields {
                syn::Fields::Named(_) => {
                    quote! { Self::#ident { #( #fields_idents ),* } }
                }
                syn::Fields::Unnamed(_) => {
                    quote! { Self::#ident ( #( #fields_idents ),* ) }
                }
                syn::Fields::Unit => quote! { Self::#ident },
            };

            fmt.extend([quote! { #matcher => { #fmt_inner }, }]);

            Ok::<_, Error>((bounds, fmt))
        },
    )?;

    let fmt = fmt
        .is_empty()
        .then(|| quote! { match *self {} })
        .unwrap_or_else(|| quote! { match self { #fmt } });

    Ok((bounds, fmt))
}

fn expand_union(
    u: &syn::DataUnion,
    (attrs, _, _, trait_name): ExpansionCtx<'_>,
) -> Result<(Vec<syn::WherePredicate>, TokenStream)> {
    let fmt_lit = attrs.display_literal.as_ref().ok_or_else(|| {
        Error::new(
            u.fields.span(),
            format!(
                "Unions must have `#[{}(\"...\", ...)]` attribute",
                trait_name_to_attribute_name(trait_name),
            ),
        )
    })?;
    let fmt_args = &attrs.display_args;

    let fmt = quote! { ::core::write!(__derive_more_f, #fmt_lit, #( #fmt_args ),*) };

    Ok((attrs.bounds.clone().into_iter().collect(), fmt))
}

#[derive(Debug)]
struct Attributes {
    display_literal: Option<syn::LitStr>,
    display_args: Vec<FmtArgument>,
    bounds: Punctuated<syn::WherePredicate, syn::token::Comma>,
}

impl Attributes {
    fn parse_attrs(
        attrs: impl AsRef<[syn::Attribute]>,
        trait_name: &str,
    ) -> Result<Self> {
        let (display_literal, display_args, bounds) = attrs
            .as_ref()
            .iter()
            .filter(|attr| attr.path.is_ident(trait_name_to_attribute_name(trait_name)))
            .try_fold(
                (None, Vec::new(), Punctuated::new()),
                |(lit, args, mut bounds), attr| {
                    let attribute =
                        Parser::parse2(Attribute::parse, attr.tokens.clone())?;
                    Ok(match attribute {
                        Attribute::Bounds(more) => {
                            bounds.extend(more);
                            (lit, args, bounds)
                        }
                        Attribute::Display {
                            display_literal,
                            display_arguments,
                        } => (
                            // TODO: use `Span::join`, once resolved:
                            //       https://github.com/rust-lang/rust/issues/54725
                            if lit.is_some() {
                                return Err(Error::new(
                                    display_literal.span(),
                                    format!(
                                        "Duplicates for `#[{}(\"...\", ...)]` attribute aren't allowed",
                                        trait_name_to_attribute_name(trait_name),
                                    ),
                                ));
                            } else {
                                Some(display_literal)
                            },
                            args.into_iter().chain(display_arguments).collect(),
                            bounds,
                        ),
                    })
                },
            )?;

        Ok(Self {
            display_literal,
            display_args,
            bounds,
        })
    }
}

#[derive(Debug)]
enum Attribute {
    Display {
        display_literal: syn::LitStr,
        display_arguments: Vec<FmtArgument>,
    },
    Bounds(Punctuated<syn::WherePredicate, syn::token::Comma>),
}

#[derive(Debug)]
struct FmtArgument {
    alias: Option<Ident>,
    expr: IdentOrTokenStream,
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

#[derive(Debug)]
enum IdentOrTokenStream {
    Ident(Ident),
    TokenStream(TokenStream),
}

impl IdentOrTokenStream {
    fn ident(&self) -> Option<&Ident> {
        match self {
            IdentOrTokenStream::Ident(i) => Some(i),
            IdentOrTokenStream::TokenStream(_) => None,
        }
    }
}

impl Default for IdentOrTokenStream {
    fn default() -> Self {
        Self::TokenStream(TokenStream::new())
    }
}

impl ToTokens for IdentOrTokenStream {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::Ident(ident) => ident.to_tokens(tokens),
            Self::TokenStream(ts) => ts.to_tokens(tokens),
        }
    }
}

impl IdentOrTokenStream {
    fn new() -> Self {
        Self::default()
    }

    fn extend_ts(&mut self, stream: TokenStream) -> &mut Self {
        let this = mem::take(self);
        *self = Self::TokenStream(match this {
            Self::Ident(ident) => {
                let mut ident = ident.into_token_stream();
                ident.extend([stream]);
                ident
            }
            Self::TokenStream(mut old) => {
                old.extend([stream]);
                old
            }
        });
        self
    }

    fn extend_tt(&mut self, tt: TokenTree) -> &mut Self {
        self.extend_ts(tt.into())
    }

    fn push_ident(mut self, ident: Ident) -> Self {
        self.extend_tt(ident.into());
        self
    }
}

impl Parse for Attribute {
    fn parse(input: ParseStream) -> Result<Self> {
        use syn::buffer::Cursor;

        const PAIRED_PUNCTS: [(char, char); 2] = [('<', '>'), ('|', '|')];

        fn paired_punct(p: char) -> Option<char> {
            PAIRED_PUNCTS
                .iter()
                .find_map(|(l, r)| (*l == p).then_some(*r))
        }

        fn parse_until_paired_punct(
            punct: char,
            mut cursor: Cursor<'_>,
        ) -> Option<(TokenStream, Cursor<'_>)> {
            let mut stream = TokenStream::new();

            while let Some((tt, c)) = cursor.token_tree() {
                match tt {
                    TokenTree::Punct(p) if p.as_char() == punct => {
                        stream.extend([TokenTree::Punct(p)]);
                        return Some((stream, c));
                    }
                    TokenTree::Punct(p) if paired_punct(p.as_char()).is_some() => {
                        let (more, c) = paired_punct(p.as_char())
                            .and_then(|p| parse_until_paired_punct(p, c))?;
                        stream.extend([TokenTree::Punct(p)]);
                        stream.extend([more]);
                        cursor = c;
                    }
                    tt => stream.extend([tt]),
                }
            }

            None
        }

        let content;
        syn::parenthesized!(content in input);

        if content.peek(syn::LitStr) {
            let display_literal = content.parse()?;

            if content.peek(syn::token::Comma) {
                let _ = content.parse::<syn::token::Comma>()?;
            }

            let display_arguments = content.step(|cursor| {
                let mut arguments = Vec::new();

                let mut rest = *cursor;
                while !rest.eof() {
                    let mut expr = None;
                    let mut alias = None;

                    if let Some((ident, c)) = rest.ident() {
                        if let Some((_eq, c)) =
                            c.punct().filter(|(p, _)| p.as_char() == '=')
                        {
                            alias = Some(ident);
                            rest = c;
                        }
                    }

                    if let Some((gr @ TokenTree::Group(_), c)) = rest.token_tree() {
                        expr.get_or_insert_with(IdentOrTokenStream::new)
                            .extend_tt(gr);
                        rest = c;
                    }

                    if let Some((ident, c)) = rest.ident() {
                        if c.eof()
                            || c.punct().filter(|(p, _)| p.as_char() == ',').is_some()
                        {
                            expr = Some(match expr.take() {
                                None => IdentOrTokenStream::Ident(ident),
                                Some(s) => s.push_ident(ident),
                            });
                            rest = c;
                        }
                    }

                    while let Some((tt, c)) = rest.token_tree() {
                        rest = c;

                        match tt {
                            TokenTree::Punct(p) if p.as_char() == ',' => {
                                break;
                            }
                            TokenTree::Punct(p)
                                if paired_punct(p.as_char()).is_some() =>
                            {
                                let (more, c) = paired_punct(p.as_char())
                                    .and_then(|p| parse_until_paired_punct(p, c))
                                    .ok_or_else::<Error, _>(|| {
                                        Error::new(
                                            p.span(),
                                            format!(
                                                "Failed to find closing '{}' for '{}'",
                                                paired_punct(p.as_char())
                                                    .unwrap_or_else(|| unreachable!()),
                                                p.as_char(),
                                            ),
                                        )
                                    })?;
                                rest = c;
                                expr.get_or_insert_with(IdentOrTokenStream::new)
                                    .extend_ts(more);
                            }
                            tt => {
                                expr.get_or_insert_with(IdentOrTokenStream::new)
                                    .extend_tt(tt);
                            }
                        }
                    }

                    if let Some(expr) = expr {
                        arguments.push(FmtArgument { alias, expr });
                    }
                }

                Ok((arguments, rest))
            })?;

            return Ok(Attribute::Display {
                display_literal,
                display_arguments,
            });
        }

        let _ = content.parse::<syn::Path>().and_then(|p| {
            if ["bound", "bounds"].into_iter().any(|i| p.is_ident(i)) {
                Ok(p)
            } else {
                Err(Error::new(
                    p.span(),
                    format!(
                        "Unknown attribute. Expected `\"...\", ...` or `bound(...)`",
                    ),
                ))
            }
        })?;

        let inner;
        syn::parenthesized!(inner in content);

        inner
            .parse_terminated(syn::WherePredicate::parse)
            .map(Attribute::Bounds)
    }
}

#[derive(Debug)]
struct StructOrEnumVariant<'a> {
    attrs: &'a Attributes,
    fields: &'a syn::Fields,
    trait_ident: &'a Ident,
    ident: &'a Ident,
}

impl<'a> StructOrEnumVariant<'a> {
    fn generate_fmt(&self) -> TokenStream {
        if let Some(lit) = &self.attrs.display_literal {
            let args = &self.attrs.display_args;
            quote! { ::core::write!(__derive_more_f, #lit, #( #args ),*) }
        } else if self.fields.iter().count() == 1 {
            let field = self
                .fields
                .iter()
                .next()
                .unwrap_or_else(|| unreachable!("count() == 1"));
            let ident = field.ident.clone().unwrap_or_else(|| format_ident!("_0"));
            let trait_ident = self.trait_ident;

            quote! { ::core::fmt::#trait_ident::fmt(#ident, __derive_more_f) }
        } else {
            let ident_str = self.ident.to_string();
            quote! { ::core::write!(__derive_more_f, #ident_str) }
        }
    }

    fn generate_bounds(&self) -> Vec<syn::WherePredicate> {
        let Some(display_literal) = &self.attrs.display_literal else {
            return self.fields.iter().next().map(|f| {
                let ty = &f.ty;
                vec![parse_quote! { #ty: ::core::fmt::Display }]
            })
            .unwrap_or_default();
        };

        let placeholders = Placeholder::parse_fmt_string(&display_literal.value());

        // We ignore unknown fields, as compiler will produce better error
        // messages.
        placeholders
            .into_iter()
            .filter_map(|placeholder| {
                let name = match placeholder.arg {
                    Parameter::Named(name) => self
                        .attrs
                        .display_args
                        .iter()
                        .find_map(|a| (a.alias.as_ref()? == &name).then_some(&a.expr))
                        .map_or(Some(name), |expr| {
                            expr.ident().map(ToString::to_string)
                        })?,
                    Parameter::Positional(i) => self
                        .attrs
                        .display_args
                        .iter()
                        .nth(i)
                        .and_then(|a| a.expr.ident().filter(|_| a.alias.is_none()))?
                        .to_string(),
                };

                let unnamed = name.strip_prefix("_").and_then(|s| s.parse().ok());
                let ty = match (&self.fields, unnamed) {
                    (syn::Fields::Unnamed(f), Some(i)) => {
                        f.unnamed.iter().nth(i).map(|f| &f.ty)
                    }
                    (syn::Fields::Named(f), None) => f.named.iter().find_map(|f| {
                        f.ident.as_ref().filter(|s| **s == name).map(|_| &f.ty)
                    }),
                    _ => None,
                }?;

                let tr = format_ident!("{}", placeholder.trait_name);
                Some(parse_quote! { #ty: ::core::fmt::#tr })
            })
            .chain(self.attrs.bounds.clone())
            .collect()
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
