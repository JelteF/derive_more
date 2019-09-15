use proc_macro2::TokenStream;
use quote::quote;
use syn::DeriveInput;
use crate::utils;


pub fn expand(input: &DeriveInput, _: &str) -> TokenStream {

    let input_type = &input.ident;
    let (impl_generics, input_generics, where_clause) = input.generics.split_for_impl();
    let (field_type, field_ident) = utils::extract_field_info(&input.data, "as_ref");

    quote! {#(
        impl#impl_generics ::core::convert::AsRef<#field_type> for #input_type#input_generics
        #where_clause
        {
            #[inline]
            fn as_ref(&self) -> &#field_type {
                &self.#field_ident
            }
        }
    )*}
}
