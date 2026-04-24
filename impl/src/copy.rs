use crate::utils::attr::{self, ParseMultiple};
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::DeriveInput;

pub fn expand(input: &DeriveInput, _: &'static str) -> syn::Result<TokenStream> {
    let name = &input.ident;
    let generics = &input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let ident = quote::format_ident!("copy");
    let real_where_clause = attr::Bounds::parse_attrs(&input.attrs, &ident)?
        .map(|clause| {
            let clause = clause.item.0;
            quote! { where #clause }
        })
        .or(where_clause.map(ToTokens::to_token_stream));

    Ok(quote! {
        impl #impl_generics derive_more::core::marker::Copy for #name #ty_generics #real_where_clause {

        }
    })
}
