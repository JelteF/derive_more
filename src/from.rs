use std::collections::HashMap;

use quote::{Tokens, ToTokens};
use syn::{Body, Field, Ident, Variant, VariantData, MacroInput, Ty};
use utils::number_idents;


/// Provides the hook to expand `#[derive(From)]` into an implementation of `From`
pub fn expand(input: &MacroInput, _: &str) -> Tokens {
    let input_type = &input.ident;
    match input.body {
        Body::Struct(VariantData::Tuple(ref fields)) => tuple_from(input, fields),
        Body::Struct(VariantData::Struct(ref fields)) => struct_from(input, fields),
        Body::Enum(ref variants) => enum_from(input, variants),
        _ => panic!("Only tuple structs and enums can derive From"),
    }
}

pub fn from_impl<T: ToTokens>(input: &MacroInput, fields: &Vec<Field>, body: T) -> Tokens {
    let generics = &input.generics;
    let input_type = &input.ident;
    let original_types: &Vec<_> = &fields.iter().map(|f| &f.ty).collect();
    quote!{
        impl#generics ::std::convert::From<(#(#original_types),*)> for #input_type#generics {
            fn from(original: (#(#original_types),*)) -> #input_type#generics {
                #body
            }
        }
    }
}

fn tuple_from(input: &MacroInput, fields: &Vec<Field>) -> Tokens {
    let input_type = &input.ident;
    let body = if fields.len() == 1 {
        quote!(#input_type(original))
    } else {
        let field_names = &number_idents(fields.len());
        quote!(#input_type(#(original.#field_names),*))

    };
    from_impl(input, fields, body)
}

fn struct_from(input: &MacroInput, fields: &Vec<Field>) -> Tokens {
    let input_type = &input.ident;
    let body = if fields.len() == 1 {
        let field_name = &fields[0].ident;
        quote!(#input_type{#field_name :original})
    } else {
        let argument_field_names = &number_idents(fields.len());
        let field_names: &Vec<_> = &fields.iter().map(|f| f.ident.as_ref().unwrap()).collect();
        quote!(#input_type{#(#field_names: original.#argument_field_names),*})
    };
    from_impl(input, fields, body)
}

fn enum_from(input: &MacroInput, variants: &Vec<Variant>) -> Tokens {
    let mut types = vec![];
    let mut idents = vec![];
    let mut type_counts = HashMap::new();

    for variant in variants {
        match variant.data {
            VariantData::Tuple(ref fields) => {
                let original_types: &Vec<_> = &fields.iter().map(|f| &f.ty).collect();
                idents.push(&variant.ident);
                types.push(original_types);
                let counter = type_counts.entry(original_types).or_insert(0);
                *counter += 1;
            }
            VariantData::Struct(ref fields) => {}
            _ => {}
        }
    }

    let mut tokens = Tokens::new();

    for (ident, old_type) in idents.iter().zip(types) {
        if *type_counts.get(&old_type).unwrap() != 1 {
            // If more than one variant is present with the same type signature don't
            // add automatic From, since it is ambiguous.
            continue;
        }

        tokens.append(&quote!{
            impl ::std::convert::From<#old_type> for #enum_ident {
                fn from(original: #old_type) -> #enum_ident {
                    #enum_ident::#ident(original)
                }
            }
        }
            .to_string())
    }
    tokens
}
