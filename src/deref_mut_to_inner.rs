use crate::utils::{SingleFieldData, State};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse::Result, DeriveInput};

/// Provides the hook to expand `#[derive(Index)]` into an implementation of `From`
pub fn expand(input: &DeriveInput, trait_name: &'static str) -> Result<TokenStream> {
    let state = State::new(
        input,
        trait_name,
        quote!(::core::ops),
        String::from("deref_mut_to_inner"),
    )?;
    let SingleFieldData {
        input_type,
        member,
        trait_path,
        impl_generics,
        ty_generics,
        where_clause,
        ..
    } = state.assert_single_enabled_field();

    Ok(quote! {
        impl#impl_generics #trait_path for #input_type#ty_generics #where_clause
        {
            #[inline]
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut #member
            }
        }
    })
}
