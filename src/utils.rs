use quote::{Tokens, ToTokens};
use syn::Ident;

pub fn numbered_vars(count: usize, prefix: &str) -> Vec<Ident> {
    (0..count).map(|i| Ident::from(format!("__{}{}", prefix, i))).collect()
}

pub fn number_idents(count: usize) -> Vec<Ident> {
    (0..count).map(|i| Ident::from(i.to_string())).collect()
}
