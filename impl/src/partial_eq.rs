//! Implementation of a [`PartialEq`] derive macro.

use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};
use syn::{parse_quote, spanned::Spanned as _, token};

use crate::utils::GenericsSearch;

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

impl StructuralExpansion<'_> {
    /// Generates body of the [`PartialEq::eq()`] method implementation for this
    /// [`StructuralExpansion`].
    fn eq_body(&self) -> TokenStream {
        // Special case: empty enum.
        if self.variants.is_empty() {
            return quote! { match *self {} };
        }
        // Special case: no fields to compare.
        if self.variants.len() == 1 && self.variants[0].1.is_empty() {
            return quote! { true };
        }

        let discriminants_eq = (self.variants.len() > 1).then(|| {
            quote! {
                derive_more::core::mem::discriminant(self) ==
                    derive_more::core::mem::discriminant(__other)
            }
        });

        let matched_variants = self
            .variants
            .iter()
            .filter_map(|(variant, fields)| {
                if fields.is_empty() {
                    return None;
                }
                let variant = variant.map(|variant| quote! { :: #variant });
                let self_pattern = fields.arm_pattern("__self_");
                let other_pattern = fields.arm_pattern("__other_");
                let val_eqs = (0..fields.len()).map(|num| {
                    let self_val = format_ident!("__self_{num}");
                    let other_val = format_ident!("__other_{num}");
                    quote! { #self_val == #other_val }
                });
                Some(quote! {
                    (Self #variant #self_pattern, Self #variant #other_pattern) => {
                        #( #val_eqs )&&*
                    }
                })
            })
            .collect::<Vec<_>>();
        let match_expr = (!matched_variants.is_empty()).then(|| {
            let always_true_arm =
                (matched_variants.len() != self.variants.len()).then(|| {
                    quote! { _ => true }
                });
            let unreachable_arm = (self.variants.len() > 1
                && always_true_arm.is_none())
            .then(|| {
                quote! {
                    // SAFETY: This arm is never reachable, but is required by the expanded
                    //         `match (self, other)` expression when there is more than one variant.
                    _ => unsafe { derive_more::core::hint::unreachable_unchecked() },
                }
            });

            quote! {
                match (self, __other) {
                    #( #matched_variants , )*
                    #always_true_arm
                    #unreachable_arm
                }
            }
        });

        let and = (discriminants_eq.is_some() && match_expr.is_some())
            .then_some(token::AndAnd::default());

        quote! {
            #discriminants_eq #and #match_expr
        }
    }
}

impl ToTokens for StructuralExpansion<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let ty = self.self_ty.0;

        let generics_search = GenericsSearch::from(self.self_ty.1);
        let mut generics = self.self_ty.1.clone();
        for variant in &self.variants {
            for field_ty in variant.1.iter().map(|field| &field.ty) {
                if generics_search.any_in(field_ty) {
                    generics.make_where_clause().predicates.push(parse_quote! {
                        #field_ty: derive_more::core::cmp::PartialEq
                    });
                }
            }
        }
        let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

        let eq_body = self.eq_body();

        quote! {
            #[automatically_derived]
            impl #impl_generics derive_more::core::cmp::PartialEq for #ty #ty_generics
                 #where_clause
            {
                #[inline]
                fn eq(&self, __other: &Self) -> bool {
                    #eq_body
                }
            }
        }
        .to_tokens(tokens);
    }
}

/// Extension of [`syn::Fields`] used by this expansion.
trait FieldsExt {
    /// Generates a pattern for matching these [`syn::Fields`] in an arm of a `match` expression.
    ///
    /// All the [`syn::Fields`] will be assigned as `{prefix}{num}` bindings for use.
    fn arm_pattern(&self, prefix: &str) -> TokenStream;
}

impl FieldsExt for syn::Fields {
    fn arm_pattern(&self, prefix: &str) -> TokenStream {
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
            Self::Unit => quote! {},
        }
    }
}
