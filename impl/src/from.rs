use std::iter;

use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote, ToTokens as _, TokenStreamExt as _};
use syn::{
    parse::{discouraged::Speculative as _, Parse, ParseStream},
    parse_quote,
    punctuated::Punctuated,
    spanned::Spanned as _,
    token, Error, Ident, Result,
};

use crate::parsing::Type;

pub fn expand(input: &syn::DeriveInput, _: &'static str) -> Result<TokenStream> {
    match &input.data {
        syn::Data::Struct(data) => Expansion {
            attrs: StructAttribute::parse_attrs(&input.attrs)?
                .map(Into::into)
                .as_ref(),
            ident: &input.ident,
            variant: None,
            fields: &data.fields,
            generics: &input.generics,
            has_forward: false,
        }
        .expand(),
        syn::Data::Enum(data) => {
            let mut has_forward = false;
            let attrs = data
                .variants
                .iter()
                .map(|variant| {
                    let attrs = VariantAttribute::parse_attrs(&variant.attrs)?;
                    if matches!(attrs, Some(VariantAttribute::Forward)) {
                        has_forward = true;
                    }
                    Ok(attrs)
                })
                .collect::<Result<Vec<_>>>()?;

            data.variants
                .iter()
                .zip(&attrs)
                .map(|(variant, attrs)| {
                    Expansion {
                        attrs: attrs.as_ref(),
                        ident: &input.ident,
                        variant: Some(&variant.ident),
                        fields: &variant.fields,
                        generics: &input.generics,
                        has_forward,
                    }
                    .expand()
                })
                .collect()
        }
        syn::Data::Union(data) => Err(Error::new(
            data.union_token.span(),
            "`From` cannot be derived for unions",
        )),
    }
}

enum StructAttribute {
    Types(Punctuated<Type, token::Comma>),
    Forward,
}

impl StructAttribute {
    /// Parses [`StructAttribute`] from the provided [`syn::Attribute`]s.
    fn parse_attrs(attrs: impl AsRef<[syn::Attribute]>) -> Result<Option<Self>> {
        Ok(attrs
            .as_ref()
            .iter()
            .filter(|attr| attr.path.is_ident("from"))
            .try_fold(None, |attrs, attr| {
                let field_attr = syn::parse2::<StructAttribute>(attr.tokens.clone())?;
                match (attrs, field_attr) {
                    (
                        Some((path, StructAttribute::Types(mut tys))),
                        StructAttribute::Types(more),
                    ) => {
                        tys.extend(more);
                        Ok(Some((path, StructAttribute::Types(tys))))
                    }
                    (None, field_attr) => Ok(Some((&attr.path, field_attr))),
                    _ => Err(Error::new(
                        attr.path.span(),
                        "Only single `#[from(...)]` attribute is allowed here",
                    )),
                }
            })?
            .map(|(_, attr)| attr))
    }
}

impl Parse for StructAttribute {
    fn parse(input: ParseStream) -> Result<Self> {
        use proc_macro2::Delimiter::Parenthesis;

        let error_span = input.cursor().group(Parenthesis).map(|(_, span, _)| span);
        let content;
        syn::parenthesized!(content in input);
        let error_span = error_span.unwrap_or_else(|| unreachable!());

        let ahead = content.fork();
        match ahead.parse::<syn::Path>() {
            Ok(p) if p.is_ident("forward") => {
                content.advance_to(&ahead);
                Ok(Self::Forward)
            }
            Ok(p) if p.is_ident("types") => legacy_error(&ahead, error_span),
            _ => content.parse_terminated(Type::parse).map(Self::Types),
        }
    }
}

enum VariantAttribute {
    Types(Punctuated<Type, token::Comma>),
    Forward,
    Skip,
    From,
}

impl VariantAttribute {
    /// Parses [`VariantAttribute`] from the provided [`syn::Attribute`]s.
    fn parse_attrs(attrs: impl AsRef<[syn::Attribute]>) -> Result<Option<Self>> {
        Ok(attrs
            .as_ref()
            .iter()
            .filter(|attr| attr.path.is_ident("from"))
            .try_fold(None, |mut attrs, attr| {
                let field_attr = syn::parse2::<VariantAttribute>(attr.tokens.clone())?;
                if let Some((path, _)) = attrs.replace((&attr.path, field_attr)) {
                    Err(Error::new(
                        path.span(),
                        "Only single `#[from(...)]` attribute is allowed here",
                    ))
                } else {
                    Ok(attrs)
                }
            })?
            .map(|(_, attr)| attr))
    }
}

impl Parse for VariantAttribute {
    fn parse(input: ParseStream) -> Result<Self> {
        use proc_macro2::Delimiter::Parenthesis;

        if input.is_empty() {
            return Ok(Self::From);
        }

        let error_span = input.cursor().group(Parenthesis).map(|(_, span, _)| span);
        let content;
        syn::parenthesized!(content in input);
        let error_span = error_span.unwrap_or_else(|| unreachable!());

        let ahead = content.fork();
        match ahead.parse::<syn::Path>() {
            Ok(p) if p.is_ident("forward") => {
                content.advance_to(&ahead);
                Ok(Self::Forward)
            }
            Ok(p) if p.is_ident("skip") || p.is_ident("ignore") => {
                content.advance_to(&ahead);
                Ok(Self::Skip)
            }
            Ok(p) if p.is_ident("types") => legacy_error(&ahead, error_span),
            _ => content.parse_terminated(Type::parse).map(Self::Types),
        }
    }
}

impl From<StructAttribute> for VariantAttribute {
    fn from(value: StructAttribute) -> Self {
        match value {
            StructAttribute::Types(tys) => Self::Types(tys),
            StructAttribute::Forward => Self::Forward,
        }
    }
}

struct Expansion<'a> {
    attrs: Option<&'a VariantAttribute>,
    ident: &'a Ident,
    variant: Option<&'a Ident>,
    fields: &'a syn::Fields,
    generics: &'a syn::Generics,
    has_forward: bool,
}

impl<'a> Expansion<'a> {
    fn expand(&self) -> Result<TokenStream> {
        let ident = self.ident;
        let field_tys = self.fields.iter().map(|f| &f.ty).collect::<Vec<_>>();
        let (impl_gens, ty_gens, where_clause) = self.generics.split_for_impl();

        // TODO: docs
        let skip_variant =
            self.has_forward || (self.variant.is_some() && self.fields.is_empty());
        match (self.attrs, skip_variant) {
            (Some(VariantAttribute::Types(tys)), _) => {
                tys.iter().map(|ty| {
                    let variant = self.variant.iter();

                    let mut from_tys = self.validate_type(ty)?;
                    let init = self.expand_fields(|ident, ty, index| {
                        let ident = ident.into_iter();
                        let index = index.into_iter();
                        let from_ty = from_tys.next().unwrap_or_else(|| unreachable!());
                        quote! {
                            #( #ident: )* <#ty as ::core::convert::From<#from_ty>>::from(
                                value #( .#index )*
                            ),
                        }
                    });

                    Ok(quote! {
                        #[automatically_derived]
                        impl #impl_gens ::core::convert::From<#ty>
                            for #ident #ty_gens
                            #where_clause
                        {
                            #[inline]
                            fn from(value: #ty) -> Self {
                                #ident #( :: #variant )* #init
                            }
                        }
                    })
                })
                .collect()
            }
            (Some(VariantAttribute::From), _) | (None, false) => {
                let variant = self.variant.iter();
                let init = self.expand_fields(|ident, _, index| {
                    let ident = ident.into_iter();
                    let index = index.into_iter();
                    quote! { #( #ident: )* value #( . #index )*, }
                });

                Ok(quote! {
                    #[automatically_derived]
                    impl #impl_gens ::core::convert::From<(#( #field_tys ),*)>
                        for #ident #ty_gens
                        #where_clause
                    {
                        #[inline]
                        fn from(value: (#( #field_tys ),*)) -> Self {
                            #ident #( :: #variant )* #init
                        }
                    }
                })
            }
            (Some(VariantAttribute::Forward), _) => {
                let mut i = 0;
                let mut gen_idents = Vec::with_capacity(self.fields.len());
                let init = self.expand_fields(|ident, ty, index| {
                    let ident = ident.into_iter();
                    let index = index.into_iter();
                    let gen_ident = format_ident!("__FromT{i}");
                    let out = quote! {
                        #( #ident: )* <#ty as ::core::convert::From<#gen_ident>>::from(
                            value #( .#index )*
                        ),
                    };
                    gen_idents.push(gen_ident);
                    i += 1;
                    out
                });

                let variant = self.variant.iter();
                let generics = {
                    let mut generics = self.generics.clone();
                    for (ty, ident) in field_tys.iter().zip(&gen_idents) {
                        generics.make_where_clause().predicates.push(
                            parse_quote! { #ty: ::core::convert::From<#ident> },
                        );
                        generics
                            .params
                            .push(syn::TypeParam::from(ident.clone()).into());
                    }
                    generics
                };
                let (impl_gens, _, where_clause) = generics.split_for_impl();

                Ok(quote! {
                    #[automatically_derived]
                    impl #impl_gens ::core::convert::From<(#( #gen_idents ),*)>
                        for #ident #ty_gens
                        #where_clause
                    {
                        #[inline]
                        fn from(value: (#( #gen_idents ),*)) -> Self {
                            #ident #(:: #variant)* #init
                        }
                    }
                })
            }
            (Some(VariantAttribute::Skip), _) | (None, true) => {
                Ok(TokenStream::new())
            }
        }
    }

    fn expand_fields(
        &self,
        mut wrap: impl FnMut(Option<&Ident>, &syn::Type, Option<syn::Index>) -> TokenStream,
    ) -> TokenStream {
        let surround = match self.fields {
            syn::Fields::Named(_) | syn::Fields::Unnamed(_) => {
                Some(|tokens| match self.fields {
                    syn::Fields::Named(named) => {
                        let mut out = TokenStream::new();
                        named
                            .brace_token
                            .surround(&mut out, |out| out.append_all(tokens));
                        out
                    }
                    syn::Fields::Unnamed(unnamed) => {
                        let mut out = TokenStream::new();
                        unnamed
                            .paren_token
                            .surround(&mut out, |out| out.append_all(tokens));
                        out
                    }
                    syn::Fields::Unit => unreachable!(),
                })
            }
            syn::Fields::Unit => None,
        };

        surround
            .map(|surround| {
                surround(if self.fields.len() == 1 {
                    let field = self
                        .fields
                        .iter()
                        .next()
                        .unwrap_or_else(|| unreachable!("self.fields.len() == 1"));
                    wrap(field.ident.as_ref(), &field.ty, None)
                } else {
                    self.fields
                        .iter()
                        .enumerate()
                        .map(|(i, field)| {
                            wrap(field.ident.as_ref(), &field.ty, Some(i.into()))
                        })
                        .collect()
                })
            })
            .unwrap_or_default()
    }

    fn validate_type<'t>(
        &self,
        ty: &'t Type,
    ) -> Result<impl Iterator<Item = &'t TokenStream>> {
        match ty {
            Type::Tuple { items, .. } if self.fields.len() > 1 => {
                if self.fields.len() > items.len() {
                    return Err(Error::new(
                        ty.span(),
                        format!(
                            "Wrong tuple length: expected {}, found {}. \
                             Consider adding {} more type{}: `({})`",
                            self.fields.len(),
                            items.len(),
                            self.fields.len() - items.len(),
                            if self.fields.len() - items.len() > 1 {
                                "s"
                            } else {
                                ""
                            },
                            items
                                .iter()
                                .map(|item| item.to_string())
                                .chain(
                                    (0..(self.fields.len() - items.len()))
                                        .map(|_| "_".to_string())
                                )
                                .collect::<Vec<_>>()
                                .join(", "),
                        ),
                    ));
                } else if self.fields.len() < items.len() {
                    return Err(Error::new(
                        ty.span(),
                        format!(
                            "Wrong tuple length: expected {}, found {}. \
                             Consider removing last {} type{}: `({})`",
                            self.fields.len(),
                            items.len(),
                            items.len() - self.fields.len(),
                            if items.len() - self.fields.len() > 1 {
                                "s"
                            } else {
                                ""
                            },
                            items
                                .iter()
                                .take(self.fields.len())
                                .map(|item| item.to_string())
                                .collect::<Vec<_>>()
                                .join(", "),
                        ),
                    ));
                }
            }
            Type::Other(other) if self.fields.len() > 1 => {
                if self.fields.len() > 1 {
                    return Err(Error::new(
                        other.span(),
                        format!(
                            "Expected tuple: `({}, {})`",
                            other.to_string(),
                            (0..(self.fields.len() - 1))
                                .map(|_| "_")
                                .collect::<Vec<_>>()
                                .join(", "),
                        ),
                    ));
                }
            }
            Type::Tuple { .. } | Type::Other(_) => {}
        }
        Ok(match ty {
            Type::Tuple { items, .. } => Either::Left(items.iter()),
            Type::Other(other) => Either::Right(iter::once(other)),
        })
    }
}

enum Either<L, R> {
    Left(L),
    Right(R),
}

impl<L, R, T> Iterator for Either<L, R>
where
    L: Iterator<Item = T>,
    R: Iterator<Item = T>,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Either::Left(left) => left.next(),
            Either::Right(right) => right.next(),
        }
    }
}

fn legacy_error<T>(tokens: ParseStream<'_>, span: Span) -> Result<T> {
    let content;
    syn::parenthesized!(content in tokens);

    let types = content
        .parse_terminated::<_, token::Comma>(syn::NestedMeta::parse)?
        .into_iter()
        .map(|meta| match meta {
            syn::NestedMeta::Meta(meta) => meta.into_token_stream().to_string(),
            syn::NestedMeta::Lit(syn::Lit::Str(str)) => str.value(),
            syn::NestedMeta::Lit(_) => unreachable!(),
        })
        .collect::<Vec<_>>()
        .join(", ");

    Err(Error::new(
        span,
        format!("legacy syntax, remove `types` and use `{types}` instead"),
    ))
}
