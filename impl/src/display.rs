//! Implementation of [`fmt`]-like derive macros.
//!
//! [`fmt`]: std::fmt

use std::{iter, mem};

use proc_macro2::{Ident, Span, TokenStream, TokenTree};
use quote::{format_ident, quote, ToTokens};
use syn::{
    buffer::Cursor,
    parse::{Parse, ParseBuffer, ParseStream},
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
    let (bounds, body) = match &input.data {
        syn::Data::Struct(s) => expand_struct(s, ctx),
        syn::Data::Enum(e) => expand_enum(e, ctx),
        syn::Data::Union(u) => expand_union(u, ctx),
    }?;

    let (impl_gens, ty_gens, where_clause) = {
        let (impl_gens, ty_gens, where_clause) = input.generics.split_for_impl();
        let mut where_clause = where_clause
            .cloned()
            .unwrap_or_else(|| parse_quote! { where });
        where_clause.predicates.extend(bounds);
        (impl_gens, ty_gens, where_clause)
    };

    Ok(quote! {
        #[automatically_derived]
        impl #impl_gens ::core::fmt::#trait_ident for #ident #ty_gens
             #where_clause
        {
            fn fmt(
                &self, __derive_more_f: &mut ::core::fmt::Formatter<'_>
            ) -> ::core::fmt::Result {
                #body
            }
        }
    })
}

/// Type alias for an expansion context:
/// - [`Attributes`].
/// - Struct/enum/union [`Ident`].
/// - Derived trait [`Ident`].
/// - Derived trait `&`[`str`].
type ExpansionCtx<'a> = (&'a Attributes, &'a Ident, &'a Ident, &'a str);

/// Expands a [`fmt`]-like derive macro for the provided struct.
///
/// [`fmt`]: std::fmt
fn expand_struct(
    s: &syn::DataStruct,
    (attrs, ident, trait_ident, _): ExpansionCtx<'_>,
) -> Result<(Vec<syn::WherePredicate>, TokenStream)> {
    let s = Expansion {
        attrs,
        fields: &s.fields,
        trait_ident,
        ident,
    };
    let bounds = s.generate_bounds();
    let body = s.generate_body();

    let vars = s.fields.iter().enumerate().map(|(i, f)| {
        let var = f.ident.clone().unwrap_or_else(|| format_ident!("_{i}"));
        let member = f
            .ident
            .clone()
            .map_or_else(|| syn::Member::Unnamed(i.into()), syn::Member::Named);
        quote! {
            let #var = &self.#member;
        }
    });

    let body = quote! {
        #( #vars )*
        #body
    };

    Ok((bounds, body))
}

/// Expands a [`fmt`]-like derive macro for the provided enum.
///
/// [`fmt`]: std::fmt
fn expand_enum(
    e: &syn::DataEnum,
    (attrs, _, trait_ident, trait_name): ExpansionCtx<'_>,
) -> Result<(Vec<syn::WherePredicate>, TokenStream)> {
    if attrs.fmt.is_some() {
        todo!("https://github.com/JelteF/derive_more/issues/142");
    }

    let (bounds, match_arms) = e.variants.iter().try_fold(
        (Vec::new(), TokenStream::new()),
        |(mut bounds, mut arms), variant| {
            let attrs = Attributes::parse_attrs(&variant.attrs, trait_name)?;
            let ident = &variant.ident;

            if attrs.fmt.is_none()
                && variant.fields.is_empty()
                && trait_name != "Display"
            {
                return Err(Error::new(
                    e.variants.span(),
                    format!(
                        "Implicit formatting of unit enum variant is supported \
                         only for `Display` macro. Use `#[{}(\"...\")]` to \
                         explicitly specify the formatting.",
                        trait_name_to_attribute_name(trait_name),
                    ),
                ));
            }

            let v = Expansion {
                attrs: &attrs,
                fields: &variant.fields,
                trait_ident,
                ident,
            };
            let arm_body = v.generate_body();
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

            arms.extend([quote! { #matcher => { #arm_body }, }]);

            Ok::<_, Error>((bounds, arms))
        },
    )?;

    let body = match_arms
        .is_empty()
        .then(|| quote! { match *self {} })
        .unwrap_or_else(|| quote! { match self { #match_arms } });

    Ok((bounds, body))
}

/// Expands a [`fmt`]-like derive macro for the provided union.
///
/// [`fmt`]: std::fmt
fn expand_union(
    u: &syn::DataUnion,
    (attrs, _, _, trait_name): ExpansionCtx<'_>,
) -> Result<(Vec<syn::WherePredicate>, TokenStream)> {
    let fmt = &attrs.fmt.as_ref().ok_or_else(|| {
        Error::new(
            u.fields.span(),
            format!(
                "Unions must have `#[{}(\"...\", ...)]` attribute",
                trait_name_to_attribute_name(trait_name),
            ),
        )
    })?;
    let (lit, args) = (&fmt.lit, &fmt.args);

    let body = quote! {
        ::core::write!(__derive_more_f, #lit, #( #args ),*)
    };

    Ok((attrs.bounds.0.clone().into_iter().collect(), body))
}

/// Representation of a [`fmt`]-like derive macro attribute.
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
    /// Interpolation [`FmtAttribute`].
    fmt: Option<FmtAttribute>,

    /// Addition trait bounds.
    bounds: BoundsAttribute,
}

impl Attributes {
    /// Parses [`Attributes`] from the provided [`syn::Attribute`]s.
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
                       attrs.bounds.0.extend(more.0);
                   }
                   Attribute::Fmt(fmt) => {
                       attrs.fmt.replace(fmt).map_or(Ok(()), |dup| Err(Error::new(
                           dup.span(),
                           format!(
                               "Multiple `#[{}(\"...\", ...)]` attributes aren't allowed",
                               trait_name_to_attribute_name(trait_name),
                           ))))?;
                   }
               };
                Ok(attrs)
            })
    }
}

/// Helper struct to generate [`Display::fmt()`] implementation body and trait
/// bounds for a struct or an enum variant.
///
/// [`Display::fmt()`]: std::fmt::Display::fmt()
#[derive(Debug)]
struct Expansion<'a> {
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

impl<'a> Expansion<'a> {
    /// Generates [`Display::fmt()`] implementation for a struct or an enum variant.
    ///
    /// [`Display::fmt()`]: std::fmt::Display::fmt()
    fn generate_body(&self) -> TokenStream {
        if let Some(fmt) = &self.attrs.fmt {
            let (lit, args) = (&fmt.lit, &fmt.args);
            quote! {
                ::core::write!(__derive_more_f, #lit, #( #args ),*)
            }
        } else if self.fields.iter().count() == 1 {
            let field = self
                .fields
                .iter()
                .next()
                .unwrap_or_else(|| unreachable!("count() == 1"));
            let ident = field.ident.clone().unwrap_or_else(|| format_ident!("_0"));
            let trait_ident = self.trait_ident;
            quote! {
                ::core::fmt::#trait_ident::fmt(#ident, __derive_more_f)
            }
        } else {
            let ident_str = self.ident.to_string();
            quote! {
                ::core::write!(__derive_more_f, #ident_str)
            }
        }
    }

    /// Generates trait bounds for a struct or an enum variant.
    fn generate_bounds(&self) -> Vec<syn::WherePredicate> {
        let Some(fmt) = &self.attrs.fmt else {
            return self.fields.iter().next().map(|f| {
                let ty = &f.ty;
                let trait_ident = &self.trait_ident;
                vec![parse_quote! { #ty: ::core::fmt::#trait_ident }]
            })
            .unwrap_or_default();
        };

        fmt.bounded_types(self.fields)
            .map(|(ty, trait_name)| {
                let tr = format_ident!("{}", trait_name);
                parse_quote! { #ty: ::core::fmt::#tr }
            })
            .chain(self.attrs.bounds.0.clone())
            .collect()
    }
}

/// Representation of a single [`fmt`]-like display attribute.
///
/// [`fmt`]: std::fmt
#[derive(Debug)]
enum Attribute {
    /// [`fmt`] attribute.
    ///
    /// [`fmt`]: std::fmt
    Fmt(FmtAttribute),

    /// Addition trait bounds.
    Bounds(BoundsAttribute),
}

impl Parse for Attribute {
    fn parse(input: ParseStream) -> Result<Self> {
        let error_span = input.span();

        let content;
        syn::parenthesized!(content in input);

        BoundsAttribute::check_legacy_fmt(&content, error_span)?;
        FmtAttribute::check_legacy_fmt(&content, error_span)?;

        if content.peek(syn::LitStr) {
            content.parse().map(Attribute::Fmt)
        } else {
            content.parse().map(Attribute::Bounds)
        }
    }
}

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
    /// Returns an [`Iterator`] over bounded [`syn::Type`]s and trait names.
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

        let (rest, c) = Self::parse_rest(cursor);
        arg.expr.extend(rest);

        (!arg.expr.is_empty()).then_some((arg, c))
    }

    /// Parses the rest, until the end of this [`FmtArgument`] (comma or eof),
    /// in case simplest case of `(ident =)? ident(,|eof)` wasn't parsed.
    fn parse_rest(mut cursor: Cursor<'_>) -> (TokenStream, Cursor<'_>) {
        let mut out = TokenStream::new();

        loop {
            if let Some(extend) = Self::turbofish(cursor) {
                cursor = extend(&mut out);
                continue;
            }
            if let Some(extend) = Self::closure_args(cursor) {
                cursor = extend(&mut out);
                continue;
            }
            if let Some(extend) = Self::qself(cursor) {
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

    /// Tries to parse `| (closure_arg),* |`.
    fn closure_args<'a>(
        cursor: Cursor<'a>,
    ) -> Option<impl FnOnce(&mut TokenStream) -> Cursor<'a> + 'a> {
        let (open, c) = cursor.punct().filter(|(p, _)| p.as_char() == '|')?;

        Some(move |stream: &mut TokenStream| {
            stream.extend([TokenTree::Punct(open)]);
            // We can ignore inner `|`, because only other place it can appear
            // is in or pattern (ex. `Either::Left(v) | Either::Right(v)`),
            // which must be parenthesized, so will be parsed as one
            // `TokenTree`.
            let (more, c) = Self::parse_until_closing('|', '|', c);
            stream.extend(more);
            c
        })
    }

    /// Tries to parse `::< ... >`.
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

    /// Tries to parse `< ... as ... >::`.
    fn qself<'a>(
        cursor: Cursor<'a>,
    ) -> Option<impl FnOnce(&mut TokenStream) -> Cursor<'a> + 'a> {
        use proc_macro2::Spacing;

        let (less, c) = cursor.punct().filter(|(p, _)| p.as_char() == '<')?;
        let (more, c) = Self::parse_until_closing('<', '>', c);
        let (colon1, c) = c
            .punct()
            .filter(|(p, _)| p.as_char() == ':' && p.spacing() == Spacing::Joint)?;
        let (colon2, c) = c.punct().filter(|(p, _)| p.as_char() == ':')?;

        Some(move |stream: &mut TokenStream| {
            stream.extend([less].map(TokenTree::Punct));
            stream.extend(more);
            stream.extend([colon1, colon2].map(TokenTree::Punct));
            c
        })
    }

    /// Parses until balanced amount of `open` and `close` [`TokenTree::Punc`]
    /// or eof.
    ///
    /// [`Cursor`] should be pointing **right after** the first `open`ing.
    fn parse_until_closing(
        open: char,
        close: char,
        mut cursor: Cursor<'_>,
    ) -> (TokenStream, Cursor<'_>) {
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

    /// Checks whether this [`FmtExpr`] is empty.
    fn is_empty(&self) -> bool {
        match self {
            Self::Ident(_) => false,
            Self::TokenStream(stream) => stream.is_empty(),
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

/// Matches trait name to [`Attribute::Fmt`] argument name.
fn trait_name_to_attribute_name(trait_name: &str) -> &'static str {
    match trait_name {
        "Binary" => "binary",
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

/// Matches derive macro name to actual trait name.
fn normalize_trait_name(name: &str) -> &'static str {
    match name {
        "Binary" => "Binary",
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

pub mod debug {
    //! Implementation of [`fmt::Debug`] derive macro.
    //!
    //! [`fmt::Debug`]: std::fmt::Debug

    use proc_macro2::TokenStream;
    use quote::{format_ident, quote};
    use syn::{
        parse::{Error, Parse, ParseStream, Result},
        parse_quote,
        spanned::Spanned as _,
        Ident,
    };

    use super::{BoundsAttribute, FmtAttribute};

    /// Expands [`fmt::Debug`] derive macro.
    ///
    /// [`fmt::Debug`]: std::fmt::Debug
    pub fn expand(input: &syn::DeriveInput, _: &str) -> Result<TokenStream> {
        let attrs = ContainerAttributes::parse_attrs(&input.attrs)?;
        let ident = &input.ident;

        let (bounds, body) = match &input.data {
            syn::Data::Struct(s) => expand_struct(attrs, ident, s),
            syn::Data::Enum(e) => expand_enum(attrs, e),
            syn::Data::Union(_) => {
                return Err(Error::new(
                    input.span(),
                    "`Debug` cannot be derived for unions",
                ));
            }
        }?;

        let (impl_gens, ty_gens, where_clause) = {
            let (impl_gens, ty_gens, where_clause) = input.generics.split_for_impl();
            let mut where_clause = where_clause
                .cloned()
                .unwrap_or_else(|| parse_quote! { where });
            where_clause.predicates.extend(bounds);
            (impl_gens, ty_gens, where_clause)
        };

        Ok(quote! {
            #[automatically_derived]
            impl #impl_gens ::core::fmt::Debug for #ident #ty_gens
                 #where_clause
            {
                fn fmt(
                    &self, __derive_more_f: &mut ::core::fmt::Formatter<'_>
                ) -> ::core::fmt::Result {
                    #body
                }
            }
        })
    }

    /// Expands a [`fmt::Debug`] derive macro for the provided struct.
    ///
    /// [`fmt::Debug`]: std::fmt::Debug
    fn expand_struct(
        attrs: ContainerAttributes,
        ident: &Ident,
        s: &syn::DataStruct,
    ) -> Result<(Vec<syn::WherePredicate>, TokenStream)> {
        let s = Expansion {
            attr: &attrs,
            fields: &s.fields,
            ident,
        };
        let bounds = s.generate_bounds()?;
        let body = s.generate_body()?;

        let vars = s.fields.iter().enumerate().map(|(i, f)| {
            let var = f.ident.clone().unwrap_or_else(|| format_ident!("_{i}"));
            let member = f
                .ident
                .clone()
                .map_or_else(|| syn::Member::Unnamed(i.into()), syn::Member::Named);
            quote! { let #var = &self.#member; }
        });

        let body = quote! {
            #( #vars )*
            #body
        };

        Ok((bounds, body))
    }

    /// Expands a [`fmt::Debug`] derive macro for the provided enum.
    ///
    /// [`fmt::Debug`]: std::fmt::Debug
    fn expand_enum(
        attrs: ContainerAttributes,
        e: &syn::DataEnum,
    ) -> Result<(Vec<syn::WherePredicate>, TokenStream)> {
        let (bounds, match_arms) = e.variants.iter().try_fold(
            (Vec::new(), TokenStream::new()),
            |(mut bounds, mut arms), variant| {
                let ident = &variant.ident;

                let v = Expansion {
                    attr: &attrs,
                    fields: &variant.fields,
                    ident,
                };
                let arm_body = v.generate_body()?;
                bounds.extend(v.generate_bounds()?);

                let fields_idents = variant.fields.iter().enumerate().map(|(i, f)| {
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

                arms.extend([quote! { #matcher => { #arm_body }, }]);

                Ok::<_, Error>((bounds, arms))
            },
        )?;

        let body = match_arms
            .is_empty()
            .then(|| quote! { match *self {} })
            .unwrap_or_else(|| quote! { match self { #match_arms } });

        Ok((bounds, body))
    }

    /// Representation of a [`fmt::Debug`] derive macro container attribute.
    ///
    /// ```rust,ignore
    /// #[debug(bound(<bounds>))]
    /// ```
    ///
    /// [`fmt::Debug`]: std::fmt::Debug
    #[derive(Debug, Default)]
    struct ContainerAttributes {
        /// Addition trait bounds.
        bounds: BoundsAttribute,
    }

    impl ContainerAttributes {
        /// Parses [`ContainerAttributes`] from the provided
        /// [`syn::Attribute`]s.
        fn parse_attrs(attrs: impl AsRef<[syn::Attribute]>) -> Result<Self> {
            attrs
                .as_ref()
                .iter()
                .filter(|attr| attr.path.is_ident("debug"))
                .try_fold(ContainerAttributes::default(), |mut attrs, attr| {
                    let attr = syn::parse2::<ContainerAttributes>(attr.tokens.clone())?;
                    attrs.bounds.0.extend(attr.bounds.0);
                    Ok(attrs)
                })
        }
    }

    impl Parse for ContainerAttributes {
        fn parse(input: ParseStream) -> Result<Self> {
            let error_span = input.span();

            let content;
            syn::parenthesized!(content in input);

            BoundsAttribute::check_legacy_fmt(&content, error_span)?;

            content.parse().map(|bounds| ContainerAttributes { bounds })
        }
    }

    /// Representation of a [`fmt::Debug`] derive macro field attribute.
    ///
    /// ```rust,ignore
    /// #[debug("<fmt_literal>", <fmt_args>)]
    /// #[debug(skip)]
    /// ```
    ///
    /// [`fmt::Debug`]: std::fmt::Debug
    enum FieldAttribute {
        /// [`fmt`] attribute.
        ///
        /// [`fmt`]: std::fmt
        Fmt(FmtAttribute),

        /// Attribute for skipping field.
        Skip,
    }

    impl FieldAttribute {
        /// Parses [`ContainerAttributes`] from the provided
        /// [`syn::Attribute`]s.
        fn parse_attrs(attrs: impl AsRef<[syn::Attribute]>) -> Result<Option<Self>> {
            Ok(attrs
                .as_ref()
                .iter()
                .filter(|attr| attr.path.is_ident("debug"))
                .try_fold(None, |mut attrs, attr| {
                    let field_attr =
                        syn::parse2::<FieldAttribute>(attr.tokens.clone())?;
                    if let Some((path, _)) = attrs.replace((&attr.path, field_attr)) {
                        Err(Error::new(
                            path.span(),
                            "Only single `#[debug(...)]` attribute is allowed here",
                        ))
                    } else {
                        Ok(attrs)
                    }
                })?
                .map(|(_, attr)| attr))
        }
    }

    impl Parse for FieldAttribute {
        fn parse(input: ParseStream) -> Result<Self> {
            let error_span = input.span();

            let content;
            syn::parenthesized!(content in input);

            FmtAttribute::check_legacy_fmt(&content, error_span)?;

            if content.peek(syn::LitStr) {
                content.parse().map(Self::Fmt)
            } else {
                let _ = content.parse::<syn::Path>().and_then(|p| {
                    if ["skip", "ignore"].into_iter().any(|i| p.is_ident(i)) {
                        Ok(p)
                    } else {
                        Err(Error::new(
                            p.span(),
                            "Unknown attribute. Expected `skip` or `ignore`",
                        ))
                    }
                })?;

                Ok(Self::Skip)
            }
        }
    }

    /// Helper struct to generate [`Debug::fmt()`] implementation body and trait
    /// bounds for a struct or an enum variant.
    ///
    /// [`Debug::fmt()`]: std::fmt::Debug::fmt()
    #[derive(Debug)]
    struct Expansion<'a> {
        attr: &'a ContainerAttributes,

        /// Struct or enum [`Ident`].
        ident: &'a Ident,

        /// Struct or enum [`syn::Fields`].
        fields: &'a syn::Fields,
    }

    impl<'a> Expansion<'a> {
        /// Generates [`Debug::fmt()`] implementation for a struct or an enum
        /// variant.
        ///
        /// [`Debug::fmt()`]: std::fmt::Debug::fmt()
        fn generate_body(&self) -> Result<TokenStream> {
            match self.fields {
                syn::Fields::Unit => {
                    let ident = self.ident.to_string();
                    Ok(quote! {
                        ::core::fmt::Formatter::write_str(
                            __derive_more_f,
                            #ident,
                        )
                    })
                }
                syn::Fields::Unnamed(unnamed) => {
                    let mut exhaustive = true;
                    let ident_str = self.ident.to_string();

                    let out = quote! {
                        &mut ::derive_more::fmt::debug_tuple(
                            __derive_more_f,
                            #ident_str,
                        )
                    };
                    let out = unnamed.unnamed.iter().enumerate().try_fold(
                        out,
                        |out, (i, field)| match FieldAttribute::parse_attrs(
                            &field.attrs,
                        )? {
                            Some(FieldAttribute::Skip) => {
                                exhaustive = false;
                                Ok::<_, Error>(out)
                            }
                            Some(FieldAttribute::Fmt(fmt)) => Ok(quote! {
                                ::derive_more::fmt::DebugTuple::field(
                                    #out,
                                    &::core::format_args!(#fmt),
                                )
                            }),
                            None => {
                                let ident = format_ident!("_{i}");
                                Ok(quote! {
                                    ::derive_more::fmt::DebugTuple::field(#out, #ident)
                                })
                            }
                        },
                    )?;
                    Ok(if exhaustive {
                        quote! { ::derive_more::fmt::DebugTuple::finish(#out) }
                    } else {
                        quote! { ::derive_more::fmt::DebugTuple::finish_non_exhaustive(#out) }
                    })
                }
                syn::Fields::Named(named) => {
                    let mut exhaustive = true;
                    let ident = self.ident.to_string();

                    let out = quote! {
                        &mut ::core::fmt::Formatter::debug_struct(
                            __derive_more_f,
                            #ident,
                        )
                    };
                    let out = named.named.iter().try_fold(out, |out, field| {
                        let field_ident = field.ident.as_ref().unwrap_or_else(|| {
                            unreachable!("`syn::Fields::Named`");
                        });
                        let field_str = field_ident.to_string();
                        match FieldAttribute::parse_attrs(&field.attrs)? {
                            Some(FieldAttribute::Skip) => {
                                exhaustive = false;
                                Ok::<_, Error>(out)
                            }
                            Some(FieldAttribute::Fmt(fmt)) => Ok(quote! {
                                ::core::fmt::DebugStruct::field(
                                    #out,
                                    #field_str,
                                    &::core::format_args!(#fmt),
                                )
                            }),
                            None => Ok(quote! {
                                ::core::fmt::DebugStruct::field(#out, #field_str, #field_ident)
                            }),
                        }
                    })?;
                    Ok(if exhaustive {
                        quote! { ::core::fmt::DebugStruct::finish(#out) }
                    } else {
                        quote! { ::core::fmt::DebugStruct::finish_non_exhaustive(#out) }
                    })
                }
            }
        }

        /// Generates trait bounds for a struct or an enum variant.
        fn generate_bounds(&self) -> Result<Vec<syn::WherePredicate>> {
            self.fields.iter().try_fold(
                self.attr.bounds.0.clone().into_iter().collect::<Vec<_>>(),
                |mut out, field| {
                    let fmt_attr =
                        FieldAttribute::parse_attrs(&field.attrs)?.and_then(|attr| {
                            match attr {
                                FieldAttribute::Fmt(fmt) => Some(fmt),
                                FieldAttribute::Skip => None,
                            }
                        });
                    let ty = &field.ty;

                    if let Some(attr) = fmt_attr {
                        out.extend(attr.bounded_types(self.fields).map(
                            |(ty, trait_name)| {
                                let trait_name = format_ident!("{trait_name}");
                                parse_quote! { #ty: ::core::fmt::#trait_name }
                            },
                        ));
                    } else {
                        out.extend([parse_quote! { #ty: ::core::fmt::Debug }]);
                    }

                    Ok(out)
                },
            )
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

#[cfg(test)]
mod attribute_parse_fmt_args_spec {
    use itertools::Itertools as _;
    use quote::ToTokens;
    use syn;

    use super::Attribute;

    fn assert<'a>(input: &'a str, parsed: impl AsRef<[&'a str]>) {
        let parsed = parsed.as_ref();
        match syn::parse_str::<Attribute>(&format!("(\"\", {})", input)).unwrap() {
            Attribute::Fmt(fmt) => {
                let fmt_args = fmt
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
            attrs @ Attribute::Bounds(_) => {
                panic!("Expected `Attribute::Fmt`, found: {attrs:?}");
            }
        }
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
