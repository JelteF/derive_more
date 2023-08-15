//! Implementation of an [`AsMut`] derive macro.

use proc_macro2::TokenStream;
use quote::format_ident;
use syn::{parse::Result, DeriveInput, Token};

pub fn expand(input: &DeriveInput, trait_name: &'static str) -> Result<TokenStream> {
    let as_mut_type = format_ident!("__AsMutT");
    let trait_ident = format_ident!("{trait_name}");
    let method_ident = format_ident!("as_mut");
    let mutability = <Token![mut]>::default();

    super::expand(
        input,
        (&trait_ident, &method_ident, &as_mut_type, Some(&mutability)),
    )
}
