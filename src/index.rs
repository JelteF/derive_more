use crate::utils::{
    add_extra_generic_param, add_extra_where_clauses, add_where_clauses_for_new_ident,
    DeriveType, MultiFieldData, MultiVariantData, SingleFieldData, State,
};
use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{parse::Result, DeriveInput, Ident};

/// Provides the hook to expand `#[derive(Index)]` into an implementation of `Index`
pub fn expand(input: &DeriveInput, trait_name: &'static str) -> Result<TokenStream> {
    let mut state = State::with_field_ignore(
        input,
        trait_name,
        quote!(::core::ops),
        trait_name.to_lowercase(),
    )?;
    if state.derive_type == DeriveType::Enum {
        Ok(enum_index(input, state))
    } else {
        Ok(struct_index(input, &mut state))
    }
}

fn struct_index(input: &DeriveInput, state: &mut State) -> TokenStream {
    let index_type = &Ident::new("__IdxT", Span::call_site());
    state.add_trait_path_type_param(quote!(#index_type));
    let SingleFieldData {
        field,
        field_type,
        input_type,
        trait_path_with_params,
        casted_trait,
        member,
        ..
    } = state.assert_single_enabled_field();

    let type_where_clauses = quote! {
        where #field_type: #trait_path_with_params
    };

    let new_generics = add_where_clauses_for_new_ident(
        &input.generics,
        &[field],
        index_type,
        type_where_clauses,
        true,
    );

    let (impl_generics, _, where_clause) = new_generics.split_for_impl();
    let (_, ty_generics, _) = input.generics.split_for_impl();
    quote! {
        impl#impl_generics #trait_path_with_params for #input_type#ty_generics #where_clause
        {
            type Output = #casted_trait::Output;
            #[inline]
            fn index(&self, idx: #index_type) -> &Self::Output {
                #casted_trait::index(&#member, idx)
            }
        }
    }
}

fn enum_index(input: &DeriveInput, mut state: State) -> TokenStream {
    let index_type = &Ident::new("__IdxT", Span::call_site());
    let output_type = &Ident::new("__IdxOutputT", Span::call_site());
    let mut new_generics =
        add_extra_generic_param(&input.generics, quote!(#index_type));
    new_generics = add_extra_generic_param(&new_generics, quote!(#output_type));
    state.add_trait_path_type_param(quote!(#index_type));
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
        let matched = &quote!(matched);
        patterns.push(single_field_data.matcher(&[0], &[matched]));

        let SingleFieldData {
            casted_trait,
            field_type,
            trait_path,
            ..
        } = single_field_data;

        let type_where_clauses = quote! {
            where #field_type: #trait_path<#index_type, Output=#output_type>
        };

        bodies.push(quote!(#casted_trait::index(#matched, idx)));
        new_generics = add_extra_where_clauses(&new_generics, type_where_clauses);
    }

    let (impl_generics, _, where_clause) = new_generics.split_for_impl();
    let (_, ty_generics, _) = input.generics.split_for_impl();

    quote! {
        impl#impl_generics #trait_path_with_params for #input_type#ty_generics #where_clause
        {
            type Output = #output_type;
            #[inline]
            fn index(&self, idx: #index_type) -> &Self::Output {
                match self { #(#patterns => #bodies),* }
            }
        }
    }
}
