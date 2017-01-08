use quote::Tokens;
use syn::{Body, Field, Ident, Variant, VariantData, MacroInput, Ty};

pub fn expand(input: &MacroInput) -> Tokens {
    let trait_name = "Add";
    let method_name = trait_name.to_lowercase();
    let method_ident = Ident::from(method_name.clone());
    let input_type = &input.ident;

    let (output_type, block) = match input.body {
        Body::Struct(VariantData::Tuple(ref fields)) => {
            (input_type, tuple_content(input_type, fields, &method_ident))
        },
        _ => panic!(format!("Only structs and enums can use dervie({})", trait_name))
    };

    quote!(
        impl std::ops::#trait_name for #input_type {
            type Output = #output_type;
            fn #method_ident(self, rhs: #input_type) -> #output_type {
                #block
            }
        }
    )
}

fn tuple_content(input_type: &Ident, fields: &Vec<Field>, method_ident: &Ident) -> Tokens  {
    let mut exprs = vec![];

    for i in 0..fields.len() {
        let i = Ident::from(i.to_string());
        let expr = quote!(self.#i.#method_ident(rhs.#i));
        println!("{:?}", expr);
        exprs.push(expr);
        return quote!(#input_type(#expr));
    }

    let call = quote!(#input_type(#exprs));
    println!("{:?}", call);
    call
}

