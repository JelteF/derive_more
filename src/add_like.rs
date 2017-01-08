use quote::{Tokens, ToTokens};
use syn::{Body, Field, Ident, Variant, VariantData, MacroInput, Ty};
use std::iter;

pub fn expand(input: &MacroInput) -> Tokens {
    let trait_name = "Add";
    let trait_ident = Ident::from(trait_name);
    let method_name = trait_name.to_lowercase();
    let method_ident = Ident::from(method_name.clone());
    let input_type = &input.ident;

    let (output_type, block) = match input.body {
        Body::Struct(VariantData::Tuple(ref fields)) => {
            (quote!(#input_type),
             tuple_content(input_type, fields, &method_ident))
        },
        Body::Struct(VariantData::Struct(ref fields)) => {
            (quote!(#input_type),
             struct_content(input_type, fields, &method_ident))
        },
        Body::Enum(ref definition) => {
            (quote!(Result<#input_type, &'static str>),
             enum_content(input_type, definition, &method_ident))
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

fn tuple_content<T: ToTokens>(input_type: &T, fields: &Vec<Field>, method_ident: &Ident) -> Tokens  {
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

fn enum_content(input_type: &Ident, variants: &Vec<Variant>, method_ident: &Ident) -> Tokens  {
    let mut matches = vec![];

    for variant in variants {
        match variant.data {
            VariantData::Tuple(ref fields) => {
                let variant_ident = &variant.ident;
                let variant_ident = quote!(#input_type::#variant_ident);
                let size = fields.len();
                let l_vars:  &Vec<_> = &(0..size).map(|i| Ident::from(format!("__l_{}", i))).collect();
                let r_vars:  &Vec<_> = &(0..size).map(|i| Ident::from(format!("__r_{}", i))).collect();
                let method_iter = iter::repeat(method_ident);
                let matcher = quote!{
                    (#variant_ident(#(#l_vars),*),
                     #variant_ident(#(#r_vars),*)) => {
                        Ok(#variant_ident(#(#l_vars.#method_iter(#r_vars)),*))
                    }
                };
                matches.push(matcher);
            }
            _ => {}
        }
    }

    if variants.len() > 1 {
        // In the strange case where there's only one enum variant this is would be an unreachable
        // match.
        matches.push(quote!(_ => Err("Trying to add mismatched enum types")));
    }
    quote!(
        match (self, rhs) {
            #(#matches),*
        }
    )
}
