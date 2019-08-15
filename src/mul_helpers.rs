use crate::utils::{field_idents, number_idents};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Field, Ident};

pub fn tuple_exprs(fields: &[&Field], method_ident: &Ident) -> Vec<TokenStream> {
    number_idents(fields.len())
        .iter()
        .map(|i| quote!(self.#i.#method_ident(rhs)))
        .collect()
}

pub fn struct_exprs(fields: &[&Field], method_ident: &Ident) -> Vec<TokenStream> {
    field_idents(fields)
        .iter()
        .map(|f| quote!(self.#f.#method_ident(rhs)))
        .collect()
}
