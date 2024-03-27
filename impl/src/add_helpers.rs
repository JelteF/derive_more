use proc_macro2::TokenStream;
use quote::quote;
use syn::{Field, Ident, Index, Type, TypeArray, TypeTuple};

pub fn tuple_exprs(fields: &[&Field], method_ident: &Ident) -> Vec<TokenStream> {
    let fields: Vec<&Type> = fields.iter().map(|field| &field.ty).collect::<Vec<_>>();
    inner_tuple_exprs(&quote! {}, &fields, method_ident)
}

pub fn struct_exprs(fields: &[&Field], method_ident: &Ident) -> Vec<TokenStream> {
    fields
        .iter()
        .map(|field| {
            // It's safe to unwrap because struct fields always have an identifier
            let field_path = field.ident.as_ref().unwrap();
            elem_content(&quote! { .#field_path }, &field.ty, method_ident)
        })
        .collect()
}

pub fn inner_tuple_exprs(
    field_path: &TokenStream,
    fields: &[&Type],
    method_ident: &Ident,
) -> Vec<TokenStream> {
    fields
        .iter()
        .enumerate()
        .map(|(i, ty)| {
            let i = Index::from(i);
            elem_content(&quote! { #field_path.#i }, ty, method_ident)
        })
        .collect()
}

pub fn elem_content(
    field_path: &TokenStream,
    ty: &Type,
    method_ident: &Ident,
) -> TokenStream {
    match ty {
        Type::Array(TypeArray { elem, .. }) => {
            let fn_body = elem_content(&quote! {}, elem.as_ref(), method_ident);

            quote! {
                lhs #field_path.into_iter().zip(rhs #field_path.into_iter())
                .map(|(lhs, rhs)| #fn_body)
                .collect::<Vec<_>>()
                .try_into()
                .unwrap_or_else(|_| unreachable!("Lengths should always match."))
            }
        }
        Type::Tuple(TypeTuple { elems, .. }) => {
            let exprs = inner_tuple_exprs(
                field_path,
                &elems.iter().collect::<Vec<_>>(),
                method_ident,
            );
            quote! { (#(#exprs),*) }
        }
        // generates `lhs.x.add(rhs.x)`
        _ => quote! { lhs #field_path.#method_ident(rhs #field_path) },
    }
}
