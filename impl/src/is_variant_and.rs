use crate::utils::{AttrParams, DeriveType, State};
use convert_case::{Case, Casing};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{DeriveInput, Fields, Result};

pub fn expand(input: &DeriveInput, trait_name: &'static str) -> Result<TokenStream> {
    let state = State::with_attr_params(
        input,
        trait_name,
        "is_variant_and".into(),
        AttrParams {
            enum_: vec!["ignore"],
            variant: vec!["ignore"],
            struct_: vec!["ignore"],
            field: vec!["ignore"],
        },
    )?;
    assert!(
        state.derive_type == DeriveType::Enum,
        "IsVariantAnd can only be derived for enums",
    );

    let enum_name = &input.ident;
    let (imp_generics, type_generics, where_clause) = input.generics.split_for_impl();

    let mut funcs = vec![];
    for variant_state in state.enabled_variant_data().variant_states {
        let variant = variant_state.variant.unwrap();
        let variant_ident = &variant.ident;
        let fn_name = format_ident!(
            "is_{}_and",
            variant_ident.to_string().to_case(Case::Snake),
            span = variant_ident.span(),
        );

        let (pattern, bindings, arg_type) = field_info(&variant.fields);
        let pattern = quote! { #enum_name ::#variant_ident #pattern };

        let doc = format!(
            "Returns `true` if this value is of type `{variant_ident}` and the \
             given closure returns `true` for the contained value(s).\n\nReturns \
             `false` otherwise.",
        );

        let func = if let Some(arg_type) = arg_type {
            quote! {
                #[doc = #doc]
                #[inline]
                #[must_use]
                pub fn #fn_name(
                    &self,
                    f: impl derive_more::core::ops::FnOnce(#arg_type) -> bool,
                ) -> bool {
                    match self {
                        #pattern => f(#bindings),
                        _ => false,
                    }
                }
            }
        } else {
            quote! {
                #[doc = #doc]
                #[inline]
                #[must_use]
                pub fn #fn_name(
                    &self,
                    f: impl derive_more::core::ops::FnOnce() -> bool,
                ) -> bool {
                    match self {
                        #pattern => f(),
                        _ => false,
                    }
                }
            }
        };
        funcs.push(func);
    }

    let imp = quote! {
        #[allow(deprecated)] // omit warnings on deprecated fields/variants
        #[allow(unreachable_code)] // omit warnings for `!` and other unreachable types
        #[allow(unreachable_patterns)] // omit warnings for single-variant enums
        #[automatically_derived]
        impl #imp_generics #enum_name #type_generics #where_clause {
            #(#funcs)*
        }
    };

    Ok(imp)
}

/// Returns the pattern binding the variant's fields, the tuple of bound field
/// references passed to the closure, and the closure's argument type.
///
/// The argument type is `None` for unit variants, whose generated method takes a
/// `FnOnce() -> bool` closure instead. For a variant with a single field the
/// resulting type collapses to a plain reference (e.g. `&T`), mirroring
/// [`Option::is_some_and`].
fn field_info(fields: &Fields) -> (TokenStream, TokenStream, Option<TokenStream>) {
    match fields {
        Fields::Named(fields) if !fields.named.is_empty() => {
            let (patterns, (bindings, types)): (Vec<_>, (Vec<_>, Vec<_>)) = fields
                .named
                .iter()
                .enumerate()
                .map(|(n, field)| {
                    let name = field.ident.as_ref().unwrap();
                    let binding = format_ident!("__field_{n}");
                    (quote! { #name: #binding }, (binding, &field.ty))
                })
                .unzip();
            (
                quote! { { #(#patterns),* } },
                quote! { (#(#bindings),*) },
                Some(quote! { (#(&#types),*) }),
            )
        }
        Fields::Unnamed(fields) if !fields.unnamed.is_empty() => {
            let (bindings, types): (Vec<_>, Vec<_>) = fields
                .unnamed
                .iter()
                .enumerate()
                .map(|(n, field)| (format_ident!("__field_{n}"), &field.ty))
                .unzip();
            (
                quote! { (#(#bindings),*) },
                quote! { (#(#bindings),*) },
                Some(quote! { (#(&#types),*) }),
            )
        }
        // Variants with no fields (unit, empty tuple `()` or empty record `{}`)
        // all take a `FnOnce() -> bool` closure.
        Fields::Named(_) => (quote! { {} }, quote! {}, None),
        Fields::Unnamed(_) => (quote! { () }, quote! {}, None),
        Fields::Unit => (quote! {}, quote! {}, None),
    }
}
