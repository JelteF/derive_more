use crate::utils::{
    add_extra_generic_param, field_idents, get_field_types, named_to_vec,
    number_idents, unnamed_to_vec, RefType,
};
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{Data, DeriveInput, Field, Fields};

/// Provides the hook to expand `#[derive(Into)]` into an implementation of `Into`
pub fn expand(input: &DeriveInput, trait_name: &str) -> TokenStream {
    let (ref_type, _) = RefType::from_derive(trait_name);
    let input_type = &input.ident;
    let field_vec: Vec<_>;
    let (field_names, fields) = match input.data {
        Data::Struct(ref data_struct) => match data_struct.fields {
            Fields::Unnamed(ref fields) => {
                field_vec = unnamed_to_vec(fields);
                (tuple_field_names(&field_vec), field_vec)
            }
            Fields::Named(ref fields) => {
                field_vec = named_to_vec(fields);
                (struct_field_names(&field_vec), field_vec)
            }
            Fields::Unit => (vec![], vec![]),
        },
        _ => panic!("Only structs can derive Into"),
    };

    let original_types = &get_field_types(&fields);
    let reference = ref_type.reference();
    let lifetime = ref_type.lifetime();
    let reference_with_lifetime = ref_type.reference_with_lifetime();

    let generics_impl;
    let (_, ty_generics, where_clause) = input.generics.split_for_impl();
    let (impl_generics, _, _) = if ref_type.is_ref() {
        generics_impl = add_extra_generic_param(&input.generics, lifetime.clone());
        generics_impl.split_for_impl()
    } else {
        input.generics.split_for_impl()
    };

    quote! {
        impl#impl_generics ::core::convert::From<#reference_with_lifetime #input_type#ty_generics> for
            (#(#reference_with_lifetime #original_types),*) #where_clause {

            #[allow(unused_variables)]
            #[inline]
            fn from(original: #reference_with_lifetime #input_type#ty_generics) -> (#(#reference_with_lifetime #original_types),*) {
                (#(#reference original.#field_names),*)
            }
        }
    }
}

fn tuple_field_names(fields: &[&Field]) -> Vec<TokenStream> {
    number_idents(fields.len())
        .iter()
        .map(|f| f.into_token_stream())
        .collect()
}

fn struct_field_names(fields: &[&Field]) -> Vec<TokenStream> {
    field_idents(fields)
        .iter()
        .map(|f| (*f).into_token_stream())
        .collect()
}
