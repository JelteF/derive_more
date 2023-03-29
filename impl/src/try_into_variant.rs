use crate::utils::{AttrParams, DeriveType, State};
use convert_case::{Case, Casing};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{DeriveInput, Fields, Result};

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

        let (data_pattern, ret_value, ret_type, ret_type_ref, ret_type_mut) =
            match variant.fields {
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
                Fields::Unit => (
                    quote! {},
                    quote! { () },
                    quote! { () },
                    quote! { () },
                    quote! { () },
                ),
            };

        let pattern = quote! { #enum_name :: #variant_ident #data_pattern };

        let variant_name = stringify!(variant_ident);
        let func = quote! {
            #[track_caller]
            #[doc = "Attempts to convert this value to the `"]
            #[doc = #variant_name]
            #[doc = "` variant.\n"]
            #[doc = "Returns the original value if this value is of any other type."]
            pub fn #fn_name(self) -> Result<#ret_type, Self> {
                match self {
                    #pattern => Ok(#ret_value),
                    val @ _ => Err(val),
                }
            }
        };

        let ref_func = quote! {
            #[track_caller]
            #[doc = "Attempts to convert this reference to the `"]
            #[doc = #variant_name]
            #[doc = "` variant.\n"]
            #[doc = "Returns the original value if this value is of any other type."]
            pub fn #ref_fn_name(&self) -> Result<#ret_type_ref, Self> {
                match self {
                    #pattern => Ok(#ret_value),
                    val @ _ => Err(val),
                }
            }
        };

        let mut_func = quote! {
            #[track_caller]
            #[doc = "Attempts to convert this mutable reference to the `"]
            #[doc = #variant_name]
            #[doc = "` variant.\n"]
            #[doc = "Returns the original value if this value is of any other type."]
            pub fn #mut_fn_name(&mut self) -> Result<#ret_type_mut, Self> {
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
