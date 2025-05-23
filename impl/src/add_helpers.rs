use std::ops::BitOr;

use proc_macro2::TokenStream;
use quote::quote;
use syn::{Field, Ident, Index, PathArguments};

use crate::utils::{
    attr::{ParseMultiple, Skip},
    extract_idents_from_generic_arguments, HashSet,
};

#[cfg(any(feature = "add", feature = "mul",))]
pub fn tuple_exprs(
    fields: &[&Field],
    method_ident: &Ident,
) -> syn::Result<(Vec<TokenStream>, HashSet<Ident>)> {
    let mut exprs = vec![];
    let mut zst_generics = HashSet::default();

    for (i, field) in fields.iter().enumerate() {
        let index = Index::from(i);
        let expr = match Skip::parse_attrs(&field.attrs, method_ident)? {
            Some(_) => match &field.ty {
                syn::Type::Path(path) => {
                    let mut ty = path.path.segments.clone();
                    if let PathArguments::AngleBracketed(args) =
                        &ty.last_mut().unwrap().arguments
                    {
                        let extracted =
                            extract_idents_from_generic_arguments(&args.args);
                        zst_generics = zst_generics.bitor(&extracted);
                    }
                    ty.last_mut().unwrap().arguments = syn::PathArguments::None;
                    quote! { #ty }
                }
                ty => quote! { #ty },
            },
            // generates `self.0.add(rhs.0)` for fields not marked with `#[skip]`
            None => quote! { self.#index.#method_ident(rhs.#index) },
        };
        exprs.push(expr);
    }
    Ok((exprs, zst_generics))
}

#[cfg(any(feature = "add", feature = "mul",))]
pub fn struct_exprs(
    fields: &[&Field],
    method_ident: &Ident,
) -> syn::Result<(Vec<TokenStream>, HashSet<Ident>)> {
    let mut exprs = vec![];
    let mut zst_generics = HashSet::default();

    for field in fields {
        // It's safe to unwrap because struct fields always have an identifier
        let field_id = field.ident.as_ref().unwrap();
        let expr = match Skip::parse_attrs(&field.attrs, method_ident)? {
            Some(_) => match &field.ty {
                syn::Type::Path(path) => {
                    let mut ty = path.path.segments.clone();
                    if let PathArguments::AngleBracketed(args) =
                        &ty.last_mut().unwrap().arguments
                    {
                        let extracted =
                            extract_idents_from_generic_arguments(&args.args);
                        zst_generics = zst_generics.bitor(&extracted);
                    }
                    ty.last_mut().unwrap().arguments = syn::PathArguments::None;
                    quote! { #ty }
                }
                ty => quote! { #ty },
            },
            // generates `self.x.add(rhs.x)` for fields not marked with `#[skip]`
            None => quote! { self.#field_id.#method_ident(rhs.#field_id) },
        };
        exprs.push(expr)
    }
    Ok((exprs, zst_generics))
}

#[cfg(any(feature = "add_assign", feature = "mul_assign",))]
pub fn tuple_assign_exprs(
    fields: &[&Field],
    method_ident: &Ident,
) -> syn::Result<(Vec<TokenStream>, HashSet<Ident>)> {
    let mut exprs = vec![];
    let mut zst_generics = HashSet::default();

    for (i, field) in fields.iter().enumerate() {
        let index = Index::from(i);
        match Skip::parse_attrs(&field.attrs, method_ident)? {
            Some(_) => {
                if let syn::Type::Path(path) = &field.ty {
                    let mut ty = path.path.segments.clone();
                    if let PathArguments::AngleBracketed(args) =
                        &ty.last_mut().unwrap().arguments
                    {
                        let extracted =
                            extract_idents_from_generic_arguments(&args.args);
                        zst_generics = zst_generics.bitor(&extracted);
                    }
                }
            }
            // generates `self.0.add_assign(rhs.0)` for fields not marked with `#[skip]`
            None => exprs.push(quote! { self.#index.#method_ident(rhs.#index) }),
        }
    }
    Ok((exprs, zst_generics))
}

#[cfg(any(feature = "add_assign", feature = "mul_assign",))]
pub fn struct_assign_exprs(
    fields: &[&Field],
    method_ident: &Ident,
) -> syn::Result<(Vec<TokenStream>, HashSet<Ident>)> {
    let mut exprs = vec![];
    let mut zst_generics = HashSet::default();

    for field in fields {
        // It's safe to unwrap because struct fields always have an identifier
        let field_id = field.ident.as_ref().unwrap();
        match Skip::parse_attrs(&field.attrs, method_ident)? {
            Some(_) => {
                if let syn::Type::Path(path) = &field.ty {
                    let mut ty = path.path.segments.clone();
                    if let PathArguments::AngleBracketed(args) =
                        &ty.last_mut().unwrap().arguments
                    {
                        let extracted =
                            extract_idents_from_generic_arguments(&args.args);
                        zst_generics = zst_generics.bitor(&extracted);
                    }
                }
            }
            // generates `self.x.add_assign(rhs.x)` for fields not marked with `#[skip]`
            None => exprs.push(quote! { self.#field_id.#method_ident(rhs.#field_id) }),
        }
    }
    Ok((exprs, zst_generics))
}
