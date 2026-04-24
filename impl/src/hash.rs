//! Implementation of an [`Hash`] derive macro.

use crate::utils::{
    attr::{self, ParseMultiple},
    pattern_matching::FieldsExt as _,
    structural_inclusion::TypeExt as _,
    GenericsSearch, HashMap, HashSet, Spanning,
};
use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};
use syn::{
    parse_quote,
    punctuated::{self, Punctuated},
    spanned::Spanned as _,
};

/// Expands a [`Hash`] derive macro.
pub fn expand(input: &syn::DeriveInput, _: &'static str) -> syn::Result<TokenStream> {
    let attr_name = format_ident!("hash");
    let secondary_attr_name = format_ident!("eq");
    let tertiary_attr_name = format_ident!("partial_eq");
    let attr_names = [&attr_name, &secondary_attr_name, &tertiary_attr_name];
    let secondary_attr_names = [&secondary_attr_name, &tertiary_attr_name];

    let mut has_skipped_variants = false;
    let mut variants = vec![];

    match &input.data {
        syn::Data::Struct(data) => {
            for attr_name in &attr_names {
                if attr::Skip::parse_attrs(&input.attrs, attr_name)?.is_some() {
                    has_skipped_variants = true;
                    break;
                }
            }
            if !has_skipped_variants {
                let mut skipped_fields = SkippedFields::default();
                let mut alternate_hash_functions =
                    FieldsWithAlternateHashFunction::default();
                'fields: for (n, field) in data.fields.iter().enumerate() {
                    match attr::WithOrSkip::parse_attrs(&field.attrs, &attr_name)? {
                        None => {
                            for attr_name in &secondary_attr_names {
                                if attr::Skip::parse_attrs(&field.attrs, attr_name)?
                                    .is_some()
                                {
                                    _ = skipped_fields.insert(n);
                                    continue 'fields;
                                }
                            }
                        }
                        Some(Spanning {
                            item: attr::WithOrSkip::Skip,
                            ..
                        }) => {
                            skipped_fields.insert(n);
                        }

                        Some(Spanning {
                            item: attr::WithOrSkip::With(with),
                            ..
                        }) => {
                            alternate_hash_functions.insert(n, with.path.clone());
                        }
                    }
                }
                variants.push((
                    None,
                    &data.fields,
                    skipped_fields,
                    alternate_hash_functions,
                ));
            }
        }
        syn::Data::Enum(data) => {
            'variants: for variant in &data.variants {
                for attr_name in &attr_names {
                    if attr::Skip::parse_attrs(&variant.attrs, attr_name)?.is_some() {
                        has_skipped_variants = true;
                        continue 'variants;
                    }
                }
                let mut skipped_fields = SkippedFields::default();
                let mut alternate_hash_functions =
                    FieldsWithAlternateHashFunction::default();
                'fields: for (n, field) in variant.fields.iter().enumerate() {
                    match attr::WithOrSkip::parse_attrs(&field.attrs, &attr_name)? {
                        None => {
                            for attr_name in &secondary_attr_names {
                                if attr::Skip::parse_attrs(&field.attrs, attr_name)?
                                    .is_some()
                                {
                                    _ = skipped_fields.insert(n);
                                    continue 'fields;
                                }
                            }
                        }
                        Some(Spanning {
                            item: attr::WithOrSkip::Skip,
                            ..
                        }) => {
                            skipped_fields.insert(n);
                        }

                        Some(Spanning {
                            item: attr::WithOrSkip::With(with),
                            ..
                        }) => {
                            alternate_hash_functions.insert(n, with.path.clone());
                        }
                    }
                }
                variants.push((
                    Some(&variant.ident),
                    &variant.fields,
                    skipped_fields,
                    alternate_hash_functions,
                ));
            }
        }
        syn::Data::Union(data) => {
            return Err(syn::Error::new(
                data.union_token.span(),
                "`Hash` cannot be derived for unions",
            ))
        }
    }

    Ok(StructuralExpansion {
        self_ty: (&input.ident, &input.generics),
        variants,
        has_skipped_variants,
        is_enum: matches!(input.data, syn::Data::Enum(_)),
    }
    .into_token_stream())
}

/// Indices of [`syn::Field`]s marked with an [`attr::Skip`].
type SkippedFields = HashSet<usize>;

/// Mapping from [`syn::Field`] marked with an [`attr::With`] to the [`syn::Path`] of the alternate
/// hash function.
type FieldsWithAlternateHashFunction = HashMap<usize, syn::Path>;

/// Expansion of a macro for generating a structural [`Hash`] implementation of an enum or a
/// struct.
struct StructuralExpansion<'i> {
    /// [`syn::Ident`] and [`syn::Generics`] of the enum/struct.
    ///
    /// [`syn::Ident`]: struct@syn::Ident
    self_ty: (&'i syn::Ident, &'i syn::Generics),

    /// [`syn::Fields`] of the enum/struct to be hashed in this [`StructuralExpansion`].
    variants: Vec<(
        Option<&'i syn::Ident>,
        &'i syn::Fields,
        SkippedFields,
        FieldsWithAlternateHashFunction,
    )>,

    /// Indicator whether some original enum variants where skipped with an [`attr::Skip`].
    has_skipped_variants: bool,

    /// Indicator whether this expansion is for an enum.
    is_enum: bool,
}

impl StructuralExpansion<'_> {
    /// Generates body of the [`core::hash::Hash::hash()`] method implementation for this
    /// [`StructuralExpansion`], if it's required.
    fn body(&self) -> TokenStream {
        let no_op_body = quote! {};

        // Special case: empty enum.
        if self.is_enum && self.variants.is_empty() && !self.has_skipped_variants {
            return no_op_body;
        }

        // Special case: fully skipped struct.
        if !self.is_enum && self.variants.is_empty() && self.has_skipped_variants {
            return no_op_body;
        }
        // Special case: no fields to hash in struct/single-variant enum.
        if !(self.is_enum && self.has_skipped_variants)
            && self.variants.len() == 1
            && (self.variants[0].1.is_empty()
                || self.variants[0].1.len() == self.variants[0].2.len())
        {
            return no_op_body;
        }

        let match_arms = self
            .variants
            .iter()
            .map(
                |(variant, all_fields, skipped_fields, alternate_hash_functions)| {
                    let variant = variant.map(|variant| quote! { :: #variant });
                    let self_pattern = all_fields
                        .non_exhaustive_arm_pattern("__self_", skipped_fields);

                    let mut hash_exprs = (0..all_fields.len())
                        .filter(|num| !skipped_fields.contains(num))
                        .map(|num| {
                            let self_val = format_ident!("__self_{num}");
                            let hash_function = alternate_hash_functions
                                .get(&num)
                                .map(|it| quote! {#it})
                                .unwrap_or_else(
                                    || quote! {derive_more::core::hash::Hash::hash},
                                );

                            punctuated::Pair::Punctuated(
                                quote! { #hash_function(#self_val, state) },
                                quote! {;},
                            )
                        })
                        .collect::<Punctuated<TokenStream, _>>();
                    _ = hash_exprs.pop_punct();
                    quote! { (Self #variant #self_pattern) => { #hash_exprs } }
                },
            )
            .collect::<Vec<_>>();

        let discriminant_exprs = self.is_enum.then(|| {
            quote! {
                let __self_discr = derive_more::core::mem::discriminant(self);
                derive_more::core::hash::Hash::hash(&__self_discr, state);
            }
        });

        let match_expr = (!match_arms.is_empty()).then(|| {
            let no_fields_arm = (match_arms.len() != self.variants.len()
                || self.has_skipped_variants)
                .then(|| {
                    quote! { _ => {} }
                });

            quote! {
                match (self) {
                    #( #match_arms  )*
                    #no_fields_arm
                }
            }
        });

        quote! {
            #discriminant_exprs
            #match_expr
        }
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
            for (_, all_fields, skipped_fields, _) in &self.variants {
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
                            #field_ty: derive_more::core::hash::Hash
                        });
                    }
                }
            }
        }
        let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

        let body = self.body();
        let hash_method = quote! {
            #[inline]
            fn hash<__H: derive_more::core::hash::Hasher>(&self,  state: &mut __H) { #body }
        };

        quote! {
            #[allow(private_bounds)]
            #[automatically_derived]
            impl #impl_generics derive_more::core::hash::Hash for #ty #ty_generics
                 #where_clause
            {
                #hash_method
            }
        }
        .to_tokens(tokens);
    }
}
