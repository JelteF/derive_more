use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{Field, Ident, Index, Type, TypeArray, TypeTuple};

pub fn tuple_exprs(fields: &[&Field], method_ident: &Ident) -> Vec<TokenStream> {
    let fields: Vec<&Type> = fields.iter().map(|field| &field.ty).collect::<Vec<_>>();
    inner_tuple_exprs(0, &quote! {}, &fields, method_ident)
}

pub fn struct_exprs(fields: &[&Field], method_ident: &Ident) -> Vec<TokenStream> {
    fields
        .iter()
        .map(|field| {
            // It's safe to unwrap because struct fields always have an identifier
            let field_path = field.ident.as_ref().unwrap();
            elem_content(0, &quote! { .#field_path }, &field.ty, method_ident)
        })
        .collect()
}

pub fn inner_tuple_exprs(
    // `depth` is needed for `index_var` generation for nested arrays
    depth: usize,
    field_path: &TokenStream,
    fields: &[&Type],
    method_ident: &Ident,
) -> Vec<TokenStream> {
    fields
        .iter()
        .enumerate()
        .map(|(i, ty)| {
            let i = Index::from(i);
            elem_content(depth + 1, &quote! { #field_path.#i }, ty, method_ident)
        })
        .collect()
}

pub fn elem_content(
    depth: usize,
    field_path: &TokenStream,
    ty: &Type,
    method_ident: &Ident,
) -> TokenStream {
    match ty {
        Type::Array(TypeArray { elem, .. }) => {
            let index_var = Ident::new(&format!("i{}", depth), Span::call_site());
            let fn_body = elem_content(
                depth + 1,
                &quote! { #field_path[#index_var] },
                elem.as_ref(),
                method_ident,
            );

            // generates `core::array::from_fn(|i0| self.x[i0].add(rhs.x[i0]))`
            quote! { core::array::from_fn(|#index_var| #fn_body) }
        }
        Type::Tuple(TypeTuple { elems, .. }) => {
            let exprs = inner_tuple_exprs(
                depth + 1,
                field_path,
                &elems.iter().collect::<Vec<_>>(),
                method_ident,
            );
            quote! { (#(#exprs),*) }
        }
        // generates `self.x.add(rhs.x)`
        _ => quote! { self #field_path.#method_ident(rhs #field_path) },
    }
}
