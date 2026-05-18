use crate::utils::attr::{self, ParseMultiple};
use proc_macro2::TokenStream;
use quote::quote;
use syn::DeriveInput;

pub fn expand(input: &DeriveInput, _: &'static str) -> syn::Result<TokenStream> {
    let name = &input.ident;
    let ident = quote::format_ident!("copy");

    let mut generics = input.generics.clone();
    if let Some(bounds) = attr::Bounds::parse_attrs(&input.attrs, &ident)? {
        generics
            .make_where_clause()
            .predicates
            .extend(bounds.item.0);
    }
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    Ok(quote! {
        impl #impl_generics derive_more::core::marker::Copy for #name #ty_generics #where_clause {

        }
    })
}
