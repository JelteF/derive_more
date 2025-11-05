use std::iter;

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::parse_quote;

use crate::add_helpers::{struct_exprs_and_used_fields, tuple_exprs_and_used_fields};
use crate::utils::{
    field_idents, named_to_vec, numbered_vars, structural_inclusion::TypeExt as _,
    unnamed_to_vec, GenericsSearch,
};

pub fn expand(input: &syn::DeriveInput, trait_name: &str) -> syn::Result<TokenStream> {
    let trait_name = trait_name.trim_end_matches("Self");
    let trait_ident = format_ident!("{trait_name}");
    let method_name = trait_name.to_lowercase();
    let method_ident = format_ident!("{method_name}");
    let input_type = &input.ident;

    let (block, used_fields) = match &input.data {
        syn::Data::Struct(data_struct) => match &data_struct.fields {
            fields @ syn::Fields::Unnamed(_) => {
                let (exprs, used_fields) =
                    tuple_exprs_and_used_fields(fields, &method_ident)?;
                (quote! { #input_type(#(#exprs,)*) }, used_fields)
            }
            fields @ syn::Fields::Named(_) => {
                let (exprs, used_fields) =
                    struct_exprs_and_used_fields(fields, &method_ident)?;
                let field_names = fields.iter().filter_map(|f| f.ident.as_ref());
                (
                    quote! { #input_type{ #(#field_names: #exprs,)* } },
                    used_fields,
                )
            }
            _ => panic!("unit structs cannot use `derive({trait_name})`"),
        },
        syn::Data::Enum(data_enum) => (
            enum_content(input_type, data_enum, &method_ident),
            data_enum.variants.iter().flat_map(|v| &v.fields).collect(),
        ),
        syn::Data::Union(_) => {
            panic!("only structs and enums can use `derive({trait_name})`");
        }
    };

    let generics_search = GenericsSearch::from(&input.generics);
    let mut generics = input.generics.clone();
    let (_, ty_generics, _) = input.generics.split_for_impl();
    let implementor_ty: syn::Type = parse_quote! { #input_type #ty_generics };
    {
        let self_ty: syn::Type = parse_quote! { Self };
        for field_ty in used_fields.iter().map(|f| &f.ty) {
            if generics_search.any_in(field_ty)
                && !field_ty.contains_type_structurally(&self_ty)
                && !field_ty.contains_type_structurally(&implementor_ty)
            {
                generics.make_where_clause().predicates.push(parse_quote! {
                    #field_ty: derive_more::core::ops::#trait_ident
                });
            }
        }
    }
    let (impl_generics, _, where_clause) = generics.split_for_impl();

    let output_type = match input.data {
        syn::Data::Struct(_) => quote! { #implementor_ty },
        syn::Data::Enum(_) => quote! {
            derive_more::core::result::Result<#implementor_ty, derive_more::BinaryError>
        },
        _ => unreachable!(),
    };

    Ok(quote! {
        #[automatically_derived]
        impl #impl_generics derive_more::core::ops::#trait_ident
         for #input_type #ty_generics #where_clause {
            type Output = #output_type;

            #[inline]
            #[track_caller]
            fn #method_ident(self, rhs: #input_type #ty_generics) -> #output_type {
                #block
            }
        }
    })
}

#[allow(clippy::cognitive_complexity)]
fn enum_content(
    input_type: &syn::Ident,
    data_enum: &syn::DataEnum,
    method_ident: &syn::Ident,
) -> TokenStream {
    let mut matches = vec![];
    let mut method_iter = iter::repeat(method_ident);

    for variant in &data_enum.variants {
        let subtype = &variant.ident;
        let subtype = quote! { #input_type::#subtype };

        match variant.fields {
            syn::Fields::Unnamed(ref fields) => {
                // The pattern that is outputted should look like this:
                // (Subtype(left_vars), TypePath(right_vars)) => Ok(TypePath(exprs))
                let size = unnamed_to_vec(fields).len();
                let l_vars = &numbered_vars(size, "l_");
                let r_vars = &numbered_vars(size, "r_");
                let method_iter = method_iter.by_ref();
                let matcher = quote! {
                    (#subtype(#(#l_vars),*),
                     #subtype(#(#r_vars),*)) => {
                        derive_more::core::result::Result::Ok(
                            #subtype(#(#l_vars.#method_iter(#r_vars)),*)
                        )
                    }
                };
                matches.push(matcher);
            }
            syn::Fields::Named(ref fields) => {
                // The pattern that is outputted should look like this:
                // (Subtype{a: __l_a, ...}, Subtype{a: __r_a, ...} => {
                //     Ok(Subtype{a: __l_a.add(__r_a), ...})
                // }
                let field_vec = named_to_vec(fields);
                let size = field_vec.len();
                let field_names = &field_idents(&field_vec);
                let l_vars = &numbered_vars(size, "l_");
                let r_vars = &numbered_vars(size, "r_");
                let method_iter = method_iter.by_ref();
                let matcher = quote! {
                    (#subtype{#(#field_names: #l_vars),*},
                     #subtype{#(#field_names: #r_vars),*}) => {
                        derive_more::core::result::Result::Ok(#subtype{
                            #(#field_names: #l_vars.#method_iter(#r_vars)),*
                        })
                    }
                };
                matches.push(matcher);
            }
            syn::Fields::Unit => {
                let operation_name = method_ident.to_string();
                matches.push(quote! {
                    (#subtype, #subtype) => derive_more::core::result::Result::Err(
                        derive_more::BinaryError::Unit(
                            derive_more::UnitError::new(#operation_name)
                        )
                    )
                });
            }
        }
    }

    if data_enum.variants.len() > 1 {
        // In the strange case where there's only one enum variant this is would be an unreachable
        // match.
        let operation_name = method_ident.to_string();
        matches.push(quote! {
            _ => derive_more::core::result::Result::Err(derive_more::BinaryError::Mismatch(
                derive_more::WrongVariantError::new(#operation_name)
            ))
        });
    }
    quote! {
        match (self, rhs) {
            #(#matches),*
        }
    }
}
