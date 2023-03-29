use crate::utils::{AttrParams, DeriveType, State};
use convert_case::{Case, Casing};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{DeriveInput, Fields, Ident, Result};

pub fn expand(input: &DeriveInput, trait_name: &'static str) -> Result<TokenStream> {
    let state = State::with_attr_params(
        input,
        trait_name,
        quote! {},
        "unwrap".into(),
        AttrParams {
            enum_: vec!["ignore", "owned", "ref", "ref_mut"],
            variant: vec!["ignore", "owned", "ref", "ref_mut"],
            struct_: vec!["ignore"],
            field: vec!["ignore"],
        },
    )?;
    assert!(
        state.derive_type == DeriveType::Enum,
        "Unwrap can only be derived for enums",
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
            "unwrap_{ident}",
            ident = variant.ident.to_string().to_case(Case::Snake),
            span = variant.ident.span(),
        );
        let ref_fn_name = format_ident!(
            "unwrap_{ident}_ref",
            ident = variant.ident.to_string().to_case(Case::Snake),
            span = variant.ident.span(),
        );
        let mut_fn_name = format_ident!(
            "unwrap_{ident}_mut",
            ident = variant.ident.to_string().to_case(Case::Snake),
            span = variant.ident.span(),
        );
        let variant_ident = &variant.ident;

        let (data_pattern, ret_value, ret_type, ret_type_ref, ret_type_mut) = match variant.fields {
            Fields::Named(_) => panic!("cannot unwrap anonymous records"),
            Fields::Unnamed(ref fields) => {
                let (data_pattern, ret_types): (Vec<_>, Vec<_>) = fields
                    .unnamed
                    .iter()
                    .enumerate()
                    .map(|(n, it)| (format_ident!("field_{n}"), &it.ty))
                    .unzip();

                (
                    quote! { (#(#data_pattern),*) },
                    quote! { (#(#data_pattern),*) },
                    quote! { (#(#ret_types),*) },
                    quote! { (#(&#ret_types),*) },
                    quote! { (#(&mut #ret_types),*) },
                )
            }
            Fields::Unit => (quote! {}, quote! { () }, quote! { () }, quote! { () }, quote! { () }),
        };

        let pattern = quote! { #enum_name :: #variant_ident #data_pattern };

        let (failed_block, failed_block_ref, failed_block_mut) = (
            failed_block(&state, enum_name, &fn_name),
            failed_block(&state, enum_name, &ref_fn_name),
            failed_block(&state, enum_name, &mut_fn_name),
        );

        let variant_name = stringify!(variant_ident);
        let func = quote! {
            #[track_caller]
            #[doc = "Unwraps this value to the `"]
            #[doc = #variant_name]
            #[doc = "` variant.\n"]
            #[doc = "Panics if this value is of any other type."]
            pub fn #fn_name(self) -> #ret_type {
                match self {
                    #pattern => #ret_value,
                    val @ _ => #failed_block,
                }
            }
        };

        let ref_func = quote! {
            #[track_caller]
            #[doc = "Unwraps this reference to the `"]
            #[doc = #variant_name]
            #[doc = "` variant.\n"]
            #[doc = "Panics if this value is of any other type."]
            pub fn #ref_fn_name(&self) -> #ret_type_ref {
                match self {
                    #pattern => #ret_value,
                    val @ _ => #failed_block_ref,
                }
            }
        };

        let mut_func = quote! {
            #[track_caller]
            #[doc = "Unwraps this mutable reference to the `"]
            #[doc = #variant_name]
            #[doc = "` variant.\n"]
            #[doc = "Panics if this value is of any other type."]
            pub fn #mut_fn_name(&mut self) -> #ret_type_mut {
                match self {
                    #pattern => #ret_value,
                    val @ _ => #failed_block_mut,
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

fn failed_block(state: &State, enum_name: &Ident, fn_name: &Ident) -> TokenStream {
    let arms = state
        .variant_states
        .iter()
        .map(|it| it.variant.unwrap())
        .map(|variant| {
            let data_pattern = match variant.fields {
                Fields::Named(_) => quote! { {..} },
                Fields::Unnamed(_) => quote! { (..) },
                Fields::Unit => quote! {},
            };
            let variant_ident = &variant.ident;
            quote! {
                #enum_name :: #variant_ident #data_pattern => panic!(concat!(
                    "called `", stringify!(#enum_name), "::", stringify!(#fn_name), "()` ",
                    "on a `", stringify!(#variant_ident), "` value"
                ))
            }
        });

    quote! {
        match val {
            #(#arms),*
        }
    }
}
