use crate::utils::{
    field_idents, get_field_types, named_to_vec, number_idents, unnamed_to_vec, DeriveType,
    MultiFieldData, State,
};
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse::Result, DeriveInput, Field, Fields};

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
        struct_from(input, state)
    }
}

pub fn struct_from(input: &DeriveInput, state: State) -> Result<TokenStream> {
    let MultiFieldData {
        fields,
        field_types,
        input_type,
        trait_path,
        ..
    } = state.enabled_fields_data();
    let body = if state.derive_type == DeriveType::Named {
        struct_body(input_type, &fields)
    } else {
        tuple_body(input_type, &fields)
    };

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    Ok(quote! {
        impl#impl_generics #trait_path<(#(#field_types),*)> for
            #input_type#ty_generics #where_clause {

            #[allow(unused_variables)]
            #[inline]
            fn from(original: (#(#field_types),*)) -> #input_type#ty_generics {
                #body
            }
        }
    })
}

pub fn from_impl<T: ToTokens>(input: &DeriveInput, fields: &[&Field], body: T) -> TokenStream {
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let input_type = &input.ident;
    let original_types = &get_field_types(fields);
    quote! {
        impl#impl_generics ::core::convert::From<(#(#original_types),*)> for
            #input_type#ty_generics #where_clause {

            #[allow(unused_variables)]
            #[inline]
            fn from(original: (#(#original_types),*)) -> #input_type#ty_generics {
                #body
            }
        }
    }
}

fn tuple_body<T: ToTokens>(return_type: T, fields: &[&Field]) -> TokenStream {
    if fields.len() == 1 {
        quote!(#return_type(original))
    } else {
        let field_names = &number_idents(fields.len());
        quote!(#return_type(#(original.#field_names),*))
    }
}

fn struct_body<T: ToTokens>(return_type: T, fields: &[&Field]) -> TokenStream {
    if fields.len() == 1 {
        let field_name = &fields[0].ident;
        quote!(#return_type{#field_name: original})
    } else {
        let argument_field_names = &number_idents(fields.len());
        let field_names = &field_idents(fields);
        quote!(#return_type{#(#field_names: original.#argument_field_names),*})
    }
}

fn enum_from(input: &DeriveInput, state: State) -> TokenStream {
    let input_type = &input.ident;
    let mut tokens = TokenStream::new();

    for variant in state.enabled_variant_data().variants {
        match variant.fields {
            Fields::Unnamed(ref fields) => {
                let field_vec = &unnamed_to_vec(fields);

                let variant_ident = &variant.ident;
                let body = tuple_body(quote!(#input_type::#variant_ident), field_vec);
                from_impl(input, field_vec, body).to_tokens(&mut tokens)
            }

            Fields::Named(ref fields) => {
                let field_vec = &named_to_vec(fields);

                let variant_ident = &variant.ident;
                let body = struct_body(quote!(#input_type::#variant_ident), field_vec);
                from_impl(input, field_vec, body).to_tokens(&mut tokens)
            }
            Fields::Unit => {
                let variant_ident = &variant.ident;
                let body = struct_body(quote!(#input_type::#variant_ident), &[]);
                from_impl(input, &[], body).to_tokens(&mut tokens)
            }
        }
    }
    tokens
}
