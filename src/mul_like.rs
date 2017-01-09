use quote::{Tokens, ToTokens};
use syn::{Body, Field, Ident, Variant, VariantData, MacroInput, Ty};


pub fn expand(input: &MacroInput, trait_name: &str) -> Tokens {
    let trait_ident = Ident::from(trait_name);
    let trait_path = quote!(::std::ops::#trait_ident);
    let method_name = trait_name.to_lowercase();
    let method_ident = &Ident::from(method_name);
    let input_type = &input.ident;

    let (block, types) = match input.body {
        Body::Struct(VariantData::Tuple(ref fields)) => {
            tuple_content(input_type, fields, method_ident)
        },
        Body::Struct(VariantData::Struct(ref fields)) => {
            struct_content(input_type, fields, method_ident)
        },

        _ => panic!(format!("Only structs can use derive({})", trait_name))
    };
    quote!(
        impl <T> #trait_path for #input_type where T:
                ::std::marker::Copy {
            type Output = #input_type;
            fn #method_ident(self, rhs) {
                #block
            }
        }

    )
}

fn tuple_content<T: ToTokens>(input_type: &T, fields: &Vec<Field>, method_ident: &Ident) -> (Tokens, Vec<Tokens>)  {
    (quote!(), vec![])
}

fn struct_content<T: ToTokens>(input_type: &T, fields: &Vec<Field>, method_ident: &Ident) -> (Tokens, Vec<Tokens>)  {
    (quote!(), vec![])
}
