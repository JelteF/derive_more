//! Implementation of a [`PartialEq`] derive macro.

use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};
use syn::{
    parse_quote,
    punctuated::{self, Punctuated},
    spanned::Spanned as _,
    token,
};

use super::TypeExt as _;
use crate::utils::{
    attr::{self, ParseMultiple as _},
    GenericsSearch, HashSet,
};

/// Expands a [`PartialEq`] derive macro.
pub fn expand(input: &syn::DeriveInput, _: &'static str) -> syn::Result<TokenStream> {
    let attr_name = format_ident!("partial_eq");
    let secondary_attr_name = format_ident!("eq");

    let variants = match &input.data {
        syn::Data::Struct(data) => {
            let skipped_fields = data
                .fields
                .iter()
                .enumerate()
                .map(|(n, field)| {
                    for attr_name in [&attr_name, &secondary_attr_name] {
                        if attr::Skip::parse_attrs(&field.attrs, attr_name)?.is_some() {
                            return Ok(Some(n));
                        }
                    }
                    Ok(None)
                })
                .filter_map(syn::Result::transpose)
                .collect::<syn::Result<HashSet<_>>>()?;
            vec![(None, &data.fields, skipped_fields)]
        }
        syn::Data::Enum(data) => data
            .variants
            .iter()
            .map(|variant| {
                let skipped_fields = variant
                    .fields
                    .iter()
                    .enumerate()
                    .map(|(n, field)| {
                        for attr_name in [&attr_name, &secondary_attr_name] {
                            if attr::Skip::parse_attrs(&field.attrs, attr_name)?
                                .is_some()
                            {
                                return Ok(Some(n));
                            }
                        }
                        Ok(None)
                    })
                    .filter_map(syn::Result::transpose)
                    .collect::<syn::Result<HashSet<_>>>()?;
                Ok((Some(&variant.ident), &variant.fields, skipped_fields))
            })
            .collect::<syn::Result<_>>()?,
        syn::Data::Union(data) => {
            return Err(syn::Error::new(
                data.union_token.span(),
                "`PartialEq` cannot be derived for unions",
            ))
        }
    };

    Ok(StructuralExpansion {
        self_ty: (&input.ident, &input.generics),
        variants,
    }
    .into_token_stream())
}

/// Indices of [`syn::Field`]s marked with an [`attr::Skip`].
type SkippedFields = HashSet<usize>;

/// Expansion of a macro for generating a structural [`PartialEq`] implementation of an enum or a
/// struct.
struct StructuralExpansion<'i> {
    /// [`syn::Ident`] and [`syn::Generics`] of the enum/struct.
    ///
    /// [`syn::Ident`]: struct@syn::Ident
    self_ty: (&'i syn::Ident, &'i syn::Generics),

    /// [`syn::Fields`] of the enum/struct to be compared in this [`StructuralExpansion`].
    variants: Vec<(Option<&'i syn::Ident>, &'i syn::Fields, SkippedFields)>,
}

impl StructuralExpansion<'_> {
    /// Generates body of the [`PartialEq::eq()`]/[`PartialEq::ne()`] method implementation for this
    /// [`StructuralExpansion`], if it's required.
    fn body(&self, eq: bool) -> Option<TokenStream> {
        // Special case: empty enum (also, no need for `ne()` method in this case).
        if self.variants.is_empty() {
            return eq.then(|| quote! { match *self {} });
        }

        let no_fields_result = quote! { #eq };

        // Special case: no fields to compare (also, no need for `ne()` method in this case).
        if self.variants.len() == 1
            && (self.variants[0].1.is_empty()
                || self.variants[0].1.len() == self.variants[0].2.len())
        {
            return eq.then_some(no_fields_result);
        }

        let (cmp, chain) = if eq {
            (quote! { == }, quote! { && })
        } else {
            (quote! { != }, quote! { || })
        };

        let discriminants_cmp = (self.variants.len() > 1).then(|| {
            quote! {
                derive_more::core::mem::discriminant(self) #cmp
                    derive_more::core::mem::discriminant(__other)
            }
        });

        let match_arms = self
            .variants
            .iter()
            .filter_map(|(variant, all_fields, skipped_fields)| {
                if all_fields.is_empty() || skipped_fields.len() == all_fields.len() {
                    return None;
                }

                let variant = variant.map(|variant| quote! { :: #variant });
                let self_pattern = all_fields.arm_pattern("__self_", skipped_fields);
                let other_pattern = all_fields.arm_pattern("__other_", skipped_fields);

                let mut val_eqs = (0..all_fields.len())
                    .filter_map(|num| {
                        (!skipped_fields.contains(&num)).then(|| {
                            let self_val = format_ident!("__self_{num}");
                            let other_val = format_ident!("__other_{num}");
                            punctuated::Pair::Punctuated(
                                quote! { #self_val #cmp #other_val },
                                &chain,
                            )
                        })
                    })
                    .collect::<Punctuated<TokenStream, _>>();
                _ = val_eqs.pop_punct();

                Some(quote! {
                    (Self #variant #self_pattern, Self #variant #other_pattern) => { #val_eqs }
                })
            })
            .collect::<Vec<_>>();
        let match_expr = (!match_arms.is_empty()).then(|| {
            let no_fields_arm = (match_arms.len() != self.variants.len()).then(|| {
                quote! { _ => #no_fields_result }
            });
            let unreachable_arm = (self.variants.len() > 1
                && no_fields_arm.is_none())
            .then(|| {
                quote! {
                    // SAFETY: This arm is never reachable, but is required by the expanded
                    //         `match (self, other)` expression when there is more than one variant.
                    _ => unsafe { derive_more::core::hint::unreachable_unchecked() },
                }
            });

            quote! {
                match (self, __other) {
                    #( #match_arms , )*
                    #no_fields_arm
                    #unreachable_arm
                }
            }
        });

        // If there is only `mem::discriminant()` comparison, there is no need to generate `ne()`
        // method in the expansion, as its default implementation will do just fine.
        if !eq && discriminants_cmp.is_some() && match_expr.is_none() {
            return None;
        }

        let chain =
            (discriminants_cmp.is_some() && match_expr.is_some()).then_some(chain);

        Some(quote! {
            #discriminants_cmp #chain #match_expr
        })
    }
}

impl ToTokens for StructuralExpansion<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let ty = self.self_ty.0;
        let (_, ty_generics, _) = self.self_ty.1.split_for_impl();

        let generics_search = GenericsSearch::from(self.self_ty.1);
        let mut generics = self.self_ty.1.clone();
        {
            let self_ty: syn::Type = parse_quote! { Self };
            let implementor_ty: syn::Type = parse_quote! { #ty #ty_generics };
            for (_, all_fields, skipped_fields) in &self.variants {
                for field_ty in
                    all_fields.iter().enumerate().filter_map(|(n, field)| {
                        (!skipped_fields.contains(&n)).then_some(&field.ty)
                    })
                {
                    if generics_search.any_in(field_ty)
                        && !field_ty.contains_type_structurally(&self_ty)
                        && !field_ty.contains_type_structurally(&implementor_ty)
                    {
                        generics.make_where_clause().predicates.push(parse_quote! {
                            #field_ty: derive_more::core::cmp::PartialEq
                        });
                    }
                }
            }
        }
        let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

        let eq_method = self.body(true).map(|body| {
            quote! {
                #[inline]
                fn eq(&self, __other: &Self) -> bool { #body }
            }
        });
        let ne_method = self.body(false).map(|body| {
            quote! {
                #[inline]
                fn ne(&self, __other: &Self) -> bool { #body }
            }
        });

        quote! {
            #[allow(private_bounds)]
            #[automatically_derived]
            impl #impl_generics derive_more::core::cmp::PartialEq for #ty #ty_generics
                 #where_clause
            {
                #eq_method
                #ne_method
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
    fn arm_pattern(&self, prefix: &str, skipped_indices: &SkippedFields)
        -> TokenStream;
}

impl FieldsExt for syn::Fields {
    fn arm_pattern(
        &self,
        prefix: &str,
        skipped_indices: &SkippedFields,
    ) -> TokenStream {
        match self {
            Self::Named(fields) => {
                let wildcard =
                    (!skipped_indices.is_empty()).then(token::DotDot::default);
                let fields =
                    fields.named.iter().enumerate().filter_map(|(num, field)| {
                        (!skipped_indices.contains(&num)).then(|| {
                            let name = &field.ident;
                            let binding = format_ident!("{prefix}{num}");
                            quote! { #name: #binding }
                        })
                    });
                quote! {{ #( #fields , )* #wildcard }}
            }
            Self::Unnamed(fields) => {
                let fields = (0..fields.unnamed.len()).map(|num| {
                    if skipped_indices.contains(&num) {
                        quote! { _ }
                    } else {
                        let binding = format_ident!("{prefix}{num}");
                        quote! { #binding }
                    }
                });
                quote! {( #( #fields , )* )}
            }
            Self::Unit => quote! {},
        }
    }
}
