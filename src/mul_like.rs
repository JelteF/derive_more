use quote::{ToTokens, Tokens};
use syn::{parse_str, Data, DeriveInput, Field, Generics, Ident, Fields, GenericParam, WhereClause};
use std::iter;
use std::collections::HashSet;
use utils::{field_idents, get_field_types_iter, number_idents, unnamed_to_vec, named_to_vec};

pub fn expand(input: &DeriveInput, trait_name: &str) -> Tokens {
    let trait_ident = Ident::from(trait_name);
    let trait_path = &quote!(::std::ops::#trait_ident);
    let method_name = trait_name.to_lowercase();
    let method_ident = &Ident::from(method_name);
    let input_type = &input.ident;

    let (block, fields) = match input.data {
        Data::Struct(ref data_struct) => match data_struct.fields {
            Fields::Unnamed(ref fields) => {
                let field_vec = unnamed_to_vec(fields);
                (tuple_content(input_type, &field_vec, method_ident), field_vec)
            },
            Fields::Named(ref fields) => {
                let field_vec = named_to_vec(fields);
                (struct_content(input_type, &field_vec, method_ident), field_vec)
            }
            _ => panic!(format!("Unit structs cannot use derive({})", trait_name)),
        },
        _ => panic!(format!("Only structs can use derive({})", trait_name)),
    };

    let scalar_ident = &Ident::from("__RhsT");
    let tys: &HashSet<_> = &get_field_types_iter(&fields).collect();
    let tys2 = tys;
    let scalar_iter = iter::repeat(scalar_ident);
    let trait_path_iter = iter::repeat(trait_path);

    let type_where_clauses: WhereClause = parse_str(&quote!{
        where #(#tys: #trait_path_iter<#scalar_iter, Output=#tys2>),*
    }.to_string()).unwrap();

    let new_generics = get_mul_generics(input, &fields, scalar_ident, type_where_clauses);
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

pub fn get_mul_generics<'a>(
    input: &'a DeriveInput,
    fields: &'a Vec<&'a Field>,
    scalar_ident: &Ident,
    mut type_where_clauses: WhereClause,
) -> Generics {
    //let constraints = if fields.len() > 1 {
    //    // If the struct has more than one field the rhs needs to be copied for each
    //    // field
    //    vec![syn::parse_str("::std::marker::Copy").unwrap()]
    //} else {
    //    vec![]
    //};

    //let new_typaram = TypeParam {
    //    attrs: vec![],
    //    ident: scalar_ident.clone(),
    //    bounds: constraints,
    //    default: None,
    //};
    let new_typaram: GenericParam = parse_str(&if fields.len() > 1 {
        quote!(#scalar_ident: ::std::marker::Copy)
    } else {
        quote!(#scalar_ident)
    }.to_string()).unwrap();

    let mut new_generics = input.generics.clone();
    new_generics.params.push(new_typaram);
    if let Some(old_where) = new_generics.where_clause {
        type_where_clauses.predicates.extend(old_where.predicates)
    }
    new_generics.where_clause = Some(type_where_clauses);

    new_generics
}

fn tuple_content<'a, T: ToTokens>(
    input_type: &T,
    fields: &'a Vec<&'a Field>,
    method_ident: &Ident,
) -> Tokens {
    let exprs = tuple_exprs(fields, method_ident);
    quote!(#input_type(#(#exprs),*))
}

pub fn tuple_exprs(fields: &Vec<&Field>, method_ident: &Ident) -> Vec<Tokens> {
    number_idents(fields.len())
        .iter()
        .map(|i| quote!(self.#i.#method_ident(rhs)))
        .collect()
}

fn struct_content<'a, T: ToTokens>(
    input_type: &T,
    fields: &'a Vec<&'a Field>,
    method_ident: &Ident,
) -> Tokens {
    let exprs = struct_exprs(fields, method_ident);
    let field_names = field_idents(fields);
    quote!(#input_type{#(#field_names: #exprs),*})
}

pub fn struct_exprs(fields: &Vec<&Field>, method_ident: &Ident) -> Vec<Tokens> {
    field_idents(fields)
        .iter()
        .map(|f| quote!(self.#f.#method_ident(rhs)))
        .collect()
}
