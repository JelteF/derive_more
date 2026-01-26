use crate::utils::attr::{self, ParseMultiple};
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{Data, DeriveInput, Fields};

pub fn expand(input: &DeriveInput, _: &'static str) -> syn::Result<TokenStream> {
    let name = &input.ident;
    let generics = &input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let ident = quote::format_ident!("clone");
    let real_where_clause = attr::Bounds::parse_attrs(&input.attrs, &ident)?
        .map(|clause| {
            let clause = clause.item.0;
            quote! { where #clause }
        })
        .or(where_clause.map(ToTokens::to_token_stream));

    let clone_body = match &input.data {
        Data::Struct(data) => clone_struct(&data.fields),
        Data::Enum(data) => {
            let variants = data.variants.iter().map(|v| {
                let variant_name = &v.ident;
                let (pattern, clone_expr) = clone_variant_fields(&v.fields);
                quote! {
                    Self::#variant_name #pattern => Self::#variant_name #clone_expr
                }
            });
            quote! {
                match self {
                    #(#variants),*
                }
            }
        }
        Data::Union(data) => {
            return Err(syn::Error::new_spanned(
                data.union_token,
                "Clone cannot be derived for unions",
            ));
        }
    };

    Ok(quote! {
        impl #impl_generics derive_more::core::clone::Clone for #name #ty_generics #real_where_clause {
            fn clone(&self) -> Self {
                #clone_body
            }
        }
    })
}

fn clone_struct(fields: &Fields) -> TokenStream {
    match fields {
        Fields::Named(named) => {
            let clones = named.named.iter().map(|f| {
                let name = &f.ident;
                quote! { #name: derive_more::core::clone::Clone::clone(&self.#name) }
            });
            quote! { Self { #(#clones),* } }
        }
        Fields::Unnamed(unnamed) => {
            let clones = (0..unnamed.unnamed.len()).map(|i| {
                let idx = syn::Index::from(i);
                quote! { derive_more::core::clone::Clone::clone(&self.#idx) }
            });
            quote! { Self(#(#clones),*) }
        }
        Fields::Unit => quote! { Self },
    }
}

fn clone_variant_fields(fields: &Fields) -> (TokenStream, TokenStream) {
    match fields {
        Fields::Named(named) => {
            let names: Vec<_> = named.named.iter().map(|f| &f.ident).collect();
            let pattern = quote! { { #(#names),* } };
            let clones = names.iter().map(|n| {
                quote! { #n: derive_more::core::clone::Clone::clone(#n) }
            });
            (pattern, quote! { { #(#clones),* } })
        }
        Fields::Unnamed(unnamed) => {
            let bindings: Vec<_> = (0..unnamed.unnamed.len())
                .map(|i| quote::format_ident!("_{}", i))
                .collect();
            let pattern = quote! { (#(#bindings),*) };
            let clones = bindings.iter().map(|b| {
                quote! { derive_more::core::clone::Clone::clone(#b) }
            });
            (pattern, quote! { (#(#clones),*) })
        }
        Fields::Unit => (TokenStream::new(), TokenStream::new()),
    }
}
