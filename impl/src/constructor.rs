use crate::utils::{
    field_idents, get_field_types, named_to_vec, numbered_vars, unnamed_to_vec,
};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    parse::ParseStream, spanned::Spanned as _, Data, DeriveInput, Field, Fields, Ident,
};

/// Provides the hook to expand `#[derive(Constructor)]` into an implementation of `Constructor`
pub fn expand(input: &DeriveInput, _: &str) -> syn::Result<TokenStream> {
    let input_type = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let (fields, named) = match &input.data {
        Data::Struct(data_struct) => match &data_struct.fields {
            Fields::Unnamed(fields) => (unnamed_to_vec(fields), false),
            Fields::Named(fields) => (named_to_vec(fields), true),
            Fields::Unit => (vec![], true),
        },
        _ => {
            return Err(syn::Error::new_spanned(
                input,
                "only structs can derive a constructor",
            ))
        }
    };

    // Determine whether each field is annotated with `#[constructor(into)]`.
    let into_flags = fields
        .iter()
        .map(|field| field_is_into(field))
        .collect::<syn::Result<Vec<_>>>()?;

    // Any `Into` fields will prevent `new()` from being constant
    let has_into = into_flags.iter().any(|&into| into);

    let constness = if has_into {
        quote! {}
    } else {
        quote! { const }
    };

    // Parameter names: the field names for named structs, `__0`, `__1`, ... otherwise.
    let types = get_field_types(&fields);

    let vars: Vec<Ident> = if named {
        field_idents(&fields)
            .iter()
            .map(|ident| (*ident).clone())
            .collect()
    } else {
        numbered_vars(fields.len(), "")
    };

    // This is where `impl Into<T>` is derived for all fields that have `#[constructor(into)]`
    // attached. Otherwise, use raw type
    let args = vars
        .iter()
        .zip(&types)
        .zip(&into_flags)
        .map(|((var, ty), &into)| {
            if into {
                quote! { #var: impl ::core::convert::Into<#ty> }
            } else {
                quote! { #var: #ty }
            }
        });

    // Field initializer expressions, calling `.into()` for `#[constructor(into)]` fields.
    let exprs: Vec<TokenStream> = vars
        .iter()
        .zip(&into_flags)
        .map(|(var, &into)| {
            if into {
                quote! { ::core::convert::Into::into(#var) }
            } else {
                quote! { #var }
            }
        })
        .collect();

    let body = if named {
        quote! { #input_type { #(#vars: #exprs),* } }
    } else {
        quote! { #input_type(#(#exprs),*) }
    };

    let inherited_lint_attrs = input.attrs.iter().filter(|attr| {
        attr.path()
            .get_ident()
            .is_some_and(|name| name == "allow" || name == "expect")
    });

    Ok(quote! {
        #[allow(deprecated)]       // omit warnings on deprecated fields/variants
        #[allow(missing_docs)]
        #[allow(unreachable_code)] // omit warnings for `!` and other unreachable types
        #(#inherited_lint_attrs)*  // proxy-pass any `#[allow]`/`#[expect]` attributes
        #[automatically_derived]
        impl #impl_generics #input_type #ty_generics #where_clause {
            #[inline]
            pub #constness fn new(#(#args),*) -> #input_type #ty_generics {
                #body
            }
        }
    })
}

/// Parses a field's attributes, returning whether it is annotated with
/// `#[constructor(into)]`.
fn field_is_into(field: &Field) -> syn::Result<bool> {
    let mut is_into = false;
    for attr in &field.attrs {
        if !attr.path().is_ident("constructor") {
            continue;
        }
        attr.parse_args_with(|input: ParseStream<'_>| {
            let path: syn::Path = input.parse()?;
            if !path.is_ident("into") {
                return Err(syn::Error::new(
                    path.span(),
                    "only `into` is supported by `#[constructor(...)]`",
                ));
            }
            if is_into {
                return Err(syn::Error::new(
                    path.span(),
                    "only single `#[constructor(into)]` attribute is allowed here",
                ));
            }
            is_into = true;
            Ok(())
        })?;
    }
    Ok(is_into)
}
