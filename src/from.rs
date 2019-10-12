use crate::utils::{add_where_clauses_for_new_ident, DeriveType, MultiFieldData, State};
use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{parse::Result, DeriveInput, Ident, Index};

/// Provides the hook to expand `#[derive(From)]` into an implementation of `From`
pub fn expand(input: &DeriveInput, trait_name: &'static str) -> Result<TokenStream> {
    let state = State::new(
        input,
        trait_name,
        quote!(::core::convert),
        trait_name.to_lowercase(),
    )?;
    if state.derive_type == DeriveType::Enum {
        Ok(enum_from(input, state))
    } else {
        Ok(struct_from(input, &state))
    }
}

pub fn struct_from(input: &DeriveInput, state: &State) -> TokenStream {
    let MultiFieldData {
        variant_type,
        fields,
        field_idents,
        infos,
        input_type,
        trait_path,
        ..
    } = state.enabled_fields_data();

    let mut new_generics = input.generics.clone();
    let sub_items: Vec<_> = infos
        .iter()
        .zip(fields.iter())
        .enumerate()
        .map(|(i, (info, field))| {
            let field_type = &field.ty;
            let variable = if fields.len() == 1 {
                quote!(original)
            } else {
                let tuple_index = Index::from(i);
                quote!(original.#tuple_index)
            };
            if info.forward {
                let type_param = &Ident::new(&format!("__FromT{}", i), Span::call_site());
                let sub_trait_path = quote!(#trait_path<#type_param>);
                let type_where_clauses = quote! {
                    where #field_type: #sub_trait_path
                };
                new_generics = add_where_clauses_for_new_ident(
                    &input.generics,
                    &[field],
                    type_param,
                    type_where_clauses,
                    true,
                );
                let casted_trait = quote!(<#field_type as #sub_trait_path>);
                (quote!(#casted_trait::from(#variable)), quote!(#type_param))
            } else {
                (variable, quote!(#field_type))
            }
        })
        .collect();
    let initializers = sub_items.iter().map(|i| &i.0);
    let from_types: Vec<_> = sub_items.iter().map(|i| &i.1).collect();
    let body = if state.derive_type == DeriveType::Named {
        quote!(#variant_type{#(#field_idents: #initializers),*})
    } else {
        quote!(#variant_type(#(#initializers),*))
    };
    let (impl_generics, _, where_clause) = new_generics.split_for_impl();
    let (_, ty_generics, _) = input.generics.split_for_impl();

    quote! {
        impl#impl_generics #trait_path<(#(#from_types),*)> for
            #input_type#ty_generics #where_clause {

            #[allow(unused_variables)]
            #[inline]
            fn from(original: (#(#from_types),*)) -> #input_type#ty_generics {
                #body
            }
        }
    }
}

fn enum_from(input: &DeriveInput, state: State) -> TokenStream {
    let mut tokens = TokenStream::new();

    for variant_state in state.enabled_variant_data().variant_states {
        struct_from(input, variant_state).to_tokens(&mut tokens);
    }
    tokens
}
