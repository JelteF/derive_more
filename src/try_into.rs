use crate::utils::{RefType, field_idents, named_to_vec, numbered_vars, unnamed_to_vec, add_extra_generic_param};
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use std::collections::HashMap;
use syn::{Data, DataEnum, DeriveInput, Fields};

/// Provides the hook to expand `#[derive(TryInto)]` into an implementation of `TryInto`
pub fn expand(input: &DeriveInput, trait_name: &str) -> TokenStream {
    match input.data {
        Data::Enum(ref data_enum) => enum_try_into(input, data_enum, trait_name),
        _ => panic!("Only enums can derive TryInto"),
    }
}

#[allow(clippy::cognitive_complexity)]
fn enum_try_into(input: &DeriveInput, data_enum: &DataEnum, trait_name: &str) -> TokenStream {
    let mut variants_per_types = HashMap::new();
    let (ref_type, _) = RefType::from_derive(trait_name);
    let pattern_ref = ref_type.pattern_ref();
    let lifetime = ref_type.lifetime();
    let reference_with_lifetime = ref_type.reference_with_lifetime();

    let generics_impl;
    let (_, ty_generics, where_clause) = input.generics.split_for_impl();
    let (impl_generics, _, _) = if ref_type.is_ref() {
        generics_impl = add_extra_generic_param(&input.generics, lifetime.clone());
        generics_impl.split_for_impl()
    } else {
        input.generics.split_for_impl()
    };
    let input_type = &input.ident;

    for variant in &data_enum.variants {
        let original_types = match variant.fields {
            Fields::Unnamed(ref fields) => unnamed_to_vec(fields).iter().map(|f| &f.ty).collect(),
            Fields::Named(ref fields) => named_to_vec(fields).iter().map(|f| &f.ty).collect(),
            Fields::Unit => vec![],
        };
        variants_per_types
            .entry(original_types)
            .or_insert_with(Vec::new)
            .push(variant);
    }

    let mut tokens = TokenStream::new();

    for (ref original_types, ref variants) in variants_per_types {
        let mut matchers = vec![];
        let vars = &numbered_vars(original_types.len(), "");
        for variant in variants.iter() {
            let subtype = &variant.ident;
            let subtype = quote!(#input_type::#subtype);
            matchers.push(match variant.fields {
                Fields::Unnamed(_) => quote!(#subtype(#(#pattern_ref #vars),*)),
                Fields::Named(ref fields) => {
                    let field_vec = &named_to_vec(fields);
                    let field_names = &field_idents(field_vec);
                    quote!(#subtype{#(#field_names: #pattern_ref #vars),*})
                }
                Fields::Unit => quote!(#subtype),
            });
        }

        let vars = if vars.len() == 1 {
            quote!(#(#vars)*)
        } else {
            quote!((#(#vars),*))
        };

        let output_type = if original_types.len() == 1 {
            format!("{}", quote!(#(#original_types)*))
        } else {
            let types = original_types
                .iter()
                .map(|t| format!("{}", quote!(#t)))
                .collect::<Vec<_>>();
            format!("({})", types.join(", "))
        };
        let variants = variants
            .iter()
            .map(|v| format!("{}", v.ident))
            .collect::<Vec<_>>()
            .join(", ");
        let message = format!("Only {} can be converted to {}", variants, output_type);

        let try_from = quote! {
            impl#impl_generics ::core::convert::TryFrom<#reference_with_lifetime #input_type#ty_generics> for
                (#(#reference_with_lifetime #original_types),*) #where_clause {
                type Error = &'static str;

                #[allow(unused_variables)]
                #[inline]
                fn try_from(value: #reference_with_lifetime #input_type#ty_generics) -> ::core::result::Result<Self, Self::Error> {
                    match value {
                        #(#matchers)|* => ::core::result::Result::Ok(#vars),
                        _ => ::core::result::Result::Err(#message),
                    }
                }
            }
        };
        try_from.to_tokens(&mut tokens)
    }
    tokens
}
