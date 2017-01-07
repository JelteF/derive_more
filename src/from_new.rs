use std::collections::HashMap;

use quote::Tokens;
use syn::{Body, Field, Ident, Variant, VariantData, MacroInput, Ty};


/// Provides the hook to expand `#[derive(From)]` into an implementation of `From`
pub fn expand(input: &MacroInput) -> Tokens {
    match input.body {
        Body::Struct(VariantData::Tuple(ref structs)) => {
            if structs.len() == 1 {
                newtype_from(input.ident.clone(), structs[0].ty.clone())
            }
            else {
                panic!("Only Tuple structs with a single field can derive From")
            }
        }
        _ => panic!("Only newtype structs can derive From")
    }
}


fn newtype_from(new_type: Ident, old_type: Ty) -> Tokens {
    quote!{
        impl ::std::convert::From<#old_type> for #new_type {
            fn from(a: #old_type) -> #new_type {
                #new_type(a)
            }
        }
    }
}
