use quote::{Tokens, ToTokens};
use syn::{Body, Field, Ident, VariantData, MacroInput, Ty, TyParam, TyParamBound,
          parse_ty_param_bound};
use std::iter;
use std::collections::HashSet;
use utils::get_field_types_iter;


pub fn expand(input: &MacroInput, trait_name: &str) -> Tokens {
    let trait_ident = Ident::from(trait_name);
    let scalar_ident = &Ident::from("__rhs_T");
    let trait_path = quote!(::std::ops::#trait_ident);
    let method_name = trait_name.to_lowercase();
    let method_ident = &Ident::from(method_name);
    let input_type = &input.ident;
    let mut generics = input.generics.clone();

    let ((block, tys), num_fields) = match input.body {
        Body::Struct(VariantData::Tuple(ref fields)) => {
            (tuple_content(input_type, fields, method_ident), fields.len())
        }
        Body::Struct(VariantData::Struct(ref fields)) => {
            (struct_content(input_type, fields, method_ident), fields.len())
        }

        _ => panic!(format!("Only structs can use derive({})", trait_name)),
    };

    let mut constraints: Vec<TyParamBound> = vec![];
    for t in tys {
        constraints.push(parse_ty_param_bound(
                &quote!(#trait_path<#t, Output=#t>).to_string())
            .unwrap());
    }

    if num_fields > 1 {
        // If the struct has more than one field the rhs needs to be copied for each
        // field
        constraints.push(parse_ty_param_bound("::std::marker::Copy").unwrap())
    }

    let new_typaram = TyParam {
        attrs: vec![],
        ident: scalar_ident.clone(),
        bounds: constraints,
        default: None,
    };
    generics.ty_params.push(new_typaram);

    let (impl_generics, _, _) = generics.split_for_impl();
    let (_, ty_generics, where_clause) = input.generics.split_for_impl();
    quote!(
        impl#impl_generics  #trait_path<#scalar_ident> for #input_type#ty_generics #where_clause {
            type Output = #input_type#ty_generics;
            fn #method_ident(self, rhs: #scalar_ident) -> #input_type#ty_generics {
                #block
            }
        }

    )
}

fn tuple_content<'a, T: ToTokens>(input_type: &T,
                                  fields: &'a Vec<Field>,
                                  method_ident: &Ident)
                                  -> (Tokens, HashSet<&'a Ty>) {
    let tys: HashSet<_> = get_field_types_iter(fields).collect();
    let count = (0..fields.len()).map(|i| Ident::from(i.to_string()));
    let method_iter = iter::repeat(method_ident);

    let body = quote!(#input_type(#(rhs.#method_iter(self.#count)),*));
    (body, tys)
}

fn struct_content<'a, T: ToTokens>(input_type: &T,
                                   fields: &'a Vec<Field>,
                                   method_ident: &Ident)
                                   -> (Tokens, HashSet<&'a Ty>) {
    let tys: HashSet<_> = get_field_types_iter(fields).collect();
    let field_names: &Vec<_> = &fields.iter()
        .map(|f| f.ident.as_ref().unwrap())
        .collect();
    // This is to fix an error in quote when using the same name twice
    let field_names2 = field_names;
    let method_iter = iter::repeat(method_ident);

    let body = quote!{
        #input_type{#(#field_names: rhs.#method_iter(self.#field_names2)),*}
    };
    (body, tys)
}
