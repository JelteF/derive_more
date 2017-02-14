use quote::{Tokens, ToTokens};
use syn::{Body, Field, VariantData, MacroInput};
use utils::{number_idents, get_field_types, field_idents, numbered_vars};


/// Provides the hook to expand `#[derive(Constructor)]` into an implementation of `Constructor`
pub fn expand(input: &MacroInput, _: &str) -> Tokens {
    match input.body {
        Body::Struct(VariantData::Tuple(ref fields)) => tuple_from(input, fields),
        Body::Struct(VariantData::Struct(ref fields)) => struct_from(input, fields),
        _ => panic!("Only structs can derive a constructor"),
    }
}

fn from_impl<T: ToTokens>(input: &MacroInput, fields: &Vec<Field>, body: T) -> Tokens {
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let input_type = &input.ident;
    let original_types = &get_field_types(fields);
    let vars = &numbered_vars(fields.len(), "");
    quote!{
        impl#impl_generics #input_type#ty_generics #where_clause {
            #[allow(unused_parens)]
            fn new(#(#vars: #original_types),*) -> #input_type#ty_generics {
                let original = (#(#vars),*);
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
