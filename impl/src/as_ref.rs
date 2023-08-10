use crate::ref_conv;
use proc_macro2::TokenStream;
use quote::format_ident;
use syn::{parse::Result, DeriveInput};

pub fn expand(input: &DeriveInput, trait_name: &'static str) -> Result<TokenStream> {
    let as_ref_type = format_ident!("__AsRefT");
    let trait_ident = format_ident!("{trait_name}");
    let method_ident = format_ident!("as_ref");

    ref_conv::expand(input, &trait_ident, &method_ident, &as_ref_type, None)
}
