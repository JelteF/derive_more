use quote::{Tokens, ToTokens};
use syn::{Body, Field, Ident, Variant, VariantData, MacroInput};
use std::iter;

pub fn expand(input: &MacroInput, trait_name: &str) -> Tokens {
    let trait_ident = Ident::from(trait_name);
    let method_name = trait_name.to_lowercase();
    let method_ident = &Ident::from(method_name);
    let input_type = &input.ident;

    let (output_type, block) = match input.body {
        Body::Struct(VariantData::Tuple(ref fields)) => {
            (quote!(#input_type), tuple_content(input_type, fields, method_ident))
        }
        Body::Struct(VariantData::Struct(ref fields)) => {
            (quote!(#input_type), struct_content(input_type, fields, method_ident))
        }
        Body::Enum(ref definition) => {
            enum_output_type_and_content(input_type, definition, &method_ident)
        }

        _ => panic!(format!("Only structs and enums can use dervie({})", trait_name)),
    };

    quote!(
        impl ::std::ops::#trait_ident for #input_type {
            type Output = #output_type;
            fn #method_ident(self) -> #output_type {
                #block
            }
        }
    )
}

fn tuple_content<T: ToTokens>(input_type: &T, fields: &Vec<Field>, method_ident: &Ident) -> Tokens {
    let mut exprs = vec![];

    for i in 0..fields.len() {
        let i = Ident::from(i.to_string());
        // generates `self.0.add()`
        let expr = quote!(self.#i.#method_ident());
        exprs.push(expr);
    }

    quote!(#input_type(#(#exprs),*))
}


fn struct_content(input_type: &Ident, fields: &Vec<Field>, method_ident: &Ident) -> Tokens {
    let mut exprs = vec![];

    for field in fields {
        // It's safe to unwrap because struct fields always have an identifier
        let field_id = field.ident.as_ref();
        // generates `x: self.x.not()`
        let expr = quote!(#field_id: self.#field_id.#method_ident());
        exprs.push(expr)
    }

    quote!(#input_type{#(#exprs),*})
}

fn enum_output_type_and_content(input_type: &Ident,
                                variants: &Vec<Variant>,
                                method_ident: &Ident)
                                -> (Tokens, Tokens) {
    let mut matches = vec![];
    let mut method_iter = iter::repeat(method_ident);
    // If the enum contains unit types that means it can error.
    let has_unit_type = variants.iter().any(|v| v.data == VariantData::Unit);

    for variant in variants {
        let subtype = &variant.ident;
        let subtype = quote!(#input_type::#subtype);

        match variant.data {
            VariantData::Tuple(ref fields) => {
                // The patern that is outputted should look like this:
                // (Subtype(vars)) => Ok(TypePath(exprs))
                let size = fields.len();
                let vars: &Vec<_> = &(0..size).map(|i| Ident::from(format!("__{}", i))).collect();
                let method_iter = method_iter.by_ref();
                let mut body = quote!(#subtype(#(#vars.#method_iter()),*));
                if has_unit_type {
                    body = quote!(Ok(#body))
                }
                let matcher = quote!{
                    #subtype(#(#vars),*) => {
                        #body
                    }
                };
                matches.push(matcher);
            }
            VariantData::Struct(ref fields) => {
                // The patern that is outputted should look like this:
                // (Subtype{a: __l_a, ...} => {
                //     Ok(Subtype{a: __l_a.neg(__r_a), ...})
                // }
                let size = fields.len();
                let field_names: &Vec<_> =
                    &fields.iter().map(|f| f.ident.as_ref().unwrap()).collect();
                let vars: &Vec<_> = &(0..size).map(|i| Ident::from(format!("__{}", i))).collect();
                let method_iter = method_iter.by_ref();
                let mut body = quote!(#subtype{#(#field_names: #vars.#method_iter()),*});
                if has_unit_type {
                    body = quote!(Ok(#body))
                }
                let matcher = quote!{
                    #subtype{#(#field_names: #vars),*} => {
                        #body
                    }
                };
                matches.push(matcher);
            }
            VariantData::Unit => {
                let message = format!("Cannot {}() unit variants", method_ident.to_string());
                matches.push(quote!(#subtype => Err(#message)));
            }
        }
    }

    let body = quote!(
        match self {
            #(#matches),*
        }
    );

    let output_type = if has_unit_type {
        quote!(Result<#input_type, &'static str>)
    } else {
        quote!(#input_type)
    };

    (output_type, body)
}
