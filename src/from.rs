use std::collections::HashMap;

use quote::{Tokens, ToTokens};
use syn::{Body, Field, Variant, VariantData, MacroInput};
use utils::{number_idents, get_field_types, field_idents};


/// Provides the hook to expand `#[derive(From)]` into an implementation of `From`
pub fn expand(input: &MacroInput, _: &str) -> Tokens {
    match input.body {
        Body::Struct(VariantData::Tuple(ref fields)) => tuple_from(input, fields),
        Body::Struct(VariantData::Struct(ref fields)) => struct_from(input, fields),
        Body::Enum(ref variants) => enum_from(input, variants),
        _ => panic!("Only structs and enums can derive From"),
    }
}

pub fn from_impl<T: ToTokens>(input: &MacroInput, fields: &Vec<Field>, body: T) -> Tokens {
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let input_type = &input.ident;
    let original_types = &get_field_types(fields);
    quote!{
        impl#impl_generics ::std::convert::From<(#(#original_types),*)> for #input_type#ty_generics #where_clause {
            fn from(original: (#(#original_types),*)) -> #input_type#ty_generics {
                #body
            }
        }
    }
}

fn tuple_from(input: &MacroInput, fields: &Vec<Field>) -> Tokens {
    let input_type = &input.ident;
    let body = tuple_body(input_type, fields);
    from_impl(input, fields, body)
}

fn tuple_body<T: ToTokens>(return_type: T, fields: &Vec<Field>) -> Tokens {
    if fields.len() == 1 {
        quote!(#return_type(original))
    } else {
        let field_names = &number_idents(fields.len());
        quote!(#return_type(#(original.#field_names),*))
    }
}

fn struct_from(input: &MacroInput, fields: &Vec<Field>) -> Tokens {
    let input_type = &input.ident;
    let body = struct_body(input_type, fields);
    from_impl(input, fields, body)
}

fn struct_body<T: ToTokens>(return_type: T, fields: &Vec<Field>) -> Tokens {
    if fields.len() == 1 {
        let field_name = &fields[0].ident;
        quote!(#return_type{#field_name: original})
    } else {
        let argument_field_names = &number_idents(fields.len());
        let field_names = &field_idents(fields);
        quote!(#return_type{#(#field_names: original.#argument_field_names),*})
    }
}


fn enum_from(input: &MacroInput, variants: &Vec<Variant>) -> Tokens {
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
            _ => {}
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
                    tokens.append(&from_impl(input, fields, body).to_string());
                }
            }

            VariantData::Struct(ref fields) => {
                let original_types = get_field_types(fields);

                if *type_signature_counts.get(&original_types).unwrap() == 1 {
                    let variant_ident = &variant.ident;
                    let body = struct_body(quote!(#input_type::#variant_ident), fields);
                    tokens.append(&from_impl(input, fields, body).to_string());
                }
            }
            _ => {}
        }
    }
    tokens
}
