use crate::mul_helpers::{struct_exprs, tuple_exprs};
use crate::utils::{
    add_where_clauses_for_new_ident, field_idents, get_field_types_iter, named_to_vec,
    unnamed_to_vec,
};
use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use std::collections::HashSet;
use std::iter;
use syn::{Data, DeriveInput, Field, Fields, Ident};

pub fn expand(input: &DeriveInput, trait_name: &str) -> TokenStream {
    let trait_ident = Ident::new(trait_name, Span::call_site());
    let trait_path = &quote!(::core::ops::#trait_ident);
    let method_name = trait_name.to_lowercase();
    let method_ident = &Ident::new(&method_name, Span::call_site());
    let input_type = &input.ident;

    let (block, fields) = match input.data {
        Data::Struct(ref data_struct) => match data_struct.fields {
            Fields::Unnamed(ref fields) => {
                let field_vec = unnamed_to_vec(fields);
                (
                    tuple_content(input_type, &field_vec, method_ident),
                    field_vec,
                )
            }
            Fields::Named(ref fields) => {
                let field_vec = named_to_vec(fields);
                (
                    struct_content(input_type, &field_vec, method_ident),
                    field_vec,
                )
            }
            _ => panic!(format!("Unit structs cannot use derive({})", trait_name)),
        },
        _ => panic!(format!("Only structs can use derive({})", trait_name)),
    };

    let scalar_ident = &Ident::new("__RhsT", Span::call_site());
    let tys = get_field_types_iter(&fields).collect::<HashSet<_>>();
    let tys = tys.iter();
    let scalar_iter = iter::repeat(scalar_ident);
    let trait_path_iter = iter::repeat(trait_path);

    let type_where_clauses = quote! {
        where #(#tys: #trait_path_iter<#scalar_iter, Output=#tys>),*
    };

    let new_generics =
        add_where_clauses_for_new_ident(&input.generics, &fields, scalar_ident, type_where_clauses, true);
    let (impl_generics, _, where_clause) = new_generics.split_for_impl();
    let (_, ty_generics, _) = input.generics.split_for_impl();

    quote!(
        impl#impl_generics  #trait_path<#scalar_ident> for #input_type#ty_generics #where_clause {
            type Output = #input_type#ty_generics;
            #[inline]
            fn #method_ident(self, rhs: #scalar_ident) -> #input_type#ty_generics {
                #block
            }
        }

    )
}

fn tuple_content<'a, T: ToTokens>(
    input_type: &T,
    fields: &[&'a Field],
    method_ident: &Ident,
) -> TokenStream {
    let exprs = tuple_exprs(fields, method_ident);
    quote!(#input_type(#(#exprs),*))
}

fn struct_content<'a, T: ToTokens>(
    input_type: &T,
    fields: &[&'a Field],
    method_ident: &Ident,
) -> TokenStream {
    let exprs = struct_exprs(fields, method_ident);
    let field_names = field_idents(fields);
    quote!(#input_type{#(#field_names: #exprs),*})
}
