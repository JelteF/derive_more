use crate::add_helpers::{struct_exprs, tuple_exprs};
use crate::utils::{
    add_extra_type_param_bound_op_output_except, field_idents, named_to_vec,
    numbered_vars, unnamed_to_vec, HashSet,
};
use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};
use std::iter;
use syn::{Data, DataEnum, DeriveInput, Field, Fields, Ident};

pub fn expand(input: &DeriveInput, trait_name: &str) -> syn::Result<TokenStream> {
    let trait_name = trait_name.trim_end_matches("Self");
    let trait_ident = format_ident!("{trait_name}");
    let method_name = trait_name.to_lowercase();
    let method_ident = format_ident!("{method_name}");
    let input_type = &input.ident;

    let (block, zst_generics) = match &input.data {
        Data::Struct(data_struct) => match &data_struct.fields {
            Fields::Unnamed(ref fields) => {
                tuple_content(input_type, &unnamed_to_vec(fields), &method_ident)?
            }
            Fields::Named(ref fields) => {
                struct_content(input_type, &named_to_vec(fields), &method_ident)?
            }
            _ => panic!("Unit structs cannot use derive({trait_name})"),
        },
        Data::Enum(data_enum) => (
            enum_content(input_type, data_enum, &method_ident),
            Default::default(),
        ),
        Data::Union(_) => panic!("Only structs and enums can use derive({trait_name})"),
    };

    let generics = add_extra_type_param_bound_op_output_except(
        &input.generics,
        &trait_ident,
        zst_generics,
    );
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let output_type = match input.data {
        Data::Struct(_) => quote! { #input_type #ty_generics },
        Data::Enum(_) => quote! {
            derive_more::core::result::Result<#input_type #ty_generics, derive_more::BinaryError>
        },
        _ => panic!("Only structs and enums can use derive({trait_name})"),
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

fn tuple_content<T: ToTokens>(
    input_type: &T,
    fields: &[&Field],
    method_ident: &Ident,
) -> syn::Result<(TokenStream, HashSet<Ident>)> {
    let (exprs, zst_generics) = tuple_exprs(fields, method_ident)?;
    Ok((quote! { #input_type(#(#exprs),*) }, zst_generics))
}

fn struct_content(
    input_type: &Ident,
    fields: &[&Field],
    method_ident: &Ident,
) -> syn::Result<(TokenStream, HashSet<Ident>)> {
    // It's safe to unwrap because struct fields always have an identifier
    let (exprs, zst_generics) = struct_exprs(fields, method_ident)?;
    let field_names = field_idents(fields);

    Ok((
        quote! { #input_type{#(#field_names: #exprs),*} },
        zst_generics,
    ))
}

#[allow(clippy::cognitive_complexity)]
fn enum_content(
    input_type: &Ident,
    data_enum: &DataEnum,
    method_ident: &Ident,
) -> TokenStream {
    let mut matches = vec![];
    let mut method_iter = iter::repeat(method_ident);

    for variant in &data_enum.variants {
        let subtype = &variant.ident;
        let subtype = quote! { #input_type::#subtype };

        match variant.fields {
            Fields::Unnamed(ref fields) => {
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
            Fields::Named(ref fields) => {
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
            Fields::Unit => {
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
