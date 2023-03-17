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
    use quote::{quote, ToTokens as _};
    use syn::{
        ext::IdentExt as _,
        parse::{discouraged::Speculative as _, Parse, ParseStream, Parser},
        punctuated::Punctuated,
        spanned::Spanned as _,
        token, Error, Ident, Result,
    };

    use crate::{
        parsing::Type,
        utils::{Either, EitherExt as _},
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

            let fields = data
                .fields
                .iter()
                .enumerate()
                .filter_map(|(i, f)| match SkipFieldAttribute::parse_attrs(&f.attrs) {
                    Ok(None) => Some(Ok((
                        &f.ty,
                        f.ident
                            .as_ref()
                            .map_or_else(|| syn::Index::from(i).right(), Either::Left),
                    ))),
                    Ok(Some(_)) => None,
                    Err(e) => Some(Err(e)),
                })
                .collect::<Result<Vec<_>>>();
            let fields = match fields {
                Ok(fields) => fields,
                Err(e) => return iter::once(Err(e)).left().right(),
            };
            let (fields, fields_idents): (Vec<_>, Vec<_>) = fields.into_iter().unzip();

            if tys.is_empty() {
                iter::once(Type::tuple(fields)).left()
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
            use proc_macro2::Delimiter::Parenthesis;

            let error_span = input.cursor().group(Parenthesis).map(|(_, span, _)| span);
            let content;
            syn::parenthesized!(content in input);
            let error_span = error_span.unwrap_or_else(|| unreachable!());

            legacy_types_attribute_error(&content, error_span, fields)?;

            let mut out = Self::default();

            let parse_inner = |ahead, types: &mut Option<_>| {
                content.advance_to(&ahead);

                let types = types.get_or_insert_with(Punctuated::new);
                if content.peek(token::Paren) {
                    let inner;
                    syn::parenthesized!(inner in content);

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

            let mut has_wrapped_type = false;
            let mut top_level_type = None;

            while !content.is_empty() {
                let ahead = content.fork();
                let res = if ahead.peek(Ident::peek_any) {
                    ahead.call(Ident::parse_any).map(Into::into)
                } else {
                    ahead.parse::<syn::Path>()
                };
                match res {
                    Ok(p) if p.is_ident("owned") => {
                        has_wrapped_type = true;
                        parse_inner(ahead, &mut out.owned)?;
                    }
                    Ok(p) if p.is_ident("ref") => {
                        has_wrapped_type = true;
                        parse_inner(ahead, &mut out.r#ref)?;
                    }
                    Ok(p) if p.is_ident("ref_mut") => {
                        has_wrapped_type = true;
                        parse_inner(ahead, &mut out.ref_mut)?;
                    }
                    _ => {
                        let ty = content.parse::<Type>()?;
                        let _ = top_level_type.get_or_insert_with(|| ty.clone());
                        out.owned.get_or_insert_with(Punctuated::new).push_value(ty);

                        if content.peek(token::Comma) {
                            out.owned
                                .get_or_insert_with(Punctuated::new)
                                .push_punct(content.parse::<token::Comma>()?)
                        }
                    }
                }
            }

            if let Some(ty) = top_level_type.filter(|_| has_wrapped_type) {
                Err(Error::new(
                    ty.span(),
                    format!(
                        "mixing regular types with wrapped into \
                        `owned`/`ref`/`ref_mut` is not allowed, try wrapping \
                         this type into `owned({ty}), ref({ty}), ref_mut({ty})`",
                        ty = ty.into_token_stream(),
                    ),
                ))
            } else {
                Ok(out)
            }
        }
    }

    struct SkipFieldAttribute;

    impl SkipFieldAttribute {
        /// Parses [`SkipFieldAttribute`] from the provided [`syn::Attribute`]s.
        fn parse_attrs(attrs: impl AsRef<[syn::Attribute]>) -> Result<Option<Self>> {
            Ok(attrs
                .as_ref()
                .iter()
                .filter(|attr| attr.path.is_ident("into"))
                .try_fold(None, |mut attrs, attr| {
                    let field_attr =
                        syn::parse2::<SkipFieldAttribute>(attr.tokens.clone())?;
                    if let Some((path, _)) = attrs.replace((&attr.path, field_attr)) {
                        Err(Error::new(
                            path.span(),
                            "only single `#[into(...)]` attribute is allowed here",
                        ))
                    } else {
                        Ok(attrs)
                    }
                })?
                .map(|(_, attr)| attr))
        }
    }

    impl Parse for SkipFieldAttribute {
        fn parse(input: ParseStream) -> Result<Self> {
            let content;
            syn::parenthesized!(content in input);

            match content.parse::<syn::Path>()? {
                p if p.is_ident("skip") | p.is_ident("ignore") => Ok(Self),
                p => Err(Error::new(
                    p.span(),
                    format!("expected `skip`, found: `{}`", p.into_token_stream()),
                )),
            }
        }
    }

    fn legacy_types_attribute_error(
        tokens: ParseStream<'_>,
        span: Span,
        fields: &syn::Fields,
    ) -> Result<()> {
        let tokens = tokens.fork();

        let map_ty = |s: String| {
            if fields.len() > 1 {
                format!(
                    "({})",
                    (0..fields.len())
                        .map(|_| s.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            } else {
                s
            }
        };
        let field = match fields.len() {
            0 => None,
            1 => Some(
                fields
                    .iter()
                    .next()
                    .unwrap_or_else(|| unreachable!("fields.len() == 1"))
                    .ty
                    .to_token_stream()
                    .to_string(),
            ),
            _ => Some(format!(
                "({})",
                fields
                    .iter()
                    .map(|f| f.ty.to_token_stream().to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            )),
        };

        let Ok(metas) = tokens.parse_terminated::<_, token::Comma>(syn::Meta::parse) else {
            return Ok(());
        };

        let parse_list = |list: syn::MetaList, attrs: &mut Option<Vec<_>>| {
            if !list.path.is_ident("types") {
                return None;
            }
            for meta in list.nested {
                attrs.get_or_insert_with(Vec::new).push(match meta {
                    syn::NestedMeta::Lit(syn::Lit::Str(str)) => str.value(),
                    syn::NestedMeta::Meta(syn::Meta::Path(path)) => {
                        path.into_token_stream().to_string()
                    }
                    _ => return None,
                })
            }
            Some(())
        };

        let Some((top_level, owned, ref_, ref_mut)) = metas
            .into_iter()
            .try_fold(
                (None, None, None, None),
                |(mut top_level, mut owned, mut ref_, mut ref_mut), meta| {
                    let is = |name| {
                        matches!(&meta, syn::Meta::Path(p) if p.is_ident(name))
                            || matches!(&meta, syn::Meta::List(list) if list.path.is_ident(name))
                    };
                    let parse_inner = |meta, attrs: &mut Option<_>| {
                        match meta {
                            syn::Meta::Path(_) => {
                                let _ = attrs.get_or_insert_with(Vec::new);
                                Some(())
                            }
                            syn::Meta::List(mut list) => {
                                if let syn::NestedMeta::Meta(syn::Meta::List(list)) = list.nested.pop()?.into_value() {
                                    parse_list(list, attrs)
                                } else {
                                    None
                                }
                            }
                            _ => None
                        }
                    };

                    match meta {
                        meta if is("owned") => parse_inner(meta, &mut owned),
                        meta if is("ref") => parse_inner(meta, &mut ref_),
                        meta if is("ref_mut") => parse_inner(meta, &mut ref_mut),
                        syn::Meta::List(list) => parse_list(list, &mut top_level),
                        _ => None,
                    }
                    .map(|_| (top_level, owned, ref_, ref_mut))
                },
            )
            .filter(|(top_level, owned, ref_, ref_mut)| {
                [top_level, owned, ref_, ref_mut]
                    .into_iter()
                    .any(|l| l.as_ref().map_or(false, |l| !l.is_empty()))
            })
        else {
            return Ok(());
        };

        if [&owned, &ref_, &ref_mut].into_iter().any(Option::is_some) {
            let format = |list: Option<Vec<_>>, name: &str| match list {
                Some(l)
                    if top_level.as_ref().map_or(true, Vec::is_empty)
                        && l.is_empty() =>
                {
                    Some(name.to_owned())
                }
                Some(l) => Some(format!(
                    "{}({})",
                    name,
                    l.into_iter()
                        .chain(top_level.clone().into_iter().flatten())
                        .map(map_ty)
                        .chain(field.clone())
                        .collect::<Vec<_>>()
                        .join(", "),
                )),
                None => None,
            };
            let format = [
                format(owned, "owned"),
                format(ref_, "ref"),
                format(ref_mut, "ref_mut"),
            ]
            .into_iter()
            .flatten()
            .collect::<Vec<_>>()
            .join(", ");

            Err(Error::new(
                span,
                format!("legacy syntax, use `{format}` instead"),
            ))
        } else {
            Err(Error::new(
                span,
                format!(
                    "legacy syntax, remove `types` and use `{}` instead",
                    top_level.unwrap_or_else(|| unreachable!()).join(", "),
                ),
            ))
        }
    }
}
