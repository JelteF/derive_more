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

pub mod new {
    use std::{borrow::Cow, iter};

    use proc_macro2::{Span, TokenStream};
    use quote::quote;
    use syn::{
        ext::IdentExt as _,
        parse::{discouraged::Speculative as _, Parse as _, ParseStream, Parser},
        punctuated::Punctuated,
        spanned::Spanned as _,
        token, Error, Ident, Result,
    };

    use crate::{
        parsing::Type,
        utils::{legacy_types_attribute_error, Either, EitherExt as _},
    };

    pub fn expand(input: &syn::DeriveInput, _: &'static str) -> Result<TokenStream> {
        let data = match &input.data {
            syn::Data::Struct(data) => Ok(data),
            syn::Data::Enum(e) => Err(Error::new(
                e.enum_token.span(),
                "`Into` cannot be derived for enums",
            )),
            syn::Data::Union(u) => Err(Error::new(
                u.union_token.span(),
                "`Into` cannot be derived for unions",
            )),
        }?;
        let attr =
            Attribute::parse_attrs(&input.attrs, &data.fields)?.unwrap_or_else(|| {
                Attribute {
                    owned: Some(Punctuated::new()),
                    r#ref: None,
                    ref_mut: None,
                }
            });
        let ident = &input.ident;

        let expand = |tys: Option<Punctuated<_, _>>, r: bool, m: bool| {
            let Some(tys) = tys else {
                return iter::empty().left();
            };

            let lf = r.then(|| {
                syn::Lifetime::new("'__deriveMoreLifetime", Span::call_site())
            });
            let r = r.then(|| token::And::default());
            let m = m.then(|| token::Mut::default());

            if tys.is_empty() {
                iter::once(Type::tuple(data.fields.iter().map(|f| &f.ty))).left()
            } else {
                tys.into_iter().right()
            }
            .map(move |ty| {
                let gens = if let Some(lf) = lf.clone() {
                    let mut gens = input.generics.clone();
                    gens.params.push(syn::LifetimeDef::new(lf).into());
                    Cow::Owned(gens)
                } else {
                    Cow::Borrowed(&input.generics)
                };
                let (impl_gens, _, where_clause) = gens.split_for_impl();
                let (_, ty_gens, _) = input.generics.split_for_impl();
                let fields_idents = data.fields.iter().enumerate().map(|(i, f)| {
                    f.ident
                        .as_ref()
                        .map_or_else(|| syn::Index::from(i).right(), Either::Left)
                });

                // TODO: validate tuple
                let tys = match ty {
                    Type::Tuple { items, .. } => items.into_iter().left(),
                    Type::Other(other) => iter::once(other).right(),
                }
                .collect::<Vec<_>>();

                Ok(quote! {
                    #[automatically_derived]
                    impl #impl_gens ::core::convert::From<#r #lf #m #ident #ty_gens>
                        for ( #( #r #lf #m #tys ),* )
                        #where_clause
                    {
                        #[inline]
                        fn from(original: #r #lf #m #ident #ty_gens) -> Self {
                            (#(
                                <#r #m #tys as ::core::convert::From<_>>::from(
                                    #r #m original. #fields_idents
                                )
                            ),*)
                        }
                    }
                })
            })
            .right()
        };

        [
            expand(attr.owned, false, false),
            expand(attr.r#ref, true, false),
            expand(attr.ref_mut, true, true),
        ]
        .into_iter()
        .flatten()
        .collect()
    }

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

                    let field_attr = Parser::parse2(
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

            let parse_inner = |ahead, types: &mut Option<_>| {
                use proc_macro2::Delimiter::Parenthesis;

                content.advance_to(&ahead);
                let types = types.get_or_insert_with(Punctuated::new);

                if content.peek(token::Paren) {
                    let error_span =
                        content.cursor().group(Parenthesis).map(|(_, span, _)| span);
                    let inner;
                    syn::parenthesized!(inner in content);
                    let error_span = error_span.unwrap_or_else(|| unreachable!());

                    let ahead = inner.fork();
                    if ahead
                        .parse::<syn::Path>()
                        .ok()
                        .filter(|p| p.is_ident("types"))
                        .is_some()
                    {
                        return Err(legacy_types_attribute_error(
                            &ahead, error_span, fields,
                        ));
                    }

                    types.extend(
                        inner
                            .parse_terminated::<_, token::Comma>(Type::parse)?
                            .into_pairs(),
                    );
                }

                if content.peek(token::Comma) {
                    let comma = content.parse::<token::Comma>()?;
                    if !types.empty_or_trailing() {
                        types.push_punct(comma);
                    }
                }

                Ok(())
            };

            while !content.is_empty() {
                let ahead = content.fork();
                let res = if ahead.peek(Ident::peek_any) {
                    ahead.call(Ident::parse_any).map(Into::into)
                } else {
                    ahead.parse::<syn::Path>()
                };
                match res {
                    Ok(p) if p.is_ident("owned") => {
                        parse_inner(ahead, &mut out.owned)?;
                    }
                    Ok(p) if p.is_ident("ref") => parse_inner(ahead, &mut out.r#ref)?,
                    Ok(p) if p.is_ident("ref_mut") => {
                        parse_inner(ahead, &mut out.ref_mut)?;
                    }
                    Ok(p) if p.is_ident("types") => {
                        return Err(legacy_types_attribute_error(
                            &ahead,
                            p.span(),
                            fields,
                        ));
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
}
