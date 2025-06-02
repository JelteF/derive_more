use crate::utils::{AttrParams, DeriveType, State};
use convert_case::{Case, Casing};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{DeriveInput, Fields, Result, Type};

pub fn expand(input: &DeriveInput, trait_name: &'static str) -> Result<TokenStream> {
    let state = State::with_attr_params(
        input,
        trait_name,
        "as_variant".into(),
        AttrParams {
            enum_: vec!["ignore", "owned", "ref", "ref_mut"],
            variant: vec!["ignore", "owned", "ref", "ref_mut"],
            struct_: vec!["ignore"],
            field: vec!["ignore"],
        },
    )?;
    assert!(
        state.derive_type == DeriveType::Enum,
        "AsVariant can only be derived for enums",
    );

    let enum_name = &input.ident;
    let (imp_generics, type_generics, where_clause) = input.generics.split_for_impl();

    let variant_data = state.enabled_variant_data();

    let mut funcs = vec![];
    for (variant_state, info) in
        Iterator::zip(variant_data.variant_states.iter(), variant_data.infos)
    {
        let variant = variant_state.variant.unwrap();
        let fn_name = format_ident!(
            "as_{}",
            variant.ident.to_string().to_case(Case::Snake),
            span = variant.ident.span(),
        );
        let ref_fn_name = format_ident!(
            "as_{}_ref",
            variant.ident.to_string().to_case(Case::Snake),
            span = variant.ident.span(),
        );
        let mut_fn_name = format_ident!(
            "as_{}_mut",
            variant.ident.to_string().to_case(Case::Snake),
            span = variant.ident.span(),
        );
        let variant_ident = &variant.ident;

        let (data_pattern, ret_value, data_types) = get_field_info(&variant.fields);
        let pattern = quote! { #enum_name :: #variant_ident #data_pattern };
        let doc_owned = format!(
            "Attempts to convert this value to the `{enum_name}::{variant_ident}` variant.\n",
        );
        let doc_ref = format!(
            "Attempts to convert this reference to the `{enum_name}::{variant_ident}` variant.\n",
        );
        let doc_mut = format!(
            "Attempts to convert this mutable reference to the `{enum_name}::{variant_ident}` variant.\n",
        );
        let doc_else = "Returns Some(..) if successful and None if this value is of any other type.";
        let func = quote! {
            #[doc = #doc_owned]
            #[doc = #doc_else]
            #[inline]
            #[must_use]
            pub fn #fn_name(self) -> Option<(#(#data_types),*)> {
                match self {
                    #pattern => Some(#ret_value),
                    _ => None
                }
            }
        };
        let ref_func = quote! {
            #[doc = #doc_ref]
            #[doc = #doc_else]
            #[inline]
            #[must_use]
            pub fn #ref_fn_name(&self) -> Option<(#(&#data_types),*)> {
                match self {
                    #pattern => Some(#ret_value),
                    _ => None
                }
            }
        };
        let mut_func = quote! {
            #[doc = #doc_mut]
            #[doc = #doc_else]
            #[inline]
            #[must_use]
            pub fn #mut_fn_name(&mut self) -> Option<(#(&mut #data_types),*)> {
                match self {
                    #pattern => Some(#ret_value),
                    _ => None
                }
            }
        };
        if info.owned && state.default_info.owned {
            funcs.push(func);
        }
        if info.ref_ && state.default_info.ref_ {
            funcs.push(ref_func);
        }
        if info.ref_mut && state.default_info.ref_mut {
            funcs.push(mut_func);
        }
    }

    let imp = quote! {
        #[allow(unreachable_code)] // omit warnings for `!` and other unreachable types
        #[automatically_derived]
        impl #imp_generics #enum_name #type_generics #where_clause {
            #(#funcs)*
        }
    };

    Ok(imp)
}

fn get_field_info(fields: &Fields) -> (TokenStream, TokenStream, Vec<&Type>) {
    match fields {
        Fields::Named(_) => panic!("cannot extract anonymous records in as_variant"),
        Fields::Unnamed(ref fields) => {
            let (idents, types) = fields
                .unnamed
                .iter()
                .enumerate()
                .map(|(n, it)| (format_ident!("field_{n}"), &it.ty))
                .unzip::<_, _, Vec<_>, Vec<_>>();
            (quote! { (#(#idents),*) }, quote! { (#(#idents),*) }, types)
        }
        Fields::Unit => (quote! {}, quote! { () }, vec![]),
    }
}
