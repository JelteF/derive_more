use crate::utils::{State, SingleFieldData};
use proc_macro2::TokenStream;
use quote::{quote};
use syn::{parse::Result, DeriveInput};

/// Provides the hook to expand `#[derive(Index)]` into an implementation of `From`
pub fn expand(input: &DeriveInput, trait_name: &'static str) -> Result<TokenStream> {
    let state = State::new(
        input,
        trait_name,
        quote!(::core::iter),
        trait_name.to_lowercase(),
    )?;
    let SingleFieldData {
        input_type,
        member,
        trait_path,
        casted_trait,
        impl_generics,
        ty_generics,
        where_clause,
        ..
    } = state.assert_single_enabled_field();

    Ok(quote! {
        impl#impl_generics #trait_path for #input_type#ty_generics #where_clause
        {
            type Item = #casted_trait::Item;
            #[inline]
            fn next(&mut self) -> Option<Self::Item> {
                #casted_trait::next(&mut #member)
            }
        }
    })
}
