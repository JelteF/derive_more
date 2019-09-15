use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{Data, DeriveInput, Field, Fields, Index, Type};
use crate::utils::{add_extra_ty_param_bound};


/// Checks whether `field` is decorated with the specifed simple attribute (e.g. `#[as_ref]`)
fn has_simple_attr(field: &Field, attr: &str) -> bool {
    field.attrs.iter().any(|a| {
        a.parse_meta()
            .map(|m| {
                m.path()
                    .segments
                    .first()
                    .map(|p| &p.ident == attr)
                    .unwrap_or(false)
            })
            .unwrap_or(false)
    })
}


/// Extracts types and identifiers from fields in the given struct
///
/// If `data` contains more than one field, only fields decorated with `attr` are considered.
fn extract_field_info<'a>(data: &'a Data, attr: &str) -> (Vec<&'a Type>, Vec<TokenStream>) {

    // Get iter over fields and check named/unnamed
    let named;
    let fields = match data {
        Data::Struct(data) => match data.fields {
            Fields::Named(_) => {
                named = true;
                data.fields.iter()
            },
            Fields::Unnamed(_) => {
                named = false;
                data.fields.iter()
            },
            Fields::Unit => panic!("struct must have one or more fields"),
        },
        _ => panic!("only structs may derive this trait"),
    };

    // If necessary, filter out undecorated fields
    let len = fields.len();
    let fields = fields.filter(|f| len == 1 || has_simple_attr(f, attr));

    // Extract info needed to generate impls
    if named {
        fields.map(|f| {
                let ident = f.ident.as_ref().unwrap();
                (&f.ty, quote!(#ident))
            })
            .unzip()
    } else {
        fields.enumerate()
            .map(|(i, f)| {
                let index = Index::from(i);
                (&f.ty, quote!(#index))
            })
            .unzip()
    }
}


pub fn expand(input: &DeriveInput, _: &str) -> TokenStream {

    let input_type = &input.ident;
    let (impl_generics, input_generics, where_clause) = input.generics.split_for_impl();
    let (field_type, field_ident) = extract_field_info(&input.data, "as_ref");

    quote! {#(
        impl#impl_generics ::core::convert::AsRef<#field_type> for #input_type#input_generics
        #where_clause
        {
            #[inline]
            fn as_ref(&self) -> &#field_type {
                &self.#field_ident
            }
        }
    )*}
}
