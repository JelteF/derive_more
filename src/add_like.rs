use quote::{Tokens, ToTokens};
use syn::{Body, Field, Ident, Variant, VariantData, MacroInput};
use std::iter;
use utils::numbered_vars;

pub fn expand(input: &MacroInput, trait_name: &str) -> Tokens {
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

        _ => panic!(format!("Only structs and enums can use derive({})", trait_name))
    };

    quote!(
        impl ::std::ops::#trait_ident for #input_type {
            type Output = #output_type;
            fn #method_ident(self, rhs: #input_type) -> #output_type {
                #block
            }
        }
    )
}

fn tuple_content<T: ToTokens>(input_type: &T, fields: &Vec<Field>, method_ident: &Ident) -> Tokens  {
    let exprs = tuple_exprs(fields, method_ident);
    quote!(#input_type(#(#exprs),*))
}

pub fn tuple_exprs(fields: &Vec<Field>, method_ident: &Ident) -> Vec<Tokens> {
    let mut exprs = vec![];

    for i in 0..fields.len() {
        let i = Ident::from(i.to_string());
        // generates `self.0.add(rhs.0)`
        let expr = quote!(self.#i.#method_ident(rhs.#i));
        exprs.push(expr);
    }
    return exprs

}


fn struct_content(input_type: &Ident, fields: &Vec<Field>, method_ident: &Ident) -> Tokens  {
    // It's safe to unwrap because struct fields always have an identifier
    let exprs = struct_exprs(fields, method_ident);
    let field_ids = fields.iter().map(|f| f.clone().ident.unwrap());

    quote!(#input_type{#(#field_ids: #exprs),*})
}

pub fn struct_exprs(fields: &Vec<Field>, method_ident: &Ident) -> Vec<Tokens> {
    let mut exprs = vec![];

    for field in fields {
        // It's safe to unwrap because struct fields always have an identifier
        let field_id = field.ident.clone().unwrap();
        // generates `x: self.x.add(rhs.x)`
        let expr = quote!(self.#field_id.#method_ident(rhs.#field_id));
        exprs.push(expr)
    }
    return exprs
}


fn enum_content(input_type: &Ident, variants: &Vec<Variant>, method_ident: &Ident) -> Tokens  {
    let mut matches = vec![];
    let method_iter = iter::repeat(method_ident);

    for variant in variants {
        let subtype = &variant.ident;
        let subtype = quote!(#input_type::#subtype);

        match variant.data {
            VariantData::Tuple(ref fields) => {
                // The patern that is outputted should look like this:
                // (Subtype(left_vars), TypePath(right_vars)) => Ok(TypePath(exprs))
                let size = fields.len();
                let l_vars = &numbered_vars(size, "l_");
                let r_vars = &numbered_vars(size, "r_");
                let method_iter = method_iter.clone();
                let matcher = quote!{
                    (#subtype(#(#l_vars),*),
                     #subtype(#(#r_vars),*)) => {
                        Ok(#subtype(#(#l_vars.#method_iter(#r_vars)),*))
                    }
                };
                matches.push(matcher);
            },
            VariantData::Struct(ref fields) => {
                // The patern that is outputted should look like this:
                // (Subtype{a: __l_a, ...}, Subtype{a: __r_a, ...} => {
                //     Ok(Subtype{a: __l_a.add(__r_a), ...})
                // }
                let size = fields.len();
                let field_names: &Vec<_> = &fields.iter().map(|f| f.ident.as_ref().unwrap()).collect();
                let l_vars = &numbered_vars(size, "l_");
                let r_vars = &numbered_vars(size, "r_");
                let method_iter = method_iter.clone();
                let matcher = quote!{
                    (#subtype{#(#field_names: #l_vars),*},
                     #subtype{#(#field_names: #r_vars),*}) => {
                        Ok(#subtype{#(#field_names: #l_vars.#method_iter(#r_vars)),*})
                    }
                };
                matches.push(matcher);
            },
            VariantData::Unit =>  {
                matches.push(quote!((#subtype, #subtype) => Err("Cannot add unit types together")));
            },
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
