//! Implementation of an [`Into`] derive macro.

use std::{borrow::Cow, iter};

use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens as _};
use syn::{
    ext::IdentExt as _,
    parse::{discouraged::Speculative as _, Parse, ParseStream},
    punctuated::Punctuated,
    spanned::Spanned as _,
    token, Field, Ident,
};

use crate::{
    parsing::Type,
    utils::{polyfill, Either, FieldsExt},
};

/// Expands an [`Into`] derive macro.
pub fn expand(input: &syn::DeriveInput, _: &'static str) -> syn::Result<TokenStream> {
    let data = match &input.data {
        syn::Data::Struct(data) => Ok(data),
        syn::Data::Enum(e) => Err(syn::Error::new(
            e.enum_token.span(),
            "`Into` cannot be derived for enums",
        )),
        syn::Data::Union(u) => Err(syn::Error::new(
            u.union_token.span(),
            "`Into` cannot be derived for unions",
        )),
    }?;

    let struct_attr = StructAttribute::parse_attrs(&input.attrs, &data.fields)?;

    let (fields, args) = data
        .fields
        .iter()
        .enumerate()
        .filter_map(|(i, f)| {
            let field_attr = match FieldAttribute::parse_attrs(&f.attrs, f) {
                Ok(field_attr) => field_attr,
                Err(err) => return Some(Err(err)),
            };

            let args = match field_attr {
                Some(FieldAttribute::Skip) => return None,
                Some(FieldAttribute::Args(args)) => Some(args),
                None => None,
            };

            let ident = f
                .ident
                .as_ref()
                .map_or_else(|| Either::Right(syn::Index::from(i)), Either::Left);

            Some(Ok(((&f.ty, ident), args)))
        })
        .collect::<syn::Result<Vec<_>>>()?
        .into_iter()
        .unzip::<_, _, Vec<_>, Vec<_>>();

    // Expand the version with all non-skipped fields if either
    // there's an explicit struct attribute
    // or there are no conversions into specific fields
    let struct_attr = struct_attr.or_else(|| {
        args.iter()
            .all(|arg| arg.is_none())
            .then(IntoArgs::all_owned)
            .map(|args| StructAttribute { args })
    });

    let mut expands = fields
        .iter()
        .zip(args)
        .filter_map(|((field_ty, ident), args)| {
            args.map(|args| {
                expand_args(
                    &input.generics,
                    &input.ident,
                    std::slice::from_ref(field_ty),
                    std::slice::from_ref(ident),
                    args,
                )
            })
        })
        .collect::<syn::Result<TokenStream>>()?;

    if let Some(struct_attr) = struct_attr {
        let (fields_tys, fields_idents) =
            fields.into_iter().unzip::<_, _, Vec<_>, Vec<_>>();

        let struct_expand = expand_args(
            &input.generics,
            &input.ident,
            &fields_tys,
            &fields_idents,
            struct_attr.args,
        )?;

        expands.extend(struct_expand);
    }

    Ok(expands)
}

fn expand_args(
    generics: &syn::Generics,
    ident: &Ident,
    fields_tys: &[&syn::Type],
    fields_idents: &[Either<&Ident, syn::Index>],
    args: IntoArgs,
) -> syn::Result<TokenStream> {
    let expand_one = |tys: Option<Punctuated<_, _>>, r: bool, m: bool| {
        let Some(tys) = tys else {
            return Either::Left(iter::empty());
        };

        let lf =
            r.then(|| syn::Lifetime::new("'__derive_more_into", Span::call_site()));
        let r = r.then(token::And::default);
        let m = m.then(token::Mut::default);

        let gens = if let Some(lf) = lf.clone() {
            let mut gens = generics.clone();
            gens.params.push(syn::LifetimeParam::new(lf).into());
            Cow::Owned(gens)
        } else {
            Cow::Borrowed(generics)
        };

        Either::Right(
            if tys.is_empty() {
                Either::Left(iter::once(Type::tuple(fields_tys.clone())))
            } else {
                Either::Right(tys.into_iter())
            }
            .map(move |ty| {
                let tys = fields_tys.validate_type(&ty)?.collect::<Vec<_>>();
                let (impl_gens, _, where_clause) = gens.split_for_impl();
                let (_, ty_gens, _) = generics.split_for_impl();

                Ok(quote! {
                    #[automatically_derived]
                    impl #impl_gens ::core::convert::From<#r #lf #m #ident #ty_gens>
                     for ( #( #r #lf #m #tys ),* ) #where_clause
                    {
                        #[inline]
                        fn from(value: #r #lf #m #ident #ty_gens) -> Self {
                            (#(
                                <#r #m #tys as ::core::convert::From<_>>::from(
                                    #r #m value. #fields_idents
                                )
                            ),*)
                        }
                    }
                })
            }),
        )
    };
    [
        expand_one(args.owned, false, false),
        expand_one(args.r#ref, true, false),
        expand_one(args.ref_mut, true, true),
    ]
    .into_iter()
    .flatten()
    .collect()
}

/// Representation of an [`Into`] derive macro struct container attribute.
///
/// ```rust,ignore
/// #[into]
/// #[into(<types>)]
/// #[into(owned(<types>), ref(<types>), ref_mut(<types>))]
/// ```
#[derive(Debug, Default)]
struct StructAttribute {
    args: IntoArgs,
}

#[derive(Debug, Default)]
struct IntoArgs {
    /// [`Type`]s wrapped into `owned(...)` or simply `#[into(...)]`.
    owned: Option<Punctuated<Type, token::Comma>>,

    /// [`Type`]s wrapped into `ref(...)`.
    r#ref: Option<Punctuated<Type, token::Comma>>,

    /// [`Type`]s wrapped into `ref_mut(...)`.
    ref_mut: Option<Punctuated<Type, token::Comma>>,
}

impl StructAttribute {
    /// Parses a [`StructAttribute`] from the provided [`syn::Attribute`]s.
    fn parse_attrs(
        attrs: impl AsRef<[syn::Attribute]>,
        fields: &syn::Fields,
    ) -> syn::Result<Option<Self>> {
        attrs
            .as_ref()
            .iter()
            .filter(|attr| attr.path().is_ident("into"))
            .try_fold(None, |mut attrs, attr| {
                let field_attr = Self::parse_attr(attr, fields)?;
                let out = attrs.get_or_insert_with(Self::default);
                merge_tys(&mut out.args.owned, field_attr.args.owned);
                merge_tys(&mut out.args.r#ref, field_attr.args.r#ref);
                merge_tys(&mut out.args.ref_mut, field_attr.args.ref_mut);

                Ok(attrs)
            })
    }

    /// Parses a single [`StructAttribute`]
    fn parse_attr(attr: &syn::Attribute, fields: &syn::Fields) -> syn::Result<Self> {
        if matches!(attr.meta, syn::Meta::Path(_)) {
            Ok(Self {
                args: IntoArgs::all_owned(),
            })
        } else {
            attr.parse_args_with(|content: ParseStream<'_>| {
                IntoArgs::parse(content, fields).map(|args| Self { args })
            })
        }
    }
}

impl IntoArgs {
    /// Parses a set of [`IntoArgs`]
    fn parse<'a, F>(content: ParseStream<'_>, fields: &'a F) -> syn::Result<Self>
    where
        F: FieldsExt + ?Sized,
        &'a F: IntoIterator<Item = &'a Field>,
    {
        check_legacy_syntax(content, fields)?;

        let mut out = Self::default();

        let parse_inner = |ahead, types: &mut Option<_>| {
            content.advance_to(&ahead);

            let types = types.get_or_insert_with(Punctuated::new);
            if content.peek(token::Paren) {
                let inner;
                syn::parenthesized!(inner in content);

                types.extend(
                    inner
                        .parse_terminated(Type::parse, token::Comma)?
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
            Err(syn::Error::new(
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

    fn all_owned() -> Self {
        Self {
            owned: Some(Punctuated::new()),
            r#ref: None,
            ref_mut: None,
        }
    }
}

/// Representation of an [`Into`] derive macro field attribute.
///
/// ```rust,ignore
/// #[into]
/// #[into(skip)]
/// #[into(<types>)]
/// #[into(owned(<types>), ref(<types>), ref_mut(<types>))]
/// ```
#[derive(Debug)]
enum FieldAttribute {
    Skip,
    Args(IntoArgs),
}

impl FieldAttribute {
    /// Parses a [`FieldAttribute`] from the provided [`syn::Attribute`]s.
    fn parse_attrs(
        attrs: impl AsRef<[syn::Attribute]>,
        field: &Field,
    ) -> syn::Result<Option<Self>> {
        attrs
            .as_ref()
            .iter()
            .filter(|attr| attr.path().is_ident("into"))
            .try_fold(None, |attrs, attr| {
                let field_attr = Self::parse_attr(attr, field)?;
                match (attrs, field_attr) {
                    (Some(Self::Args(mut args)), Self::Args(more)) => {
                        merge_tys(&mut args.owned, more.owned);
                        merge_tys(&mut args.r#ref, more.r#ref);
                        merge_tys(&mut args.ref_mut, more.ref_mut);
                        Ok(Some(Self::Args(args)))
                    }
                    (None, field_attr) => Ok(Some(field_attr)),
                    (Some(_), _) => Err(syn::Error::new(
                        attr.path().span(),
                        "only single `#[into(...)]` attribute is allowed here",
                    )),
                }
            })
    }

    /// Parses a single [`FieldAttribute`]
    fn parse_attr(attr: &syn::Attribute, field: &Field) -> syn::Result<Self> {
        if matches!(attr.meta, syn::Meta::Path(_)) {
            Ok(Self::Args(IntoArgs::all_owned()))
        } else {
            attr.parse_args_with(|content: ParseStream| Self::parse(content, field))
        }
    }

    /// Parses a single [`FieldAttribute`]'s args
    fn parse(content: ParseStream, field: &Field) -> syn::Result<Self> {
        let ahead = content.fork();
        match ahead.parse::<syn::Path>() {
            Ok(p) if p.is_ident("skip") | p.is_ident("ignore") => {
                content.advance_to(&ahead);
                Ok(Self::Skip)
            }
            _ => {
                let fields = std::slice::from_ref(field);
                IntoArgs::parse(content, fields).map(Self::Args)
            }
        }
    }
}

fn merge_tys(
    out: &mut Option<Punctuated<Type, token::Comma>>,
    tys: Option<Punctuated<Type, token::Comma>>,
) {
    match (out.as_mut(), tys) {
        (None, Some(tys)) => {
            *out = Some::<Punctuated<_, _>>(tys);
        }
        (Some(out), Some(tys)) => out.extend(tys),
        (Some(_), None) | (None, None) => {}
    };
}

/// [`Error`]ors for legacy syntax: `#[into(types(i32, "&str"))]`.
fn check_legacy_syntax<'a, F>(tokens: ParseStream<'_>, fields: &'a F) -> syn::Result<()>
where
    F: FieldsExt + ?Sized,
    &'a F: IntoIterator<Item = &'a Field>,
{
    let span = tokens.span();
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
                .into_iter()
                .next()
                .unwrap_or_else(|| unreachable!("fields.len() == 1"))
                .ty
                .to_token_stream()
                .to_string(),
        ),
        _ => Some(format!(
            "({})",
            fields
                .into_iter()
                .map(|f| f.ty.to_token_stream().to_string())
                .collect::<Vec<_>>()
                .join(", ")
        )),
    };

    let Ok(metas) = tokens.parse_terminated(polyfill::Meta::parse, token::Comma) else {
        return Ok(());
    };

    let parse_list = |list: polyfill::MetaList, attrs: &mut Option<Vec<_>>| {
        if !list.path.is_ident("types") {
            return None;
        }
        for meta in list
            .parse_args_with(Punctuated::<_, token::Comma>::parse_terminated)
            .ok()?
        {
            attrs.get_or_insert_with(Vec::new).push(match meta {
                polyfill::NestedMeta::Lit(syn::Lit::Str(str)) => str.value(),
                polyfill::NestedMeta::Meta(polyfill::Meta::Path(path)) => {
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
                        matches!(&meta, polyfill::Meta::Path(p) if p.is_ident(name))
                            || matches!(&meta, polyfill::Meta::List(list) if list.path.is_ident(name))
                    };
                    let parse_inner = |meta, attrs: &mut Option<_>| {
                        match meta {
                            polyfill::Meta::Path(_) => {
                                let _ = attrs.get_or_insert_with(Vec::new);
                                Some(())
                            }
                            polyfill::Meta::List(list) => {
                                if let polyfill::NestedMeta::Meta(polyfill::Meta::List(list)) = list
                                    .parse_args_with(Punctuated::<_, token::Comma>::parse_terminated)
                                    .ok()?
                                    .pop()?
                                    .into_value()
                                {
                                    parse_list(list, attrs)
                                } else {
                                    None
                                }
                            }
                        }
                    };

                    match meta {
                        meta if is("owned") => parse_inner(meta, &mut owned),
                        meta if is("ref") => parse_inner(meta, &mut ref_),
                        meta if is("ref_mut") => parse_inner(meta, &mut ref_mut),
                        polyfill::Meta::List(list) => parse_list(list, &mut top_level),
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
                if top_level.as_ref().map_or(true, Vec::is_empty) && l.is_empty() =>
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

        Err(syn::Error::new(
            span,
            format!("legacy syntax, use `{format}` instead"),
        ))
    } else {
        Err(syn::Error::new(
            span,
            format!(
                "legacy syntax, remove `types` and use `{}` instead",
                top_level.unwrap_or_else(|| unreachable!()).join(", "),
            ),
        ))
    }
}
