use crate::utils::{AttrParams, DeriveType, State};
use convert_case::{Case, Casing};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{DeriveInput, Fields, Ident, Result, Type};

pub fn expand(input: &DeriveInput, trait_name: &'static str) -> Result<TokenStream> {
    let state = State::with_attr_params(
        input,
        trait_name,
        quote!(),
        String::from("try_into_variant"),
        AttrParams {
            enum_: vec!["ignore", "owned", "ref", "ref_mut"],
            variant: vec!["ignore", "owned", "ref", "ref_mut"],
            struct_: vec![],
            field: vec![],
        },
    )?;
    assert!(
        state.derive_type == DeriveType::Enum,
        "TryIntoVariant can only be derived for enums"
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
            "try_into_{ident}",
            ident = variant.ident.to_string().to_case(Case::Snake),
            span = variant.ident.span(),
        );
        let ref_fn_name = format_ident!(
            "try_into_{ident}_ref",
            ident = variant.ident.to_string().to_case(Case::Snake),
            span = variant.ident.span(),
        );
        let mut_fn_name = format_ident!(
            "try_into_{ident}_mut",
            ident = variant.ident.to_string().to_case(Case::Snake),
            span = variant.ident.span(),
        );
        let variant_ident = &variant.ident;
        let (data_patterns, data_types) = get_field_info(&variant.fields);
        let pattern = quote! { #enum_name :: #variant_ident (#(#data_patterns),*) };
        let ret_value = quote! { (#(#data_patterns),*) };

        let doc_owned = format!(
            "Attempts to convert this value to the `{enum_name}::{variant_ident}` variant.\n",
        );
        let doc_ref = format!(
            "Attempts to convert this reference to the `{enum_name}::{variant_ident}` variant.\n",
        );
        let doc_mut = format!(
            "Attempts to convert this mutable reference to the `{enum_name}::{variant_ident}` variant.\n",
        );
        let doc_else = "Returns the original value if this value is of any other type.";
        let func = quote! {
            #[inline]
            #[track_caller]
            #[doc = #doc_owned]
            #[doc = #doc_else]
            pub fn #fn_name(self) -> Result<(#(#data_types),*), Self> {
                match self {
                    #pattern => Ok(#ret_value),
                    val @ _ => Err(val),
                }
            }
        };

        let ref_func = quote! {
            #[inline]
            #[track_caller]
            #[doc = #doc_ref]
            #[doc = #doc_else]
            pub fn #ref_fn_name(&self) -> Result<(#(&#data_types),*), Self> {
                match self {
                    #pattern => Ok(#ret_value),
                    val @ _ => Err(val),
                }
            }
        };

        let mut_func = quote! {
            #[inline]
            #[track_caller]
            #[doc = #doc_mut]
            #[doc = #doc_else]
            pub fn #mut_fn_name(&mut self) -> Result<(#(&mut #data_types),*), Self> {
                match self {
                    #pattern => Ok(#ret_value),
                    val @ _ => Err(val),
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
        #[automatically_derived]
        impl #imp_generics #enum_name #type_generics #where_clause {
            #(#funcs)*
        }
    };

    Ok(imp)
}

fn get_field_info(fields: &Fields) -> (Vec<Ident>, Vec<&Type>) {
    match fields {
        Fields::Named(_) => panic!("cannot unwrap anonymous records"),
        Fields::Unnamed(ref fields) => fields
            .unnamed
            .iter()
            .enumerate()
            .map(|(n, it)| (format_ident!("field_{n}"), &it.ty))
            .unzip(),
        Fields::Unit => (vec![], vec![]),
    }
}
