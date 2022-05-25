use crate::utils::{AttrParams, DeriveType, SingleVariantData, State};
use convert_case::{Case, Casing};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{
    DeriveInput, Field, Fields, FieldsNamed, FieldsUnnamed, Ident, Result, Variant,
};

pub fn expand(input: &DeriveInput, trait_name: &'static str) -> Result<TokenStream> {
    let state = State::with_attr_params(
        input,
        "Default",
        quote!(::core::default),
        String::from("default"),
        AttrParams {
            enum_: vec![],
            variant: vec!["ignore"],
            struct_: vec![],
            field: vec![],
        },
    )?;
    let SingleVariantData {
        input_type,
        variant:
            Variant {
                ident: variant_ident,
                fields,
                ..
            },
        trait_path,
        impl_generics,
        ty_generics,
        where_clause,
        ..
    } = state.assert_single_enabled_variant();

    let fields = match fields {
        Fields::Named(fields) => {
            let fields = fields
                .named
                .iter()
                .map(|Field { ident, .. }| quote!(#ident: #trait_path::default()));
            quote!({#(#fields),*})
        }
        Fields::Unnamed(fields) => {
            let fields = fields
                .unnamed
                .iter()
                .map(|_| quote!(#trait_path::default()));
            quote!((#(#fields),*))
        }
        Fields::Unit => quote!(),
    };

    Ok(quote! {
        impl #impl_generics #trait_path for #input_type #ty_generics #where_clause{
            fn default() -> Self {
                #input_type::#variant_ident#fields
            }
        }
    })
}
