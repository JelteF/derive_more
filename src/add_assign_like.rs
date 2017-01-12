use quote::{Tokens};
use syn::{Body, Ident, Variant, VariantData, MacroInput};
use add_like::{tuple_content, struct_content};

pub fn expand(input: &MacroInput, trait_name: &str) -> Tokens {
    let trait_ident = Ident::from(trait_name);
    let method_name = trait_name.to_lowercase().to_string();
    let method_name = method_name.trim_right_matches("Assign");
    let method_ident = Ident::from(method_name.clone().to_string() + "_assign");
    let method_ident_no_assign = Ident::from(method_name.clone());
    let input_type = &input.ident;

    let (output_type, block) = match input.body {
        Body::Struct(VariantData::Tuple(ref fields)) => {
            (quote!(#input_type),
             tuple_content(input_type, fields, &method_ident_no_assign))
        },
        Body::Struct(VariantData::Struct(ref fields)) => {
            (quote!(#input_type),
             struct_content(input_type, fields, &method_ident_no_assign))
        },

        _ => panic!(format!("Only structs can use derive({})", trait_name))
    };

    quote!(
        impl ::std::ops::#trait_ident for #input_type {
            type Output = #output_type;
            fn #method_ident(&mut self, rhs: #input_type) -> #output_type {
                *self = #block
            }
        }
    )
}
