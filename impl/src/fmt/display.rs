//! Implementation of [`fmt::Display`]-like derive macros.

#[cfg(doc)]
use std::fmt;

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{ext::IdentExt as _, parse_quote, spanned::Spanned as _};

use crate::utils::{attr::ParseMultiple as _, Spanning};

use super::{trait_name_to_attribute_name, ContainerAttributes, FmtAttribute};

/// Expands a [`fmt::Display`]-like derive macro.
///
/// Available macros:
/// - [`Binary`](fmt::Binary)
/// - [`Display`](fmt::Display)
/// - [`LowerExp`](fmt::LowerExp)
/// - [`LowerHex`](fmt::LowerHex)
/// - [`Octal`](fmt::Octal)
/// - [`Pointer`](fmt::Pointer)
/// - [`UpperExp`](fmt::UpperExp)
/// - [`UpperHex`](fmt::UpperHex)
pub fn expand(input: &syn::DeriveInput, trait_name: &str) -> syn::Result<TokenStream> {
    let trait_name = normalize_trait_name(trait_name);
    let attr_name = format_ident!("{}", trait_name_to_attribute_name(trait_name));

    let attrs = ContainerAttributes::parse_attrs(&input.attrs, &attr_name)?
        .map(Spanning::into_inner)
        .unwrap_or_default();
    let trait_ident = format_ident!("{trait_name}");
    let ident = &input.ident;

    let type_params: Vec<_> = input
        .generics
        .params
        .iter()
        .filter_map(|p| match p {
            syn::GenericParam::Type(t) => Some(&t.ident),
            syn::GenericParam::Const(..) | syn::GenericParam::Lifetime(..) => None,
        })
        .collect();

    let ctx: ExpansionCtx = (&attrs, &type_params, ident, &trait_ident, &attr_name);
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
        impl #impl_gens derive_more::#trait_ident for #ident #ty_gens #where_clause {
            fn fmt(
                &self, __derive_more_f: &mut derive_more::core::fmt::Formatter<'_>
            ) -> derive_more::core::fmt::Result {
                #body
            }
        }
    })
}

/// Type alias for an expansion context:
/// - [`ContainerAttributes`].
/// - Type parameters. Slice of [`syn::Ident`].
/// - Struct/enum/union [`syn::Ident`].
/// - Derived trait [`syn::Ident`].
/// - Attribute name [`syn::Ident`].
///
/// [`syn::Ident`]: struct@syn::Ident
type ExpansionCtx<'a> = (
    &'a ContainerAttributes,
    &'a [&'a syn::Ident],
    &'a syn::Ident,
    &'a syn::Ident,
    &'a syn::Ident,
);

/// Expands a [`fmt::Display`]-like derive macro for the provided struct.
fn expand_struct(
    s: &syn::DataStruct,
    (attrs, type_params, ident, trait_ident, _): ExpansionCtx<'_>,
) -> syn::Result<(Vec<syn::WherePredicate>, TokenStream)> {
    let s = Expansion {
        shared_attr: None,
        attrs,
        fields: &s.fields,
        type_params,
        trait_ident,
        ident,
    };
    let bounds = s.generate_bounds();
    let body = s.generate_body()?;

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
fn expand_enum(
    e: &syn::DataEnum,
    (container_attrs, type_params, _, trait_ident, attr_name): ExpansionCtx<'_>,
) -> syn::Result<(Vec<syn::WherePredicate>, TokenStream)> {
    if let Some(shared_fmt) = &container_attrs.fmt {
        if shared_fmt
            .placeholders_by_arg("_variant")
            .any(|p| p.has_modifiers || p.trait_name != "Display")
        {
            // TODO: This limitation can be lifted, by analyzing the `shared_fmt` deeper and using
            //       `&dyn fmt::TraitName` for transparency instead of just `format_args!()` in the
            //       expansion.
            return Err(syn::Error::new(
                shared_fmt.span(),
                "shared format `_variant` placeholder cannot contain format specifiers",
            ));
        }
    }

    let (bounds, match_arms) = e.variants.iter().try_fold(
        (Vec::new(), TokenStream::new()),
        |(mut bounds, mut arms), variant| {
            let attrs = ContainerAttributes::parse_attrs(&variant.attrs, attr_name)?
                .map(Spanning::into_inner)
                .unwrap_or_default();
            let ident = &variant.ident;

            if attrs.fmt.is_none()
                && variant.fields.is_empty()
                && attr_name != "display"
            {
                return Err(syn::Error::new(
                    e.variants.span(),
                    format!(
                        "implicit formatting of unit enum variant is supported only for `Display` \
                         macro, use `#[{attr_name}(\"...\")]` to explicitly specify the formatting",
                    ),
                ));
            }

            let v = Expansion {
                shared_attr: container_attrs.fmt.as_ref(),
                attrs: &attrs,
                fields: &variant.fields,
                type_params,
                trait_ident,
                ident,
            };
            let arm_body = v.generate_body()?;
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

            Ok::<_, syn::Error>((bounds, arms))
        },
    )?;

    let body = match_arms
        .is_empty()
        .then(|| quote! { match *self {} })
        .unwrap_or_else(|| quote! { match self { #match_arms } });

    Ok((bounds, body))
}

/// Expands a [`fmt::Display`]-like derive macro for the provided union.
fn expand_union(
    u: &syn::DataUnion,
    (attrs, _, _, _, attr_name): ExpansionCtx<'_>,
) -> syn::Result<(Vec<syn::WherePredicate>, TokenStream)> {
    let fmt = &attrs.fmt.as_ref().ok_or_else(|| {
        syn::Error::new(
            u.fields.span(),
            format!("unions must have `#[{attr_name}(\"...\", ...)]` attribute"),
        )
    })?;

    Ok((
        attrs.bounds.0.clone().into_iter().collect(),
        quote! { derive_more::core::write!(__derive_more_f, #fmt) },
    ))
}

/// Helper struct to generate [`Display::fmt()`] implementation body and trait
/// bounds for a struct or an enum variant.
///
/// [`Display::fmt()`]: fmt::Display::fmt()
#[derive(Debug)]
struct Expansion<'a> {
    /// [`FmtAttribute`] shared between all variants of an enum.
    ///
    /// [`None`] for a struct.
    shared_attr: Option<&'a FmtAttribute>,

    /// Derive macro [`ContainerAttributes`].
    attrs: &'a ContainerAttributes,

    /// Struct or enum [`syn::Ident`].
    ///
    /// [`syn::Ident`]: struct@syn::Ident
    ident: &'a syn::Ident,

    /// Struct or enum [`syn::Fields`].
    fields: &'a syn::Fields,

    /// Type parameters in this struct or enum.
    type_params: &'a [&'a syn::Ident],

    /// [`fmt`] trait [`syn::Ident`].
    ///
    /// [`syn::Ident`]: struct@syn::Ident
    trait_ident: &'a syn::Ident,
}

impl<'a> Expansion<'a> {
    /// Generates [`Display::fmt()`] implementation for a struct or an enum variant.
    ///
    /// # Errors
    ///
    /// In case [`FmtAttribute`] is [`None`] and [`syn::Fields`] length is
    /// greater than 1.
    ///
    /// [`Display::fmt()`]: fmt::Display::fmt()
    fn generate_body(&self) -> syn::Result<TokenStream> {
        let mut body = TokenStream::new();

        // If `shared_attr` is a transparent call, then we consider it being absent.
        let has_shared_attr = self
            .shared_attr
            .map_or(false, |a| a.transparent_call().is_none());

        if !has_shared_attr
            || self
                .shared_attr
                .map_or(true, |a| a.contains_arg("_variant"))
        {
            body = match &self.attrs.fmt {
                Some(fmt) => {
                    if has_shared_attr {
                        quote! { &derive_more::core::format_args!(#fmt) }
                    } else if let Some((expr, trait_ident)) = fmt.transparent_call() {
                        quote! {
                            derive_more::core::fmt::#trait_ident::fmt(&(#expr), __derive_more_f)
                        }
                    } else {
                        quote! { derive_more::core::write!(__derive_more_f, #fmt) }
                    }
                }
                None if self.fields.is_empty() => {
                    let ident_str = self.ident.unraw().to_string();

                    if has_shared_attr {
                        quote! { #ident_str }
                    } else {
                        quote! { __derive_more_f.write_str(#ident_str) }
                    }
                }
                None if self.fields.len() == 1 => {
                    let field = self
                        .fields
                        .iter()
                        .next()
                        .unwrap_or_else(|| unreachable!("count() == 1"));
                    let ident =
                        field.ident.clone().unwrap_or_else(|| format_ident!("_0"));
                    let trait_ident = self.trait_ident;

                    if has_shared_attr {
                        let placeholder =
                            trait_name_to_default_placeholder_literal(trait_ident);

                        quote! { &derive_more::core::format_args!(#placeholder, #ident) }
                    } else {
                        quote! {
                            derive_more::core::fmt::#trait_ident::fmt(#ident, __derive_more_f)
                        }
                    }
                }
                _ => {
                    return Err(syn::Error::new(
                        self.fields.span(),
                        format!(
                            "struct or enum variant with more than 1 field must have \
                     `#[{}(\"...\", ...)]` attribute",
                            trait_name_to_attribute_name(self.trait_ident),
                        ),
                    ))
                }
            };
        }

        if has_shared_attr {
            if let Some(shared_fmt) = &self.shared_attr {
                let shared_body = quote! {
                    derive_more::core::write!(__derive_more_f, #shared_fmt)
                };

                body = if body.is_empty() {
                    shared_body
                } else {
                    quote! { match #body { _variant => #shared_body } }
                }
            }
        }

        Ok(body)
    }

    /// Generates trait bounds for a struct or an enum variant.
    fn generate_bounds(&self) -> Vec<syn::WherePredicate> {
        let mut bounds = vec![];

        if self
            .shared_attr
            .map_or(true, |a| a.contains_arg("_variant"))
        {
            if let Some(fmt) = &self.attrs.fmt {
                bounds.extend(
                    fmt.bounded_types(self.fields)
                        .filter_map(|(ty, trait_name)| {
                            if !self.contains_generic_param(ty) {
                                return None;
                            }
                            let trait_ident = format_ident!("{trait_name}");

                            Some(parse_quote! { #ty: derive_more::core::fmt::#trait_ident })
                        })
                        .chain(self.attrs.bounds.0.clone()),
                );
            } else {
                bounds.extend(
                self.fields
                    .iter()
                    .next()
                    .map(|f| {
                        let ty = &f.ty;
                        if !self.contains_generic_param(ty) {
                            return vec![];
                        }
                        let trait_ident = &self.trait_ident;
                        vec![parse_quote! { #ty: derive_more::core::fmt::#trait_ident }]
                    })
                    .unwrap_or_default(),
                );
            };
        }

        if let Some(shared_fmt) = &self.shared_attr {
            bounds.extend(shared_fmt.bounded_types(self.fields).filter_map(
                |(ty, trait_name)| {
                    if !self.contains_generic_param(ty) {
                        return None;
                    }
                    let trait_ident = format_ident!("{trait_name}");

                    Some(parse_quote! { #ty: derive_more::core::fmt::#trait_ident })
                },
            ));
        }

        bounds
    }

    /// Checks whether the provided [`syn::Path`] contains any of these [`Expansion::type_params`].
    fn path_contains_generic_param(&self, path: &syn::Path) -> bool {
        path.segments
            .iter()
            .any(|segment| match &segment.arguments {
                syn::PathArguments::None => false,
                syn::PathArguments::AngleBracketed(
                    syn::AngleBracketedGenericArguments { args, .. },
                ) => args.iter().any(|generic| match generic {
                    syn::GenericArgument::Type(ty)
                    | syn::GenericArgument::AssocType(syn::AssocType { ty, .. }) => {
                        self.contains_generic_param(ty)
                    }

                    syn::GenericArgument::Lifetime(_)
                    | syn::GenericArgument::Const(_)
                    | syn::GenericArgument::AssocConst(_)
                    | syn::GenericArgument::Constraint(_) => false,
                    _ => unimplemented!(
                        "syntax is not supported by `derive_more`, please report a bug",
                    ),
                }),
                syn::PathArguments::Parenthesized(
                    syn::ParenthesizedGenericArguments { inputs, output, .. },
                ) => {
                    inputs.iter().any(|ty| self.contains_generic_param(ty))
                        || match output {
                            syn::ReturnType::Default => false,
                            syn::ReturnType::Type(_, ty) => {
                                self.contains_generic_param(ty)
                            }
                        }
                }
            })
    }

    /// Checks whether the provided [`syn::Type`] contains any of these [`Expansion::type_params`].
    fn contains_generic_param(&self, ty: &syn::Type) -> bool {
        if self.type_params.is_empty() {
            return false;
        }
        match ty {
            syn::Type::Path(syn::TypePath { qself, path }) => {
                if let Some(qself) = qself {
                    if self.contains_generic_param(&qself.ty) {
                        return true;
                    }
                }

                if let Some(ident) = path.get_ident() {
                    self.type_params.iter().any(|param| *param == ident)
                } else {
                    self.path_contains_generic_param(path)
                }
            }

            syn::Type::Array(syn::TypeArray { elem, .. })
            | syn::Type::Group(syn::TypeGroup { elem, .. })
            | syn::Type::Paren(syn::TypeParen { elem, .. })
            | syn::Type::Ptr(syn::TypePtr { elem, .. })
            | syn::Type::Reference(syn::TypeReference { elem, .. })
            | syn::Type::Slice(syn::TypeSlice { elem, .. }) => {
                self.contains_generic_param(elem)
            }

            syn::Type::BareFn(syn::TypeBareFn { inputs, output, .. }) => {
                inputs
                    .iter()
                    .any(|arg| self.contains_generic_param(&arg.ty))
                    || match output {
                        syn::ReturnType::Default => false,
                        syn::ReturnType::Type(_, ty) => self.contains_generic_param(ty),
                    }
            }
            syn::Type::Tuple(syn::TypeTuple { elems, .. }) => {
                elems.iter().any(|ty| self.contains_generic_param(ty))
            }

            syn::Type::ImplTrait(_) => false,
            syn::Type::Infer(_) => false,
            syn::Type::Macro(_) => false,
            syn::Type::Never(_) => false,
            syn::Type::TraitObject(syn::TypeTraitObject { bounds, .. }) => {
                bounds.iter().any(|bound| match bound {
                    syn::TypeParamBound::Trait(syn::TraitBound { path, .. }) => {
                        self.path_contains_generic_param(path)
                    }
                    syn::TypeParamBound::Lifetime(_) => false,
                    syn::TypeParamBound::Verbatim(_) => false,
                    _ => unimplemented!(
                        "syntax is not supported by `derive_more`, please report a bug",
                    ),
                })
            }
            syn::Type::Verbatim(_) => false,
            _ => unimplemented!(
                "syntax is not supported by `derive_more`, please report a bug",
            ),
        }
    }
}

/// Matches the provided derive macro `name` to appropriate actual trait name.
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

/// Matches the provided [`fmt`] trait `name` to its default formatting placeholder.
fn trait_name_to_default_placeholder_literal(name: &syn::Ident) -> &'static str {
    match () {
        _ if name == "Binary" => "{:b}",
        _ if name == "Debug" => "{:?}",
        _ if name == "Display" => "{}",
        _ if name == "LowerExp" => "{:e}",
        _ if name == "LowerHex" => "{:x}",
        _ if name == "Octal" => "{:o}",
        _ if name == "Pointer" => "{:p}",
        _ if name == "UpperExp" => "{:E}",
        _ if name == "UpperHex" => "{:X}",
        _ => unimplemented!(),
    }
}
