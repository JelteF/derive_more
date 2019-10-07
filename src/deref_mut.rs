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
        String::from("deref_mut"),
    )?;
    let SingleFieldData {
        input_type,
        trait_path,
        casted_trait,
        impl_generics,
        ty_generics,
        where_clause,
        member,
        info,
        ..
    } = state.assert_single_enabled_field();
    let body = if info.forward {
        quote!(&mut #member)
    } else {
        quote!(#casted_trait::deref_mut(&mut #member))
    };

    Ok(quote! {
        impl#impl_generics #trait_path for #input_type#ty_generics #where_clause
        {
            #[inline]
            fn deref_mut(&mut self) -> &mut Self::Target {
                #body
            }
        }
    })
}
