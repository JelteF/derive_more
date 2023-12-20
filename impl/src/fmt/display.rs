//! Implementation of [`fmt::Display`]-like derive macros.

#[cfg(doc)]
use std::fmt;

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_quote, spanned::Spanned as _};

use crate::utils::{attr::ParseMultiple as _, Spanning};

use super::{trait_name_to_attribute_name, ContainerAttributes};

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

    let ctx = (&attrs, ident, &trait_ident, &attr_name);
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
/// - [`ContainerAttributes`].
/// - Struct/enum/union [`syn::Ident`].
/// - Derived trait [`syn::Ident`].
/// - Attribute name [`syn::Ident`].
type ExpansionCtx<'a> = (
    &'a ContainerAttributes,
    &'a syn::Ident,
    &'a syn::Ident,
    &'a syn::Ident,
);

/// Expands a [`fmt::Display`]-like derive macro for the provided struct.
fn expand_struct(
    s: &syn::DataStruct,
    (attrs, ident, trait_ident, _): ExpansionCtx<'_>,
) -> syn::Result<(Vec<syn::WherePredicate>, TokenStream)> {
    let s = Expansion {
        attrs,
        fields: &s.fields,
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
    (attrs, _, trait_ident, attr_name): ExpansionCtx<'_>,
) -> syn::Result<(Vec<syn::WherePredicate>, TokenStream)> {
    if attrs.fmt.is_some() {
        todo!("https://github.com/JelteF/derive_more/issues/142");
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
                attrs: &attrs,
                fields: &variant.fields,
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
    (attrs, _, _, attr_name): ExpansionCtx<'_>,
) -> syn::Result<(Vec<syn::WherePredicate>, TokenStream)> {
    let fmt = &attrs.fmt.as_ref().ok_or_else(|| {
        syn::Error::new(
            u.fields.span(),
            format!("unions must have `#[{attr_name}(\"...\", ...)]` attribute"),
        )
    })?;

    Ok((
        attrs.bounds.0.clone().into_iter().collect(),
        quote! { ::core::write!(__derive_more_f, #fmt) },
    ))
}

/// Helper struct to generate [`Display::fmt()`] implementation body and trait
/// bounds for a struct or an enum variant.
///
/// [`Display::fmt()`]: fmt::Display::fmt()
#[derive(Debug)]
struct Expansion<'a> {
    /// Derive macro [`ContainerAttributes`].
    attrs: &'a ContainerAttributes,

    /// Struct or enum [`syn::Ident`].
    ident: &'a syn::Ident,

    /// Struct or enum [`syn::Fields`].
    fields: &'a syn::Fields,

    /// [`fmt`] trait [`syn::Ident`].
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
        match &self.attrs.fmt {
            Some(fmt) => {
                Ok(if let Some((expr, trait_ident)) = fmt.transparent_call() {
                    quote! { ::core::fmt::#trait_ident::fmt(&(#expr), __derive_more_f) }
                } else {
                    quote! { ::core::write!(__derive_more_f, #fmt) }
                })
            }
            None if self.fields.is_empty() => {
                let ident_str = self.ident.to_string();

                Ok(quote! {
                    ::core::write!(__derive_more_f, #ident_str)
                })
            }
            None if self.fields.len() == 1 => {
                let field = self
                    .fields
                    .iter()
                    .next()
                    .unwrap_or_else(|| unreachable!("count() == 1"));
                let ident = field.ident.clone().unwrap_or_else(|| format_ident!("_0"));
                let trait_ident = self.trait_ident;

                Ok(quote! {
                    ::core::fmt::#trait_ident::fmt(#ident, __derive_more_f)
                })
            }
            _ => Err(syn::Error::new(
                self.fields.span(),
                format!(
                    "struct or enum variant with more than 1 field must have \
                     `#[{}(\"...\", ...)]` attribute",
                    trait_name_to_attribute_name(self.trait_ident),
                ),
            )),
        }
    }

    /// Generates trait bounds for a struct or an enum variant.
    fn generate_bounds(&self) -> Vec<syn::WherePredicate> {
        let Some(fmt) = &self.attrs.fmt else {
            return self
                .fields
                .iter()
                .next()
                .map(|f| {
                    let ty = &f.ty;
                    let trait_ident = &self.trait_ident;
                    vec![parse_quote! { #ty: ::core::fmt::#trait_ident }]
                })
                .unwrap_or_default();
        };

        fmt.bounded_types(self.fields)
            .map(|(ty, trait_name)| {
                let trait_ident = format_ident!("{trait_name}");

                parse_quote! { #ty: ::core::fmt::#trait_ident }
            })
            .chain(self.attrs.bounds.0.clone())
            .collect()
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
