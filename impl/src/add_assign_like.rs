use crate::{
    add_helpers::{struct_assign_exprs, tuple_assign_exprs},
    utils::{add_extra_ty_param_bound_op_except, named_to_vec, unnamed_to_vec},
};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{Data, DeriveInput, Fields};

pub fn expand(input: &DeriveInput, trait_name: &str) -> syn::Result<TokenStream> {
    let trait_ident = format_ident!("{trait_name}");
    let method_name = trait_name.trim_end_matches("Assign").to_lowercase();
    let method_ident = format_ident!("{method_name}_assign");
    let input_type = &input.ident;

    let (exprs, zst_generics) = match input.data {
        Data::Struct(ref data_struct) => match data_struct.fields {
            Fields::Unnamed(ref fields) => {
                tuple_assign_exprs(&unnamed_to_vec(fields), &method_ident)?
            }
            Fields::Named(ref fields) => {
                struct_assign_exprs(&named_to_vec(fields), &method_ident)?
            }
            _ => panic!("Unit structs cannot use derive({trait_name})"),
        },

        _ => panic!("Only structs can use derive({trait_name})"),
    };

    let generics =
        add_extra_ty_param_bound_op_except(&input.generics, &trait_ident, zst_generics);
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    Ok(quote! {
        #[automatically_derived]
        impl #impl_generics derive_more::core::ops::#trait_ident
         for #input_type #ty_generics #where_clause {
            #[inline]
            #[track_caller]
            fn #method_ident(&mut self, rhs: #input_type #ty_generics) {
                #( #exprs; )*
            }
        }
    })
}
