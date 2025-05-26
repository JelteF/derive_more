//! Implementation of a [`PartialEq`] derive macro.

use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};
use syn::spanned::Spanned as _;

/// Expands a [`PartialEq`] derive macro.
pub fn expand(input: &syn::DeriveInput, _: &'static str) -> syn::Result<TokenStream> {
    Ok(StructuralExpansion::try_from(input)?.into_token_stream())
}

/// Expansion of a macro for generating a structural [`PartialEq`] implementation of an enum or a
/// struct.
struct StructuralExpansion<'i> {
    /// [`syn::Ident`] and [`syn::Generics`] of the enum/struct.
    ///
    /// [`syn::Ident`]: struct@syn::Ident
    self_ty: (&'i syn::Ident, &'i syn::Generics),

    /// [`syn::Fields`] of the enum/struct to be compared in this [`StructuralExpansion`].
    variants: Vec<(Option<&'i syn::Ident>, &'i syn::Fields)>,
}

impl<'i> TryFrom<&'i syn::DeriveInput> for StructuralExpansion<'i> {
    type Error = syn::Error;

    fn try_from(input: &'i syn::DeriveInput) -> syn::Result<Self> {
        let variants = match &input.data {
            syn::Data::Struct(data) => {
                vec![(None, &data.fields)]
            }
            syn::Data::Enum(data) => data
                .variants
                .iter()
                .map(|variant| (Some(&variant.ident), &variant.fields))
                .collect(),
            syn::Data::Union(data) => {
                return Err(syn::Error::new(
                    data.union_token.span(),
                    "`PartialEq` cannot be derived structurally for unions",
                ))
            }
        };

        Ok(Self {
            self_ty: (&input.ident, &input.generics),
            variants,
        })
    }
}

impl ToTokens for StructuralExpansion<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let ty = self.self_ty.0;

        let mut generics = self.self_ty.1.clone();
        /*if !generics.params.is_empty() {
            generics.make_where_clause().predicates.push(parse_quote! {
                #inner_ty: derive_more::core::str::FromStr
            });
        }*/
        let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

        let discriminants_eq = (self.variants.len() > 1).then(|| {
            quote! {
                derive_more::core::mem::discriminant(self) ==
                    derive_more::core::mem::discriminant(other) &&
            }
        });
        let fields_eqs = self.variants.iter().map(|(variant_name, fields)| {
            let variant = variant_name.map(|i| quote! { :: #variant_name });
            let self_pattern = fields.pattern("__self_");
            let other_pattern = fields.pattern("__other_");
            let eqs = (0..fields.len()).map(|num| {
                let self_val = format_ident!("__self_{num}");
                let other_val = format_ident!("__other_{num}");
                quote! { #self_val == #other_val }
            });
            quote! {
                (Self #variant #self_pattern, Self #variant #other_pattern) => {
                    #( #eqs )&&*
                }
            }
        });

        let unreachable_arm = (self.variants.len() > 1).then(|| {
            quote! {
                // SAFETY: This arm is never reachable, but is required by the expanded
                //         `match (self, other)` expression when there is more than one variant.
                _ => unsafe { derive_more::core::intrinsics::unreachable() },
            }
        });

        quote! {
            #[automatically_derived]
            impl #impl_generics derive_more::core::cmp::PartialEq for #ty #ty_generics #where_clause {
                #[inline]
                fn eq(&self, other: &Self) -> bool {
                    #discriminants_eq
                    match (self, other) {
                        #( #fields_eqs , )*
                        #unreachable_arm
                    }
                }
            }
        }.to_tokens(tokens);
    }
}

trait FieldsExt {
    fn pattern(&self, prefix: &str) -> TokenStream;
}

impl FieldsExt for syn::Fields {
    fn pattern(&self, prefix: &str) -> TokenStream {
        match self {
            Self::Named(fields) => {
                let fields = fields.named.iter().enumerate().map(|(num, field)| {
                    let name = &field.ident;
                    let binding = format_ident!("{prefix}{num}");
                    quote! { #name: #binding }
                });
                quote! {{ #( #fields , )* }}
            }
            Self::Unnamed(fields) => {
                let fields = (0..fields.unnamed.len()).map(|num| {
                    let binding = format_ident!("{prefix}{num}");
                    quote! { #binding }
                });
                quote! {( #( #fields , )* )}
            }
            Self::Unit => todo!(),
        }
    }
}
