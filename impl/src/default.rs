use crate::utils::attr::{self, ParseMultiple};
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{Data, DeriveInput, Fields};

pub fn expand(input: &DeriveInput, _: &'static str) -> syn::Result<TokenStream> {
    let name = &input.ident;
    let generics = &input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let ident = quote::format_ident!("default");
    let real_where_clause = attr::Bounds::parse_attrs(&input.attrs, &ident)?
        .map(|clause| {
            let clause = clause.item.0;
            quote! { where #clause }
        })
        .or(where_clause.map(ToTokens::to_token_stream));

    let default_body = match &input.data {
        Data::Struct(data) => default(&data.fields, None),
        Data::Enum(data) => {
            let default_variant = data
                .variants
                .iter()
                .find(|v| v.attrs.iter().any(|a| a.path().is_ident("default")))
                .ok_or_else(|| {
                    syn::Error::new_spanned(
                        &input.ident,
                        "one variant must be marked with #[default]",
                    )
                })?;

            let default_attr = default_variant
                .attrs
                .iter()
                .find(|a| a.path().is_ident("default"))
                .expect("just checked existence");

            if !matches!(default_attr.meta, syn::Meta::Path(_)) {
                return Err(syn::Error::new_spanned(
                    default_attr,
                    "#[default] on variant must not have arguments",
                ));
            }

            let variant_ident = &default_variant.ident;
            default(&default_variant.fields, Some(quote! { ::#variant_ident }))
        }
        Data::Union(data) => {
            return Err(syn::Error::new_spanned(
                data.union_token,
                "Clone cannot be derived for unions",
            ));
        }
    };

    Ok(quote! {
        impl #impl_generics derive_more::core::default::Default for #name #ty_generics #real_where_clause {
            fn default() -> Self {
                #default_body
            }
        }
    })
}

fn default(fields: &Fields, variant: Option<TokenStream>) -> TokenStream {
    match fields {
        Fields::Named(named) => {
            let defaults = named.named.iter().map(|f| {
                let name = &f.ident;
                let value = field_default_value(f);
                quote! { #name: #value }
            });
            quote! { Self #variant { #(#defaults),* } }
        }
        Fields::Unnamed(unnamed) => {
            let defaults = unnamed.unnamed.iter().map(|f| field_default_value(f));
            quote! { Self #variant (#(#defaults),*) }
        }
        Fields::Unit => quote! { Self #variant },
    }
}

fn field_default_value(field: &syn::Field) -> TokenStream {
    for attr in &field.attrs {
        if attr.path().is_ident("default") {
            if let syn::Meta::List(meta_list) = &attr.meta {
                return meta_list.tokens.clone();
            }
        }
    }
    quote! { derive_more::core::default::Default::default() }
}
