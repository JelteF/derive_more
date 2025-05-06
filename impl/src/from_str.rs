//! Implementation of a [`FromStr`] derive macro.

#[cfg(doc)]
use std::str::FromStr;

use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::spanned::Spanned as _;

/// Expands a [`FromStr`] derive macro.
pub fn expand(input: &syn::DeriveInput, _: &'static str) -> syn::Result<TokenStream> {
    match &input.data {
        syn::Data::Struct(_) => {
            Ok(ForwardExpansion::try_from(input)?.into_token_stream())
        }
        syn::Data::Enum(_data) => todo!(),
        syn::Data::Union(data) => Err(syn::Error::new(
            data.union_token.span(),
            "`FromStr` cannot be derived for unions",
        )),
    }
}

/// Expansion of a macro for generating a forwarding [`FromStr`] implementation of a struct.
struct ForwardExpansion<'i> {
    /// [`syn::Ident`] of the struct.
    ///
    /// [`syn::Ident`]: struct@syn::Ident
    ident: &'i syn::Ident,

    /// [`syn::Generics`] of the struct.
    generics: &'i syn::Generics,

    /// [`syn::Field`] of the value wrapped by the struct to forward implementation on.
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
            inner,
            ident: &input.ident,
            generics: &input.generics,
        })
    }
}

impl ToTokens for ForwardExpansion<'_> {
    /// Expands [`TryFrom`] implementations for a struct.
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let ty = self.ident;
        let (impl_generics, ty_generics, where_clause) = self.generics.split_for_impl();
        
        let inner_ty = &self.inner.ty;
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
                    derive_more::core::str::FromStr::from_str(s).map(|v| #constructor);
                }
            }
        }.to_tokens(tokens);
    }
}

