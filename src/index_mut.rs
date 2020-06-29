use crate::utils::{
    add_extra_generic_param, add_extra_where_clauses, DeriveType, MultiVariantData,
    SingleFieldData, State,
};
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{parse::Result, DeriveInput, Generics, Ident};

/// Provides the hook to expand `#[derive(IndexMut)]` into an implementation of `IndexMut`
pub fn expand(input: &DeriveInput, trait_name: &'static str) -> Result<TokenStream> {
    let mut state = State::with_field_ignore(
        input,
        trait_name,
        quote!(::core::ops),
        String::from("index_mut"),
    )?;
    let index_type = &Ident::new("__IdxT", Span::call_site());
    state.add_trait_path_type_param(quote!(#index_type));
    let output_type = &Ident::new("__IdxOutputT", Span::call_site());
    let mut new_generics =
        add_extra_generic_param(&input.generics, quote!(#index_type));
    new_generics = add_extra_generic_param(
        &new_generics,
        quote!(#output_type: ?::core::marker::Sized),
    );
    if state.derive_type == DeriveType::Enum {
        Ok(expand_enum(
            input,
            state,
            new_generics,
            index_type,
            output_type,
        ))
    } else {
        Ok(expand_struct(
            input,
            state,
            new_generics,
            index_type,
            output_type,
        ))
    }
}

fn expand_body(
    single_field_data: &SingleFieldData,
    mut new_generics: Generics,
    index_type: &Ident,
    output_type: &Ident,
) -> (TokenStream, Generics) {
    let SingleFieldData {
        casted_trait,
        field_type,
        trait_path,
        ..
    } = single_field_data;

    let type_where_clauses = quote! {
        where #field_type: #trait_path<#index_type, Output=#output_type>
    };

    let body = quote!(#casted_trait::index_mut(indexable, idx));
    new_generics = add_extra_where_clauses(&new_generics, type_where_clauses);
    (body, new_generics)
}

fn expand_struct(
    input: &DeriveInput,
    state: State,
    new_generics: Generics,
    index_type: &Ident,
    output_type: &Ident,
) -> TokenStream {
    let single_field_data = state.assert_single_enabled_field();

    let (body, new_generics) =
        expand_body(&single_field_data, new_generics, index_type, output_type);
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
            fn index_mut(&mut self, idx: #index_type) -> &mut Self::Output {
                let indexable = &mut #member;
                #body
            }
        }
    }
}

fn expand_enum(
    input: &DeriveInput,
    state: State,
    mut new_generics: Generics,
    index_type: &Ident,
    output_type: &Ident,
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
        let indexable = &quote!(indexable);
        patterns.push(single_field_data.matcher(&[0], &[indexable]));
        let (body, temp_new_generics) =
            expand_body(&single_field_data, new_generics, index_type, output_type);
        new_generics = temp_new_generics;
        bodies.push(body);
    }

    let (impl_generics, _, where_clause) = new_generics.split_for_impl();
    let (_, ty_generics, _) = input.generics.split_for_impl();

    quote! {
        impl#impl_generics #trait_path_with_params for #input_type#ty_generics #where_clause
        {
            #[inline]
            fn index_mut(&mut self, idx: #index_type) -> &mut Self::Output {
                match self { #(#patterns => #bodies),* }
            }
        }
    }
}
