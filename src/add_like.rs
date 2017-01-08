use quote::Tokens;
use syn::{Body, Field, Ident, Variant, VariantData, MacroInput, Ty};

pub fn expand(input: &MacroInput) -> Tokens {
    let trait_name = "Add";
    let method_name = trait_name.to_lowercase();
    let method_ident = Ident::from(method_name);
    let input_type = input.ident;
    let (output_type, block) = match input.body {
        Body::Struct(VariantData::Tuple(ref fields)) => {
            (input_type, tuple_content(input_type, fields, method_name))
        },
        _ => panic!(format!("Only structs and enums can use dervie({})", trait_name))
    };
    quote!()
}

fn tuple_content(input_type: Ident, fields: &Vec<Field>, method_name: String) -> Tokens  {
    let mut exprs = vec![];

    for i in 0..fields.len() {
        let i = &i.to_string();
        exprs.push(quote!((format!("self.{}.{}(rhs.{})", i, method_name, i)));
    }

    cx.expr_call_ident(span, type_name, exprs)
}

