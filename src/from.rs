use std::collections::HashMap;

use quote::{ToTokens, Tokens};
use syn::{Data, DeriveInput, Field, DataEnum, Fields};
use utils::{field_idents, get_field_types, number_idents, unnamed_to_vec, named_to_vec};

/// Provides the hook to expand `#[derive(From)]` into an implementation of `From`
pub fn expand(input: &DeriveInput, trait_name: &str) -> Tokens {
    match input.data {
        Data::Struct(ref data_struct) => match data_struct.fields {
            Fields::Unnamed(ref fields) => tuple_from(input, &unnamed_to_vec(fields)),
            Fields::Named(ref fields) =>struct_from(input, &named_to_vec(fields)),
            Fields::Unit => struct_from(input, &vec![]),
        }
        Data::Enum(ref data_enum) => enum_from(input, data_enum),
        _ => panic!(format!(
            "Only structs and enums can use dervie({})",
            trait_name
        )),
    }
}

pub fn from_impl<T: ToTokens>(input: &DeriveInput, fields: &Vec<&Field>, body: T) -> Tokens {
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let input_type = &input.ident;
    let original_types = &get_field_types(fields);
    quote!{
        impl#impl_generics ::std::convert::From<(#(#original_types),*)> for
            #input_type#ty_generics #where_clause {

            #[allow(unused_variables)]
            fn from(original: (#(#original_types),*)) -> #input_type#ty_generics {
                #body
            }
        }
    }
}

fn tuple_from(input: &DeriveInput, fields: &Vec<&Field>) -> Tokens {
    let input_type = &input.ident;
    let body = tuple_body(input_type, fields);
    from_impl(input, fields, body)
}

fn tuple_body<T: ToTokens>(return_type: T, fields: &Vec<&Field>) -> Tokens {
    if fields.len() == 1 {
        quote!(#return_type(original))
    } else {
        let field_names = &number_idents(fields.len());
        quote!(#return_type(#(original.#field_names),*))
    }
}

fn struct_from(input: &DeriveInput, fields: &Vec<&Field>) -> Tokens {
    let input_type = &input.ident;
    let body = struct_body(input_type, fields);
    from_impl(input, fields, body)
}

fn struct_body<T: ToTokens>(return_type: T, fields: &Vec<&Field>) -> Tokens {
    if fields.len() == 1 {
        let field_name = &fields[0].ident;
        quote!(#return_type{#field_name: original})
    } else {
        let argument_field_names = &number_idents(fields.len());
        let field_names = &field_idents(fields);
        quote!(#return_type{#(#field_names: original.#argument_field_names),*})
    }
}

fn enum_from(input: &DeriveInput, data_enum: &DataEnum) -> Tokens {
    let mut type_signature_counts = HashMap::new();
    let input_type = &input.ident;

    for variant in data_enum.variants {
        match variant.fields {
            Fields::Unnamed(ref fields) => {
                let original_types = get_field_types(&unnamed_to_vec(fields));
                let counter = type_signature_counts.entry(original_types).or_insert(0);
                *counter += 1;
            },
            Fields::Named(ref fields) => {
                let original_types = get_field_types(&named_to_vec(fields));
                let counter = type_signature_counts.entry(original_types).or_insert(0);
                *counter += 1;
            }
            Fields::Unit => {
                let counter = type_signature_counts.entry(vec![]).or_insert(0);
                *counter += 1;
            }
        }
    }

    let mut tokens = Tokens::new();

    for variant in data_enum.variants {
        match variant.fields {
            Fields::Unnamed(ref fields) => {
                let field_vec = &unnamed_to_vec(fields);
                let original_types = get_field_types(field_vec);

                if *type_signature_counts.get(&original_types).unwrap() == 1 {
                    let variant_ident = &variant.ident;
                    let body = tuple_body(quote!(#input_type::#variant_ident), field_vec);
                    from_impl(input, field_vec, body).to_tokens(&mut tokens)
                }
            }

            Fields::Named(ref fields) => {
                let field_vec = &named_to_vec(fields);
                let original_types = get_field_types(field_vec);

                if *type_signature_counts.get(&original_types).unwrap() == 1 {
                    let variant_ident = &variant.ident;
                    let body = struct_body(quote!(#input_type::#variant_ident), field_vec);
                    from_impl(input, field_vec, body).to_tokens(&mut tokens)
                }
            }
            Fields::Unit => {
                if *type_signature_counts.get(&vec![]).unwrap() == 1 {
                    let variant_ident = &variant.ident;
                    let body = struct_body(quote!(#input_type::#variant_ident), &vec![]);
                    from_impl(input, &vec![], body).to_tokens(&mut tokens)
                }
            }
        }
    }
    tokens
}
