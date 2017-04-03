use quote::Tokens;
use syn::{Body, Ident, Field, VariantData, MacroInput};
use utils::{get_field_types, field_idents, numbered_vars};


/// Provides the hook to expand `#[derive(Constructor)]` into an implementation of `Constructor`
pub fn expand(input: &MacroInput, _: &str) -> Tokens {
    let input_type = &input.ident;
    let empty_fields = &vec![];
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let ((body, vars), fields) = match input.body {
        Body::Struct(VariantData::Tuple(ref fields)) => (tuple_body(input_type, fields), fields),
        Body::Struct(VariantData::Struct(ref fields)) => (struct_body(input_type, fields), fields),
        Body::Struct(VariantData::Unit) => (struct_body(input_type, empty_fields), empty_fields),
        _ => panic!("Only structs can derive a constructor"),
    };
    let original_types = &get_field_types(fields);
    quote!{
        impl#impl_generics #input_type#ty_generics #where_clause {
            pub fn new(#(#vars: #original_types),*) -> #input_type#ty_generics {
                #body
            }
        }
    }
}

fn tuple_body(return_type: &Ident, fields: &Vec<Field>) -> (Tokens, Vec<Ident>) {
    let vars = &numbered_vars(fields.len(), "");
    (quote!(#return_type(#(#vars),*)), vars.clone())
}

fn struct_body(return_type: &Ident, fields: &Vec<Field>) -> (Tokens, Vec<Ident>) {
    let field_names: &Vec<Ident> = &field_idents(fields).iter().map(|f| (*f).clone()).collect();
    let vars = field_names;
    (quote!(#return_type{#(#field_names: #vars),*}), vars.clone())
}
