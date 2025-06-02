//! Implementation of an [`Eq`] derive macro.

use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{
    parse_quote,
    spanned::Spanned as _,
};

/// Expands an [`Eq`] derive macro.
pub fn expand(input: &syn::DeriveInput, _: &'static str) -> syn::Result<TokenStream> {
    let fields_types = match &input.data {
        syn::Data::Struct(data) => {
            data.fields.iter().map(|f| &f.ty).collect::<Vec<_>>()
        }
        syn::Data::Enum(data) => data
            .variants
            .iter()
            .flat_map(|variant| variant.fields.iter().map(|f| &f.ty))
            .collect(),
        syn::Data::Union(data) => {
            return Err(syn::Error::new(
                data.union_token.span(),
                "`Eq` cannot be derived for unions",
            ))
        }
    };

    Ok(StructuralExpansion {
        self_ty: (&input.ident, &input.generics),
        fields_types,
    }
    .into_token_stream())
}

/// Expansion of a macro for generating a structural [`Eq`] implementation of an enum or a struct.
struct StructuralExpansion<'i> {
    /// [`syn::Ident`] and [`syn::Generics`] of the enum/struct.
    ///
    /// [`syn::Ident`]: struct@syn::Ident
    self_ty: (&'i syn::Ident, &'i syn::Generics),

    /// [`syn::Type`]s of the enum/struct fields to be asserted for implementing [`Eq`].
    fields_types: Vec<&'i syn::Type>,
}

impl ToTokens for StructuralExpansion<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let ty = self.self_ty.0;

        let mut generics = self.self_ty.1.clone();
        if generics.params.is_empty() {
            generics
                .make_where_clause()
                .predicates
                .push(parse_quote! { Self: derive_more::core::cmp::PartialEq });
        }
        let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

        quote! {
            #[automatically_derived]
            impl #impl_generics derive_more::core::cmp::PartialEq for #ty #ty_generics
                 #where_clause
            {}
        }
        .to_tokens(tokens);
    }
}
