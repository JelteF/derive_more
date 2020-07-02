use crate::utils::{
    add_extra_where_clauses, DeriveType, MultiVariantData, SingleFieldData, State,
};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse::Result, DeriveInput, Generics};

/// Provides the hook to expand `#[derive(Read)]` into an implementation of `Read`
pub fn expand(input: &DeriveInput, trait_name: &'static str) -> Result<TokenStream> {
    let state = State::with_field_ignore(
        input,
        trait_name,
        quote!(::std::io),
        trait_name.to_lowercase(),
    )?;
    let new_generics = input.generics.clone();

    if state.derive_type == DeriveType::Enum {
        Ok(expand_enum(input, state, new_generics))
    } else {
        Ok(expand_struct(input, state, new_generics))
    }
}

fn expand_body(
    single_field_data: &SingleFieldData,
    mut new_generics: Generics,
) -> (TokenStream, Generics) {
    let SingleFieldData {
        casted_trait,
        field_type,
        trait_path,
        ..
    } = single_field_data;

    let type_where_clauses = quote! {
        where #field_type: #trait_path
    };

    let body = quote!(#casted_trait::read(readable, buf));
    new_generics = add_extra_where_clauses(&new_generics, type_where_clauses);
    (body, new_generics)
}

fn expand_struct(
    input: &DeriveInput,
    state: State,
    new_generics: Generics,
) -> TokenStream {
    let single_field_data = state.assert_single_enabled_field();

    let (body, new_generics) = expand_body(&single_field_data, new_generics);
    let SingleFieldData {
        input_type,
        trait_path_with_params,
        member,
        ..
    } = single_field_data;

    let (impl_generics, _, where_clause) = new_generics.split_for_impl();
    let (_, ty_generics, _) = input.generics.split_for_impl();
    quote! {
        impl#impl_generics #trait_path_with_params for #input_type#ty_generics #where_clause
        {
            #[inline]
            fn read(&mut self, buf: &mut [u8]) -> ::std::io::Result<usize> {
                let readable = &mut #member;
                #body
            }
        }
    }
}

fn expand_enum(
    input: &DeriveInput,
    state: State,
    mut new_generics: Generics,
) -> TokenStream {
    let MultiVariantData {
        input_type,
        variant_states,
        trait_path_with_params,
        ..
    } = state.enabled_variant_data();

    let mut bodies = vec![];
    let mut patterns = vec![];
    let mut first_single_field_data = None;
    for variant_state in variant_states {
        let single_field_data = variant_state.assert_single_enabled_field();
        if first_single_field_data.is_none() {
            first_single_field_data = Some(single_field_data.clone());
        }
        let readable = &quote!(readable);
        patterns.push(single_field_data.matcher(&[0], &[readable]));
        let (body, temp_new_generics) = expand_body(&single_field_data, new_generics);
        new_generics = temp_new_generics;
        bodies.push(body);
    }

    let (impl_generics, _, where_clause) = new_generics.split_for_impl();
    let (_, ty_generics, _) = input.generics.split_for_impl();

    quote! {
        impl#impl_generics #trait_path_with_params for #input_type#ty_generics #where_clause
        {
            #[inline]
            fn read(&mut self, buf: &mut [u8]) -> ::std::io::Result<usize> {
                match self { #(#patterns => #bodies),* }
            }
        }
    }
}
