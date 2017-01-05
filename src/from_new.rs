use std::collections::HashMap;

use syntax::ast::*;
use syntax::codemap::Span;
use syntax::ext::base::{Annotatable, ExtCtxt};
use syntax::ext::build::AstBuilder;
use syntax::ptr::P;
use syntax::print::pprust::ty_to_string;
use proc_macro::TokenStream;
use syn;
use quote::Tokens;
use syn::{Field, Ident, Body, Variant, VariantData, MacroInput};


/// Provides the hook to expand `#[derive(From)]` into an implementation of `From`
pub fn expand(input: &MacroInput) -> Tokens {
    let old_type = Ident::from("i32");
    let new_type = &input.ident;
    let tokens = quote!{
        impl ::std::convert::From<#old_type> for #new_type {
            fn from(a: #old_type) -> #new_type {
                #new_type(a)
            }
        }
    };
    tokens
}
