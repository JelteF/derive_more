use crate::utils::{add_extra_ty_param_bound, State};
use proc_macro2::TokenStream;
use quote::{quote};
use syn::{parse::Result, DeriveInput};

/// Provides the hook to expand `#[derive(Index)]` into an implementation of `From`
pub fn expand(input: &DeriveInput, trait_name: &'static str) -> Result<TokenStream> {
    let input_type = &input.ident;
    let state = State::new(
        input,
        trait_name,
        quote!(::core::iter),
        trait_name.to_lowercase(),
    )?;
    let (_, field_type, field_ident) = state.assert_single_enabled_field();
    let trait_path = &state.trait_path;

    let generics = add_extra_ty_param_bound(&input.generics, trait_path);
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let casted_trait = &quote!(<#field_type as #trait_path>);
    Ok(quote! {
        impl#impl_generics #trait_path for #input_type#ty_generics #where_clause
        {
            type Item = #casted_trait::Item;
            #[inline]
            fn next(&mut self) -> Option<Self::Item> {
                #casted_trait::next(&mut self.#field_ident)
            }
        }
    })
}
