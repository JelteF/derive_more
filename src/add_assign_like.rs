use quote::Tokens;
use syn::{Body, Ident, VariantData, MacroInput};
use add_like::{tuple_exprs, struct_exprs};

pub fn expand(input: &MacroInput, trait_name: &str) -> Tokens {
    let trait_ident = Ident::from(trait_name);
    let method_name = trait_name.to_string();
    let method_name = method_name.trim_right_matches("Assign");
    let method_name = method_name.to_lowercase();
    let method_ident = Ident::from(method_name.to_string() + "_assign");
    let input_type = &input.ident;

    let exprs = match input.body {
        Body::Struct(VariantData::Tuple(ref fields)) => tuple_exprs(fields, &method_ident),
        Body::Struct(VariantData::Struct(ref fields)) => struct_exprs(fields, &method_ident),

        _ => panic!(format!("Only structs can use derive({})", trait_name)),
    };

    quote!(
        impl ::std::ops::#trait_ident for #input_type {
            fn #method_ident(&mut self, rhs: #input_type) {
                #(#exprs;
                  )*
            }
        }
    )
}
