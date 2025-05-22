use proc_macro2::TokenStream;
use quote::quote;
use syn::{Field, Ident, Index, PathArguments, Type};

pub fn tuple_exprs(fields: &[&Field], method_ident: &Ident) -> Vec<TokenStream> {
    let mut exprs = vec![];

    for (i, field) in fields.iter().enumerate() {
        let index = Index::from(i);
        let expr = match field.attrs.iter().any(|a| a.path().is_ident("skip")) {
            true => match &field.ty {
                Type::Path(path) => {
                    let mut ty = path.path.segments.clone();
                    ty.last_mut().unwrap().arguments = PathArguments::None;
                    quote! { #ty }
                }
                ty => quote! { #ty },
            },
            // generates `self.0.add(rhs.0)` for fields not marked with `#[skip]`
            false => quote! { self.#index.#method_ident(rhs.#index) },
        };
        exprs.push(expr);
    }
    exprs
}

pub fn struct_exprs(fields: &[&Field], method_ident: &Ident) -> Vec<TokenStream> {
    let mut exprs = vec![];

    for field in fields {
        // It's safe to unwrap because struct fields always have an identifier
        let field_id = field.ident.as_ref().unwrap();
        let expr = match field.attrs.iter().any(|a| a.path().is_ident("skip")) {
            true => match &field.ty {
                Type::Path(path) => {
                    let mut ty = path.path.segments.clone();
                    ty.last_mut().unwrap().arguments = PathArguments::None;
                    quote! { #ty }
                }
                ty => quote! { #ty },
            },
            // generates `x: self.x.add(rhs.x)` for fields not marked with `#[skip]`
            false => quote! { self.#field_id.#method_ident(rhs.#field_id) },
        };
        exprs.push(expr)
    }
    exprs
}
