use quote::{Tokens, ToTokens};
use syn::{Body, Field, Ident, VariantData, MacroInput, Ty, TyParam, TyParamBound,
          parse_ty_param_bound, parse_where_clause};
use std::iter;
use std::collections::HashSet;
use utils::{get_field_types_iter, add_extra_ty_param_bound_rhs};


pub fn expand(input: &MacroInput, trait_name: &str) -> Tokens {
    let trait_ident = Ident::from(trait_name);
    let scalar_ident = &Ident::from("__rhs_T");
    let trait_path = &quote!(::std::ops::#trait_ident);
    let method_name = trait_name.to_lowercase();
    let method_ident = &Ident::from(method_name);
    let input_type = &input.ident;

    let ((block, tys), num_fields) = match input.body {
        Body::Struct(VariantData::Tuple(ref fields)) => {
            (tuple_content(input_type, fields, method_ident), fields.len())
        }
        Body::Struct(VariantData::Struct(ref fields)) => {
            (struct_content(input_type, fields, method_ident), fields.len())
        }

        _ => panic!(format!("Only structs can use derive({})", trait_name)),
    };

    let tys = &tys;
    let tys2 = tys;
    let scalar_iter = iter::repeat(scalar_ident);
    let trait_path_iter = iter::repeat(trait_path);


    let type_where_clauses = quote!{
        where #(#tys: #trait_path_iter<#scalar_iter, Output=#tys2>),*
    };

    let mut type_where_clauses = parse_where_clause(&type_where_clauses.to_string()).unwrap();

    let constraints = if num_fields > 1 {
        // If the struct has more than one field the rhs needs to be copied for each
        // field
        vec![parse_ty_param_bound("::std::marker::Copy").unwrap()]
    } else {
        vec![]
    };

    let new_typaram = TyParam {
        attrs: vec![],
        ident: scalar_ident.clone(),
        bounds: constraints,
        default: None,
    };

    let mut new_generics = input.generics.clone();
    new_generics.ty_params.push(new_typaram);
    new_generics.where_clause.predicates.append(&mut type_where_clauses.predicates);

    let (impl_generics, _, where_clause) = new_generics.split_for_impl();
    let (_, ty_generics, _) = input.generics.split_for_impl();

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

    let body = quote!(#input_type(#(self.#count.#method_iter(rhs)),*));
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
        #input_type{#(#field_names: self.#field_names2.#method_iter(rhs)),*}
    };
    (body, tys)
}
