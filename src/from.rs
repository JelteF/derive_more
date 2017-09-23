use std::collections::HashMap;

use quote::{Tokens, ToTokens};
use syn::{Body, Field, Variant, VariantData, DeriveInput, Ident, TyParam, parse_ty_param_bound};
use utils::{number_idents, numbered_vars, get_field_types, field_idents};


/// Provides the hook to expand `#[derive(From)]` into an implementation of `From`
pub fn expand(input: &DeriveInput, _: &str) -> Tokens {
    match input.body {
        Body::Struct(VariantData::Tuple(ref fields)) => tuple_from(input, fields),
        Body::Struct(VariantData::Struct(ref fields)) => struct_from(input, fields),
        Body::Enum(ref variants) => enum_from(input, variants),
        Body::Struct(VariantData::Unit) => struct_from(input, &vec![]),
    }
}

pub fn simple_from_impl<T: ToTokens>(input: &DeriveInput, fields: &Vec<Field>, body: T) -> Tokens {
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let input_type = &input.ident;
    let original_types = &get_field_types(fields);
    quote!{
        impl#impl_generics ::std::convert::From<(#(#original_types),*)> for #input_type#ty_generics #where_clause {

            #[allow(unused_variables)]
            fn from(original: (#(#original_types),*)) -> #input_type#ty_generics {
                #body
            }
        }
    }
}

pub fn into_from_impl<T: ToTokens>(input: &DeriveInput, fields: &Vec<Field>, body: T) -> Tokens {
    let mut generics = input.generics.clone();
    let input_type = &input.ident;
    let original_types = &get_field_types(fields);

    let (original_impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let (impl_generics, types) = if fields.len() > 1 {
        let into_names = &numbered_vars(original_types.len(), "Into");
        let mut extra_ty_params: Vec<TyParam> = into_names
            .iter()
            .zip(original_types)
            .map(|(name, ty)| {
                TyParam {
                    attrs: vec![],
                    ident: Ident::from(name.to_string()),
                    bounds: vec![parse_ty_param_bound(&quote!(::std::convert::Into<#ty>).to_string())
                                     .unwrap()],
                    default: None,
                }
            }).collect();
        generics.ty_params.append(&mut extra_ty_params);
        (generics.split_for_impl().0, quote!((#(#into_names,)*)))
    } else {
        (original_impl_generics, quote!((#(#original_types),*)))
    };
    quote!{
        impl#impl_generics ::std::convert::From<#types> for #input_type#ty_generics #where_clause {

            #[allow(unused_variables)]
            fn from(original: #types) -> #input_type#ty_generics {
                #body
            }
        }
    }
}

fn tuple_from(input: &DeriveInput, fields: &Vec<Field>) -> Tokens {
    let input_type = &input.ident;
    let body = tuple_body(input_type, fields);
    into_from_impl(input, fields, body)
}

fn tuple_body<T: ToTokens>(return_type: T, fields: &Vec<Field>) -> Tokens {
    if fields.len() == 1 {
        quote!(#return_type(original.into()))
    } else {
        let field_names = &number_idents(fields.len());
        quote!(#return_type(#(original.#field_names.into()),*))
    }
}

fn struct_from(input: &DeriveInput, fields: &Vec<Field>) -> Tokens {
    let input_type = &input.ident;
    let body = struct_body(input_type, fields);
    into_from_impl(input, fields, body)
}

fn struct_body<T: ToTokens>(return_type: T, fields: &Vec<Field>) -> Tokens {
    if fields.len() == 1 {
        let field_name = &fields[0].ident;
        quote!(#return_type{#field_name: original.into()})
    } else {
        let argument_field_names = &number_idents(fields.len());
        let field_names = &field_idents(fields);
        quote!(#return_type{#(#field_names: original.#argument_field_names.into()),*})
    }
}


fn enum_from(input: &DeriveInput, variants: &Vec<Variant>) -> Tokens {
    let mut type_signature_counts = HashMap::new();
    let input_type = &input.ident;

    for variant in variants {
        match variant.data {
            VariantData::Tuple(ref fields) |
            VariantData::Struct(ref fields) => {
                let original_types = get_field_types(fields);
                let counter = type_signature_counts.entry(original_types).or_insert(0);
                *counter += 1;
            }
            VariantData::Unit => {
                let counter = type_signature_counts.entry(vec![]).or_insert(0);
                *counter += 1;
            }
        }
    }

    let mut tokens = Tokens::new();

    for variant in variants.iter() {
        match variant.data {

            VariantData::Tuple(ref fields) => {
                let original_types = get_field_types(fields);

                if *type_signature_counts.get(&original_types).unwrap() == 1 {
                    let variant_ident = &variant.ident;
                    let body = tuple_body(quote!(#input_type::#variant_ident), fields);
                    tokens.append(&simple_from_impl(input, fields, body).to_string());
                }
            }

            VariantData::Struct(ref fields) => {
                let original_types = get_field_types(fields);

                if *type_signature_counts.get(&original_types).unwrap() == 1 {
                    let variant_ident = &variant.ident;
                    let body = struct_body(quote!(#input_type::#variant_ident), fields);
                    tokens.append(&simple_from_impl(input, fields, body).to_string());
                }
            }
            VariantData::Unit => {
                if *type_signature_counts.get(&vec![]).unwrap() == 1 {
                    let variant_ident = &variant.ident;
                    let body = struct_body(quote!(#input_type::#variant_ident), &vec![]);
                    tokens.append(&simple_from_impl(input, &vec![], body).to_string());
                }
            }
        }
    }
    tokens
}
