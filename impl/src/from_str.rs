//! Implementation of a [`FromStr`] derive macro.

use std::collections::HashMap;
#[cfg(doc)]
use std::str::FromStr;

use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse_quote, spanned::Spanned as _};

/// Expands a [`FromStr`] derive macro.
pub fn expand(input: &syn::DeriveInput, _: &'static str) -> syn::Result<TokenStream> {
    match &input.data {
        syn::Data::Struct(_) => {
            Ok(ForwardExpansion::try_from(input)?.into_token_stream())
        }
        syn::Data::Enum(_) => {
            Ok(EnumFlatExpansion::try_from(input)?.into_token_stream())
        }
        syn::Data::Union(data) => Err(syn::Error::new(
            data.union_token.span(),
            "`FromStr` cannot be derived for unions",
        )),
    }
}

/// Expansion of a macro for generating a forwarding [`FromStr`] implementation of a struct.
struct ForwardExpansion<'i> {
    /// [`syn::Ident`] and [`syn::Generics`] of the struct.
    ///
    /// [`syn::Ident`]: struct@syn::Ident
    self_ty: (&'i syn::Ident, &'i syn::Generics),

    /// [`syn::Field`] representing the wrapped type to forward implementation on.
    inner: &'i syn::Field,
}

impl<'i> TryFrom<&'i syn::DeriveInput> for ForwardExpansion<'i> {
    type Error = syn::Error;

    fn try_from(input: &'i syn::DeriveInput) -> syn::Result<Self> {
        let syn::Data::Struct(data) = &input.data else {
            return Err(syn::Error::new(
                input.span(),
                "expected a struct for forward `FromStr` derive",
            ));
        };

        // TODO: Unite these two conditions via `&&` once MSRV is bumped to 1.88 or above.
        if data.fields.len() != 1 {
            return Err(syn::Error::new(
                data.fields.span(),
                "only structs with single field can derive `FromStr`",
            ));
        }
        let Some(inner) = data.fields.iter().next() else {
            return Err(syn::Error::new(
                data.fields.span(),
                "only structs with single field can derive `FromStr`",
            ));
        };

        Ok(Self {
            self_ty: (&input.ident, &input.generics),
            inner,
        })
    }
}

impl ToTokens for ForwardExpansion<'_> {
    /// Expands a forwarding [`FromStr`] implementations for a struct.
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let inner_ty = &self.inner.ty;
        let ty = self.self_ty.0;

        let mut generics = self.self_ty.1.clone();
        if !generics.params.is_empty() {
            generics.make_where_clause().predicates.push(parse_quote! {
                #inner_ty: derive_more::core::str::FromStr
            });
        }
        let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

        let constructor = if let Some(name) = &self.inner.ident {
            quote! { Self { #name: v } }
        } else {
            quote! { Self(v) }
        };

        quote! {
            #[automatically_derived]
            impl #impl_generics derive_more::core::str::FromStr for #ty #ty_generics #where_clause {
                type Err = <#inner_ty as derive_more::core::str::FromStr>::Err;

                #[inline]
                fn from_str(s: &str) -> derive_more::core::result::Result<Self, Self::Err> {
                    derive_more::core::str::FromStr::from_str(s).map(|v| #constructor)
                }
            }
        }.to_tokens(tokens);
    }
}

/// Expansion of a macro for generating a flat [`FromStr`] implementation of an enum.
struct EnumFlatExpansion<'i> {
    /// [`syn::Ident`] and [`syn::Generics`] of the enum.
    ///
    /// [`syn::Ident`]: struct@syn::Ident
    self_ty: (&'i syn::Ident, &'i syn::Generics),

    /// [`syn::Ident`]s of the enum variants.
    ///
    /// [`syn::Ident`]: struct@syn::Ident
    variants: Vec<&'i syn::Ident>,
}

impl<'i> TryFrom<&'i syn::DeriveInput> for EnumFlatExpansion<'i> {
    type Error = syn::Error;

    fn try_from(input: &'i syn::DeriveInput) -> syn::Result<Self> {
        let syn::Data::Enum(data) = &input.data else {
            return Err(syn::Error::new(
                input.span(),
                "expected an enum for flat `FromStr` derive",
            ));
        };

        let variants = data
            .variants
            .iter()
            .map(|variant| {
                if !variant.fields.is_empty() {
                    return Err(syn::Error::new(
                        variant.fields.span(),
                        "only enums with no fields can derive `FromStr`",
                    ));
                }
                Ok(&variant.ident)
            })
            .collect::<syn::Result<_>>()?;

        Ok(Self {
            self_ty: (&input.ident, &input.generics),
            variants,
        })
    }
}

impl ToTokens for EnumFlatExpansion<'_> {
    /// Expands a flat [`FromStr`] implementations for an enum.
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let ty = self.self_ty.0;
        let (impl_generics, ty_generics, where_clause) =
            self.self_ty.1.split_for_impl();
        let ty_name = ty.to_string();

        let similar_lowercased = self
            .variants
            .iter()
            .map(|v| v.to_string().to_lowercase())
            .fold(<HashMap<_, u8>>::new(), |mut counts, v| {
                *counts.entry(v).or_default() += 1;
                counts
            });

        let match_arms = self.variants.iter().map(|variant| {
            let name = variant.to_string();
            let lowercased = name.to_lowercase();
            let exact_guard =
                (similar_lowercased[&lowercased] > 1).then(|| quote! { if s == #name });

            quote! { #lowercased #exact_guard => Self::#variant, }
        });

        quote! {
            #[automatically_derived]
            impl #impl_generics derive_more::core::str::FromStr for #ty #ty_generics #where_clause {
                type Err = derive_more::FromStrError;

                fn from_str(
                    s: &str,
                ) -> derive_more::core::result::Result<Self, derive_more::FromStrError> {
                    derive_more::core::result::Result::Ok(match s.to_lowercase().as_str() {
                        #( #match_arms )*
                        _ => return derive_more::core::result::Result::Err(
                            derive_more::FromStrError::new(#ty_name),
                        ),
                    })
                }
            }
        }.to_tokens(tokens);
    }
}
