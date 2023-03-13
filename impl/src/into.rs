use std::iter;

use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse::Result, DeriveInput};

use crate::utils::{add_extra_generic_param, AttrParams, MultiFieldData, State};

/// Provides the hook to expand `#[derive(Into)]` into an implementation of `Into`
pub fn expand(input: &DeriveInput, trait_name: &'static str) -> Result<TokenStream> {
    let state = State::with_attr_params(
        input,
        trait_name,
        quote! { ::core::convert },
        trait_name.to_lowercase(),
        AttrParams {
            enum_: vec!["ignore", "owned", "ref", "ref_mut"],
            variant: vec!["ignore", "owned", "ref", "ref_mut"],
            struct_: vec!["ignore", "owned", "ref", "ref_mut", "types"],
            field: vec!["ignore"],
        },
    )?;
    let MultiFieldData {
        variant_info,
        field_types,
        field_idents,
        input_type,
        ..
    } = state.enabled_fields_data();

    let mut tokens = TokenStream::new();

    for ref_type in variant_info.ref_types() {
        let reference = ref_type.reference();
        let lifetime = ref_type.lifetime();
        let reference_with_lifetime = ref_type.reference_with_lifetime();

        let generics_impl;
        let (_, ty_generics, where_clause) = input.generics.split_for_impl();
        let (impl_generics, _, _) = if ref_type.is_ref() {
            generics_impl = add_extra_generic_param(&input.generics, lifetime);
            generics_impl.split_for_impl()
        } else {
            input.generics.split_for_impl()
        };

        let additional_types = variant_info.additional_types(ref_type);
        for explicit_type in iter::once(None).chain(additional_types.iter().map(Some)) {
            let into_types: Vec<_> = field_types
                .iter()
                .map(|field_type| {
                    // No, `.unwrap_or()` won't work here, because we use different types.
                    if let Some(type_) = explicit_type {
                        quote! { #reference_with_lifetime #type_ }
                    } else {
                        quote! { #reference_with_lifetime #field_type }
                    }
                })
                .collect();

            let initializers = field_idents.iter().map(|field_ident| {
                if let Some(type_) = explicit_type {
                    quote! { <#reference #type_>::from(#reference original.#field_ident) }
                } else {
                    quote! { #reference original.#field_ident }
                }
            });

            (quote! {
                #[automatically_derived]
                impl #impl_generics
                     ::core::convert::From<#reference_with_lifetime #input_type #ty_generics> for
                     (#(#into_types),*)
                     #where_clause
                {
                    #[inline]
                    fn from(original: #reference_with_lifetime #input_type #ty_generics) -> Self {
                        (#(#initializers),*)
                    }
                }
            }).to_tokens(&mut tokens);
        }
    }
    Ok(tokens)
}

mod new {
    use std::iter;

    use quote::ToTokens as _;
    use syn::{
        parse::{discouraged::Speculative as _, Parse, ParseStream, Parser},
        punctuated::Punctuated,
        spanned::Spanned as _,
        token, Error, Result,
    };

    use crate::{parsing::Type, utils::Either};

    #[derive(Debug, Default)]
    struct Attribute {
        owned: Option<Punctuated<Type, token::Comma>>,
        r#ref: Option<Punctuated<Type, token::Comma>>,
        ref_mut: Option<Punctuated<Type, token::Comma>>,
    }

    impl Attribute {
        /// Parses [`Attribute`] from the provided [`syn::Attribute`]s.
        fn parse_attrs(
            attrs: impl AsRef<[syn::Attribute]>,
            fields: &syn::Fields,
        ) -> Result<Option<Self>> {
            fn infer<T>(v: T) -> T
            where
                T: for<'a> FnOnce(ParseStream<'a>) -> Result<Attribute>,
            {
                v
            }

            attrs
                .as_ref()
                .iter()
                .filter(|attr| attr.path.is_ident("into"))
                .try_fold(None, |mut attrs, attr| {
                    let merge = |out: &mut Option<_>, tys| match (out.as_mut(), tys) {
                        (None, Some(tys)) => {
                            *out = Some::<Punctuated<_, _>>(tys);
                        }
                        (Some(out), Some(tys)) => out.extend(tys),
                        (Some(_), None) | (None, None) => {}
                    };

                    let field_attr: Self = Parser::parse2(
                        infer(|stream| Self::parse(stream, fields)),
                        attr.tokens.clone(),
                    )?;
                    let out = attrs.get_or_insert_with(Self::default);
                    merge(&mut out.owned, field_attr.owned);
                    merge(&mut out.r#ref, field_attr.r#ref);
                    merge(&mut out.ref_mut, field_attr.ref_mut);

                    Ok(attrs)
                })
        }

        fn parse(input: ParseStream<'_>, fields: &syn::Fields) -> Result<Self> {
            let content;
            syn::parenthesized!(content in input);

            let mut out = Self::default();

            while !content.is_empty() {
                let ahead = content.fork();
                check_legacy_attribute(&ahead, fields)?;
                let res = ahead.parse::<syn::Path>();
                check_legacy_attribute(&ahead, fields)?;
                match res {
                    Ok(p) if p.is_ident("owned") => {
                        check_legacy_attribute(&ahead, fields)?;
                        content.advance_to(&ahead);
                        let inner;
                        syn::parenthesized!(inner in content);
                        out.owned.get_or_insert_with(Punctuated::new).extend(
                            inner
                                .parse_terminated::<_, token::Comma>(Type::parse)?
                                .into_pairs(),
                        );
                        if content.peek(token::Comma) {
                            let _ = content.parse::<token::Comma>()?;
                        }
                    }
                    Ok(p) if p.is_ident("ref") => {
                        content.advance_to(&ahead);
                        let inner;
                        syn::parenthesized!(inner in content);
                        out.r#ref.get_or_insert_with(Punctuated::new).extend(
                            inner
                                .parse_terminated::<_, token::Comma>(Type::parse)?
                                .into_pairs(),
                        );
                        if content.peek(token::Comma) {
                            let _ = content.parse::<token::Comma>()?;
                        }
                    }
                    Ok(p) if p.is_ident("ref_mut") => {
                        content.advance_to(&ahead);
                        let inner;
                        syn::parenthesized!(inner in content);
                        out.ref_mut.get_or_insert_with(Punctuated::new).extend(
                            inner
                                .parse_terminated::<_, token::Comma>(Type::parse)?
                                .into_pairs(),
                        );
                        if content.peek(token::Comma) {
                            let _ = content.parse::<token::Comma>()?;
                        }
                    }
                    _ => {
                        out.owned
                            .get_or_insert_with(Punctuated::new)
                            .push_value(content.parse::<Type>()?);
                        if content.peek(token::Comma) {
                            out.owned
                                .get_or_insert_with(Punctuated::new)
                                .push_punct(content.parse::<token::Comma>()?)
                        }
                    }
                }
            }

            Ok(out)
        }
    }

    fn check_legacy_attribute(
        input: ParseStream<'_>,
        fields: &syn::Fields,
    ) -> Result<()> {
        use proc_macro2::Delimiter::Parenthesis;

        let input = input.fork();

        if input
            .parse::<syn::Path>()
            .ok()
            .filter(|p| p.is_ident("types"))
            .is_none()
        {
            return Ok(());
        }

        let error_span = input.cursor().group(Parenthesis).map(|(_, span, _)| span);
        let content;
        syn::parenthesized!(content in input);
        let error_span = error_span.unwrap_or_else(|| unreachable!());

        let types = content
            .parse_terminated::<_, token::Comma>(syn::NestedMeta::parse)?
            .into_iter()
            .map(|meta| {
                let value = match meta {
                    syn::NestedMeta::Meta(meta) => meta.into_token_stream().to_string(),
                    syn::NestedMeta::Lit(syn::Lit::Str(str)) => str.value(),
                    meta => {
                        return Err(Error::new(
                            meta.span(),
                            format!(
                                "expected path (`i32`) of string literal (`\"...\"`), \
                                 found: `{}`",
                                meta.into_token_stream(),
                            ),
                        ))
                    }
                };
                Ok(if fields.len() > 1 {
                    format!(
                        "({})",
                        fields
                            .iter()
                            .map(|_| value.clone())
                            .collect::<Vec<_>>()
                            .join(", "),
                    )
                } else {
                    value
                })
            })
            .chain(match fields.len() {
                0 => Either::Left(iter::empty()),
                1 => Either::Right(iter::once(Ok(fields
                    .iter()
                    .next()
                    .unwrap_or_else(|| unreachable!("fields.len() == 1"))
                    .ty
                    .to_token_stream()
                    .to_string()))),
                _ => Either::Right(iter::once(Ok(format!(
                    "({})",
                    fields
                        .iter()
                        .map(|f| f.ty.to_token_stream().to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
                )))),
            })
            .collect::<Result<Vec<_>>>()?
            .join(", ");

        Err(Error::new(
            error_span,
            format!("legacy syntax, remove `types` and use `{types}` instead"),
        ))
    }
}
