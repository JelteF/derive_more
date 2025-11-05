use proc_macro2::TokenStream;
use quote::quote;

use crate::utils::attr::{ParseMultiple as _, Skip};

pub(crate) fn tuple_exprs_and_used_fields<'f>(
    fields: impl IntoIterator<Item = &'f syn::Field>,
    method_ident: &syn::Ident,
) -> syn::Result<(Vec<TokenStream>, Vec<&'f syn::Field>)> {
    let (mut exprs, mut used_fields) = (vec![], vec![]);
    for (i, field) in fields.into_iter().enumerate() {
        let index = syn::Index::from(i);
        exprs.push(
            if Skip::parse_attrs(&field.attrs, method_ident)?.is_some() {
                quote! { self.#index }
            } else {
                used_fields.push(field);
                quote! { self.#index.#method_ident(rhs.#index) }
            },
        );
    }
    Ok((exprs, used_fields))
}

pub fn struct_exprs_and_used_fields<'f>(
    fields: impl IntoIterator<Item = &'f syn::Field>,
    method_ident: &syn::Ident,
) -> syn::Result<(Vec<TokenStream>, Vec<&'f syn::Field>)> {
    let (mut exprs, mut used_fields) = (vec![], vec![]);
    for field in fields {
        // PANIC: OK, because struct fields are always named.
        let field_name = field.ident.as_ref().unwrap();
        exprs.push(
            if Skip::parse_attrs(&field.attrs, method_ident)?.is_some() {
                quote! { self.#field_name }
            } else {
                used_fields.push(field);
                quote! { self.#field_name.#method_ident(rhs.#field_name) }
            },
        );
    }
    Ok((exprs, used_fields))
}
