//! Description for [`fmt`]-like derive macros.
//!
//! [`fmt`]: std::fmt

use std::mem;

use proc_macro2::{Ident, TokenStream, TokenTree};
use quote::{format_ident, quote, ToTokens};
use syn::buffer::Cursor;
use syn::{
    parse::{Parse, ParseStream},
    parse_quote,
    punctuated::Punctuated,
    spanned::Spanned as _,
    Error, Result,
};

use crate::parsing;

/// Expands [`fmt`]-like derive macro.
///
/// Available macros:
/// - [`Binary`]
/// - [`Debug`]
/// - [`Display`]
/// - [`LowerExp`]
/// - [`LowerHex`]
/// - [`Octal`]
/// - [`Pointer`]
/// - [`UpperExp`]
/// - [`UpperHex`]
///
/// [`fmt`]: std::fmt
/// [`Binary`]: std::fmt::Binary
/// [`Debug`]: std::fmt::Debug
/// [`Display`]: std::fmt::Display
/// [`LowerExp`]: std::fmt::LowerExp
/// [`LowerHex`]: std::fmt::LowerHex
/// [`Octal`]: std::fmt::Octal
/// [`Pointer`]: std::fmt::Pointer
/// [`UpperExp`]: std::fmt::UpperExp
/// [`UpperHex`]: std::fmt::UpperHex
pub fn expand(input: &syn::DeriveInput, trait_name: &str) -> Result<TokenStream> {
    let trait_name = normalize_trait_name(trait_name);

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

/// Type alias for expansion context:
/// - [`Attributes`]
/// - Struct/enum/union [`Ident`]
/// - Derived trait [`Ident`]
/// - Derived trait `&`[`str`]
type ExpansionCtx<'a> = (&'a Attributes, &'a Ident, &'a Ident, &'a str);

/// Expands [`fmt`]-like derive macro for struct.
///
/// [`fmt`]: std::fmt
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
/// Expands [`fmt`]-like derive macro for enum.
///
/// [`fmt`]: std::fmt
fn expand_enum(
    e: &syn::DataEnum,
    (attrs, _, trait_ident, trait_name): ExpansionCtx<'_>,
) -> Result<(Vec<syn::WherePredicate>, TokenStream)> {
    if attrs.fmt_lit.is_some() {
        todo!("https://github.com/JelteF/derive_more/issues/142");
    }

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

/// Expands [`fmt`]-like derive macro for union.
///
/// [`fmt`]: std::fmt
fn expand_union(
    u: &syn::DataUnion,
    (attrs, _, _, trait_name): ExpansionCtx<'_>,
) -> Result<(Vec<syn::WherePredicate>, TokenStream)> {
    let fmt_lit = attrs.fmt_lit.as_ref().ok_or_else(|| {
        Error::new(
            u.fields.span(),
            format!(
                "Unions must have `#[{}(\"...\", ...)]` attribute",
                trait_name_to_attribute_name(trait_name),
            ),
        )
    })?;
    let fmt_args = &attrs.fmt_args;

    let fmt = quote! { ::core::write!(__derive_more_f, #fmt_lit, #( #fmt_args ),*) };

    Ok((attrs.bounds.clone().into_iter().collect(), fmt))
}

/// [`fmt`]-like derive macro attributes:
///
/// ```rust,ignore
/// #[<fmt_trait>("<fmt_literal>", <fmt_args>)]
/// #[bound(<bounds>)]
/// ```
///
/// `#[<fmt_trait>(...)]` can be specified only once, while multiple
/// `#[bound(...)]` are allowed.
///
/// [`fmt`]: std::fmt
#[derive(Debug, Default)]
struct Attributes {
    /// Interpolation [`syn::LitStr`].
    fmt_lit: Option<syn::LitStr>,

    /// Interpolation arguments.
    fmt_args: Vec<FmtArgument>,

    /// Addition trait bounds.
    bounds: Punctuated<syn::WherePredicate, syn::token::Comma>,
}

impl Attributes {
    /// Parses [`Attributes`] from `&[`[`syn::Attribute`]`]`.
    fn parse_attrs(
        attrs: impl AsRef<[syn::Attribute]>,
        trait_name: &str,
    ) -> Result<Self> {
        attrs
            .as_ref()
            .iter()
            .filter(|attr| attr.path.is_ident(trait_name_to_attribute_name(trait_name)))
            .try_fold(Attributes::default(), |mut attrs, attr| {
                let attr = syn::parse2::<Attribute>(attr.tokens.clone())?;
               match attr {
                   Attribute::Bounds(more) => {
                       attrs.bounds.extend(more);
                   }
                   Attribute::Fmt { fmt_lit, fmt_args } => {
                       attrs.fmt_lit.replace(fmt_lit).map_or(Ok(()), |dup| Err(Error::new(
                           dup.span(),
                           format!(
                               "Multiple `#[{}(\"...\", ...)]` attributes aren't allowed",
                               trait_name_to_attribute_name(trait_name),
                           ))))?;
                       attrs.fmt_args.extend(fmt_args)
                   }
               };
                Ok(attrs)
            })
    }
}

/// Helper struct to [`StructOrEnumVariant::generate_fmt()`] or
/// [`StructOrEnumVariant::generate_bounds()`].
#[derive(Debug)]
struct StructOrEnumVariant<'a> {
    /// Derive macro [`Attributes`].
    attrs: &'a Attributes,

    /// Struct or enum [`Ident`].
    ident: &'a Ident,

    /// Struct or enum [`syn::Fields`].
    fields: &'a syn::Fields,

    /// [`fmt`] trait [`Ident`].
    ///
    /// [`fmt`]: std::fmt
    trait_ident: &'a Ident,
}

impl<'a> StructOrEnumVariant<'a> {
    /// Generates [`Display::fmt()`] impl for struct or enum variant.
    ///
    /// [`Display::fmt()`]: std::fmt::Display::fmt()
    fn generate_fmt(&self) -> TokenStream {
        if let Some(lit) = &self.attrs.fmt_lit {
            let args = &self.attrs.fmt_args;
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

    /// Generates trait bounds.
    fn generate_bounds(&self) -> Vec<syn::WherePredicate> {
        let Some(display_literal) = &self.attrs.fmt_lit else {
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
                        .fmt_args
                        .iter()
                        .find_map(|a| (a.alias.as_ref()? == &name).then_some(&a.expr))
                        .map_or(Some(name), |expr| {
                            expr.ident().map(ToString::to_string)
                        })?,
                    Parameter::Positional(i) => self
                        .attrs
                        .fmt_args
                        .get(i)
                        .and_then(|a| a.expr.ident().filter(|_| a.alias.is_none()))?
                        .to_string(),
                };

                let unnamed = name.strip_prefix('_').and_then(|s| s.parse().ok());
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

/// Single [`fmt`]-like display attribute.
///
/// [`fmt`]: std::fmt
#[derive(Debug)]
enum Attribute {
    /// [`fmt`] attribute.
    ///
    /// [`fmt`]: std::fmt
    Fmt {
        /// Interpolation [`syn::LitStr`].
        fmt_lit: syn::LitStr,

        /// Interpolation arguments.
        fmt_args: Vec<FmtArgument>,
    },

    /// Addition trait bounds.
    Bounds(Punctuated<syn::WherePredicate, syn::token::Comma>),
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

impl ToTokens for FmtArgument {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        if let Some(alias) = &self.alias {
            alias.to_tokens(tokens);
            syn::token::Eq::default().to_tokens(tokens);
        }
        self.expr.to_tokens(tokens);
    }
}

/// Expression of [`FmtArgument`].
///
/// This type is used instead of [`syn::Expr`] to avoid [`syn`]'s `full`
/// feature.
#[derive(Debug)]
enum FmtExpr {
    /// [`Ident`].
    Ident(Ident),

    /// Plain [`TokenStream`].
    TokenStream(TokenStream),
}

impl FmtExpr {
    /// Returns [`Ident`] in case this [`FmtExpr`] contains only it or [`None`]
    /// otherwise.
    fn ident(&self) -> Option<&Ident> {
        match self {
            Self::Ident(i) => Some(i),
            Self::TokenStream(_) => None,
        }
    }

    fn is_empty(&self) -> bool {
        match self {
            FmtExpr::Ident(_) => false,
            FmtExpr::TokenStream(stream) => stream.is_empty(),
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

impl Parse for Attribute {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        syn::parenthesized!(content in input);

        // Parses `"..."(,)? (expr),*`
        if content.peek(syn::LitStr) {
            let display_literal = content.parse()?;

            if content.peek(syn::token::Comma) {
                let _ = content.parse::<syn::token::Comma>()?;
            }

            let display_arguments = content.step(|cursor| {
                let mut cursor = *cursor;
                let mut arguments = Vec::new();

                while let Some((arg, c)) = FmtArgument::parse(cursor) {
                    arguments.push(arg);
                    cursor = c;
                }

                Ok((arguments, cursor))
            })?;

            return Ok(Attribute::Fmt {
                fmt_lit: display_literal,
                fmt_args: display_arguments,
            });
        }

        let _ = content.parse::<syn::Path>().and_then(|p| {
            if ["bound", "bounds"].into_iter().any(|i| p.is_ident(i)) {
                Ok(p)
            } else {
                Err(Error::new(
                    p.span(),
                    "Unknown attribute. Expected `\"...\", ...` or `bound(...)`",
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

impl FmtArgument {
    fn parse(mut cursor: Cursor<'_>) -> Option<(FmtArgument, Cursor<'_>)> {
        let mut arg = FmtArgument::default();

        // `ident =`
        if let Some((ident, c)) = cursor.ident() {
            if let Some((_eq, c)) = c.punct().filter(|(p, _)| p.as_char() == '=') {
                arg.alias = Some(ident);
                cursor = c;
            }
        }

        // `ident(,|eof)`
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

        let (rest, c) = Self::parse_inner(cursor);
        arg.expr.extend(rest);

        (!arg.expr.is_empty()).then_some((arg, c))
    }

    fn parse_inner(mut cursor: Cursor<'_>) -> (TokenStream, Cursor<'_>) {
        let mut out = TokenStream::new();

        loop {
            if let Some(extend) = Self::turbofish(cursor) {
                cursor = extend(&mut out);
                continue;
            }
            if let Some(extend) = Self::closure(cursor) {
                cursor = extend(&mut out);
                continue;
            }

            if let Some(c) = cursor
                .punct()
                .and_then(|(p, c)| (p.as_char() == ',').then_some(c))
                .or_else(|| cursor.eof().then_some(cursor))
            {
                return (out, c);
            }

            let (tt, c) = cursor
                .token_tree()
                .unwrap_or_else(|| unreachable!("checked for eof"));
            out.extend([tt]);
            cursor = c;
        }
    }

    fn closure<'a>(
        cursor: Cursor<'a>,
    ) -> Option<impl FnOnce(&mut TokenStream) -> Cursor<'a> + 'a> {
        let (open, c) = cursor.punct().filter(|(p, _)| p.as_char() == '|')?;

        Some(move |stream: &mut TokenStream| {
            stream.extend([TokenTree::Punct(open)]);
            let (more, c) = Self::parse_until_closing('|', '|', c);
            stream.extend(more);
            c
        })
    }

    fn turbofish<'a>(
        cursor: Cursor<'a>,
    ) -> Option<impl FnOnce(&mut TokenStream) -> Cursor<'a> + 'a> {
        use proc_macro2::Spacing;

        let (colon1, c) = cursor
            .punct()
            .filter(|(p, _)| p.as_char() == ':' && p.spacing() == Spacing::Joint)?;
        let (colon2, c) = c.punct().filter(|(p, _)| p.as_char() == ':')?;
        let (less, c) = c.punct().filter(|(p, _)| p.as_char() == '<')?;

        Some(move |stream: &mut TokenStream| {
            stream.extend([colon1, colon2, less].map(TokenTree::Punct));
            let (more, c) = Self::parse_until_closing('<', '>', c);
            stream.extend(more);
            c
        })
    }

    fn parse_until_closing<'a>(
        open: char,
        close: char,
        mut cursor: Cursor<'a>,
    ) -> (TokenStream, Cursor<'a>) {
        let mut out = TokenStream::new();
        let mut count = 1;

        while let Some((tt, c)) = cursor.token_tree().filter(|_| count != 0) {
            match tt {
                TokenTree::Punct(ref p) if p.as_char() == close => {
                    count -= 1;
                }
                TokenTree::Punct(ref p) if p.as_char() == open => {
                    count += 1;
                }
                _ => {}
            }

            out.extend([tt]);
            cursor = c;
        }

        (out, cursor)
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

fn trait_name_to_attribute_name(trait_name: &str) -> &'static str {
    match trait_name {
        "Binary" => "binary",
        "Debug" => "debug",
        "Display" => "display",
        "LowerExp" => "lower_exp",
        "LowerHex" => "lower_hex",
        "Octal" => "octal",
        "Pointer" => "pointer",
        "UpperExp" => "upper_exp",
        "UpperHex" => "upper_hex",
        _ => unimplemented!(),
    }
}

fn normalize_trait_name(name: &str) -> &'static str {
    match name {
        "Binary" => "Binary",
        "Debug" | "DebugCustom" => "Debug",
        "Display" => "Display",
        "LowerExp" => "LowerExp",
        "LowerHex" => "LowerHex",
        "Octal" => "Octal",
        "Pointer" => "Pointer",
        "UpperExp" => "UpperExp",
        "UpperHex" => "UpperHex",
        _ => unimplemented!(),
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

#[cfg(test)]
mod attribute_parse_fmt_args_spec {
    use itertools::Itertools as _;
    use quote::ToTokens;
    use syn;

    use super::Attribute;

    fn assert<'a>(input: &'a str, parsed: impl AsRef<[&'a str]>) {
        let parsed = parsed.as_ref();
        match syn::parse_str::<Attribute>(&format!("(\"\", {})", input)).unwrap() {
            Attribute::Fmt { fmt_args, .. } => {
                let fmt_args = fmt_args
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
            attrs @ Attribute::Bounds(_) => {
                panic!("Expected `Attribute::Fmt`, found: {attrs:?}");
            }
        }
    }

    #[test]
    fn cases() {
        assert("", []);

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
            "(a + b)",
            "i32 :: MAX",
            "1 .. 2",
            "& a",
            "[0u8 ; N]",
            "(a , b , c , d)",
        ];

        for i in 0..4 {
            for permutations in cases.into_iter().permutations(i) {
                let input = permutations.clone().join(",");
                assert(&input, &permutations);
            }
        }
    }
}
