use crate::utils::{MultiFieldData, State};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse::Result, DeriveInput};

pub fn expand(input: &DeriveInput, trait_name: &'static str) -> Result<TokenStream> {
    let state = State::new(
        input,
        trait_name,
        quote!(::core::convert),
        String::from("as_mut"),
    )?;
    let MultiFieldData {
        input_type,
        field_types,
        members,
        trait_path,
        impl_generics,
        ty_generics,
        where_clause,
        ..
    } = state.enabled_fields_data();

    Ok(quote! {#(
        impl#impl_generics #trait_path<#field_types> for #input_type#ty_generics
        #where_clause
        {
            #[inline]
            fn as_mut(&mut self) -> &mut #field_types {
                &mut #members
            }
        }
    )*})
}
