use quote::Tokens;
use syn::{Body, Field, Ident, Variant, VariantData, MacroInput, Ty};

pub fn expand(input: &MacroInput) -> Tokens {
    let trait_name = "Add";
    let trait_ident = Ident::from(trait_name);
    let method_name = trait_name.to_lowercase();
    let method_ident = Ident::from(method_name.clone());
    let input_type = &input.ident;

    let (output_type, block) = match input.body {
        Body::Struct(VariantData::Tuple(ref fields)) => {
            (input_type, tuple_content(input_type, fields, &method_ident))
        },
        Body::Struct(VariantData::Struct(ref fields)) => {
            (input_type, struct_content(input_type, fields, &method_ident))
        },

        _ => panic!(format!("Only structs and enums can use dervie({})", trait_name))
    };

    quote!(
        impl std::ops::#trait_ident for #input_type {
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
        // generates `self.0.add(rhs.0)`
        let expr = quote!(self.#i.#method_ident(rhs.#i));
        exprs.push(expr);
    }

    quote!(#input_type(#(#exprs),*))
}


fn struct_content(input_type: &Ident, fields: &Vec<Field>, method_ident: &Ident) -> Tokens  {
    let mut exprs = vec![];

    for field in fields {
        // It's safe to unwrap because struct fields always have an identifier
        let field_id = field.ident.clone().unwrap();
        // generates `x: self.x.add(rhs.x)`
        let expr = quote!(#field_id: self.#field_id.#method_ident(rhs.#field_id));
        exprs.push(expr)
    }

    quote!(#input_type{#(#exprs),*})
}

