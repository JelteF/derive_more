use std::collections::HashMap;

use quote::{ToTokens, Tokens};
use syn::{Body, DeriveInput, Field, Variant, VariantData, Ident, Type};
use utils::{field_idents, get_field_types, number_idents};


/// Provides the hook to expand `#[derive(FromStr)]` into an implementation of `From`
pub fn expand(input: &DeriveInput, trait_name: &str) -> Tokens {
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let input_type = &input.ident;
    let (result, field_type) = match input.body {
        Body::Struct(VariantData::Tuple(ref fields)) => tuple_from_str(input_type, trait_name, fields),
        // Body::Struct(VariantData::Struct(ref fields)) => struct_newtype(input, fields),
        Body::Enum(_) => panic_one_field(trait_name),
        Body::Struct(VariantData::Unit) => panic_one_field(trait_name),
        _ => panic!("nooo not implemeted yet")
    };
    let trait_path = quote!(::std::str::FromStr);
    quote!{
        impl#impl_generics #trait_path for #input_type#ty_generics #where_clause
        {
            type Err = <#field_type as #trait_path>::Err;
            fn from_str(src: &str) -> Result<Self, Self::Err> {
                return Ok(#result)
            }
        }
    }
}

fn panic_one_field(trait_name : &str) -> ! {
    panic!(format!("Only structs with one field can derive({})", trait_name))
}

fn tuple_from_str<'a>(input_type: &Ident, trait_name: &str, fields: &'a Vec<Field>) -> (Tokens, &'a Type) {
    if fields.len() != 1 {
        panic_one_field(trait_name)
    };
    let field = &fields[0];
    let field_type = &field.ty;
    (quote!(#input_type(#field_type::from_str(src)?)), field_type)
}
