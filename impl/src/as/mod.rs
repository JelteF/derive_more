//! Implementations of [`AsRef`]/[`AsMut`] derive macros.

#[cfg(feature = "as_mut")]
pub(crate) mod r#mut;
#[cfg(feature = "as_ref")]
pub(crate) mod r#ref;

use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    spanned::Spanned,
    Token,
};

use crate::utils::{add_where_clauses_for_new_ident, Either};

/// Type alias for an expansion context:
/// - Derived trait [`syn::Ident`]
/// - Derived trait method [`syn::Ident`]
/// - Type name used as generic in blanket impls [`syn::Ident`]
/// - Optional `mut` token used in expansion
type ExpansionCtx<'a> = (
    &'a syn::Ident,
    &'a syn::Ident,
    &'a syn::Ident,
    Option<&'a Token![mut]>,
);

/// Expands a [`AsRef`]/[`AsMut`] derive macro.
pub fn expand(
    input: &syn::DeriveInput,
    (trait_ident, method_ident, conv_type, mutability): ExpansionCtx<'_>,
) -> syn::Result<TokenStream> {
    let trait_path = quote! { ::derive_more::#trait_ident };

    let field_args = extract_field_args(input, trait_ident, method_ident)?;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let input_type = &input.ident;

    Ok(
        field_args
            .into_iter()
            .map(
                |FieldArgs {forward, field, ident}| {
                    let member = quote! { self.#ident };
                    let field_type = &field.ty;
                    let (
                        body,
                        impl_generics,
                        where_clause,
                        trait_path,
                        return_type
                    ) = if forward {
                        let trait_path = quote! { #trait_path<#conv_type> };
                        let type_where_clauses = quote! {
                            where #field_type: #trait_path
                        };
                        let new_generics = add_where_clauses_for_new_ident(
                            &input.generics,
                            &[field],
                            conv_type,
                            type_where_clauses,
                            false,
                        );
                        let (impl_generics, _, where_clause) =
                            new_generics.split_for_impl();
                        let casted_trait = quote! { <#field_type as #trait_path> };

                        (
                            quote! { #casted_trait::#method_ident(& #mutability #member) },
                            quote! { #impl_generics },
                            quote! { #where_clause },
                            quote! { #trait_path },
                            quote! { & #mutability #conv_type },
                        )
                    } else {
                        (
                            quote! { & #mutability #member },
                            quote! { #impl_generics },
                            quote! { #where_clause },
                            quote! { #trait_path<#field_type> },
                            quote! { & #mutability #field_type },
                        )
                    };

                    quote! {
                        #[automatically_derived]
                        impl #impl_generics #trait_path for #input_type #ty_generics #where_clause {
                            #[inline]
                            fn #method_ident(& #mutability self) -> #return_type {
                                #body
                            }
                        }
                    }
                })
            .collect()
    )
}

/// A [`AsRef`]/[`AsMut`] derive macro struct attribute with parsed arguments.
struct StructAttr<'a> {
    args: StructAttrArgs,
    attr: &'a syn::Attribute,
}

/// Representation of a [`AsRef`]/[`AsMut`] derive macro struct attribute's arguments.
///
/// ```rust,ignore
/// #[as_ref(forward)]
/// ```
enum StructAttrArgs {
    Forward,
}

impl<'a> StructAttr<'a> {
    /// Parses a [`StructAttr`] from the provided [`syn::Attribute`]s.
    fn parse_attrs(
        attrs: &'a [syn::Attribute],
        method_ident: &syn::Ident,
    ) -> syn::Result<Option<Self>> {
        attrs
            .as_ref()
            .iter()
            .filter(|attr| attr.path().is_ident(method_ident))
            .try_fold(None, |mut attrs, attr| {
                let args = attr.parse_args()?;
                let field_attr = Self { args, attr };
                if attrs.replace(field_attr).is_some() {
                    Err(syn::Error::new(
                        attr.path().span(),
                        format!("only single `#[{method_ident}(...)]` attribute is allowed here"),
                    ))
                } else {
                    Ok(attrs)
                }
            })
    }
}

impl Parse for StructAttrArgs {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        input.parse::<syn::Path>().and_then(|path| {
            if path.is_ident("forward") {
                Ok(Self::Forward)
            } else {
                Err(syn::Error::new(
                    path.span(),
                    "unknown argument, only `forward` is allowed",
                ))
            }
        })
    }
}

/// A [`AsRef`]/[`AsMut`] derive macro field attribute with parsed arguments.
struct FieldAttr<'a> {
    attr: &'a syn::Attribute,
    args: FieldAttrArgs,
}

/// Representation of a [`AsRef`]/[`AsMut`] derive macro field attribute's arguments.
///
/// ```rust,ignore
/// #[as_ref]
/// #[as_ref(forward)]
/// #[as_ref(ignore)]
/// ```
enum FieldAttrArgs {
    Empty,
    Forward,
    Ignore,
}

impl<'a> FieldAttr<'a> {
    /// Parses a [`FieldAttr`] from the provided [`syn::Attribute`]s.
    fn parse_attrs(
        attrs: &'a [syn::Attribute],
        method_ident: &syn::Ident,
    ) -> syn::Result<Option<Self>> {
        attrs
            .as_ref()
            .iter()
            .filter(|attr| attr.path().is_ident(method_ident))
            .try_fold(None, |mut attrs, attr| {
                let args = FieldAttrArgs::parse_attr(attr)?;
                let field_attr = Self { attr, args };
                if attrs.replace(field_attr).is_some() {
                    Err(syn::Error::new(
                        attr.path().span(),
                        format!("only single `#[{method_ident}(...)]` attribute is allowed here"),
                    ))
                } else {
                    Ok(attrs)
                }
            })
    }
}

impl FieldAttrArgs {
    /// Parses a [`FieldAttrArgs`] from the provided [`syn::Attribute`].
    fn parse_attr(attr: &syn::Attribute) -> syn::Result<Self> {
        if matches!(attr.meta, syn::Meta::Path(_)) {
            return Ok(Self::Empty);
        }
        attr.parse_args::<syn::Path>().and_then(|p| {
            if p.is_ident("forward") {
                return Ok(Self::Forward);
            }
            if p.is_ident("ignore") {
                return Ok(Self::Ignore);
            }
            Err(syn::Error::new(
                p.span(),
                "unknown argument, only `forward` and `ignore` are allowed",
            ))
        })
    }
}

/// Parsed arguments to generate a single [`AsRef`]/[`AsMut`] impl
struct FieldArgs<'a> {
    forward: bool,
    field: &'a syn::Field,
    ident: Either<&'a syn::Ident, syn::Index>,
}

impl<'a> FieldArgs<'a> {
    /// [`FieldArgs`] constructor
    ///
    /// `index` is used as `ident` for unnamed fields
    fn new(field: &'a syn::Field, forward: bool, index: usize) -> Self {
        Self {
            field,
            forward,
            ident: field
                .ident
                .as_ref()
                .map_or_else(|| Either::Right(syn::Index::from(index)), Either::Left),
        }
    }
}

/// Extracts [`FieldArgs`] for each enabled field from provided [`syn::DeriveInput`]
///
/// # Enabled fields
///
/// If the struct is annotated, this is the struct's single field.
/// If any field is annotated with argument `ignore`, each field that's not ignored.
/// Otherwise, each annotated field.
///
/// Annotating multi-field structs is disallowed.
/// Annotating some fields with argument `ignore`, and some with other arguments is disallowed.
fn extract_field_args<'a>(
    input: &'a syn::DeriveInput,
    trait_ident: &syn::Ident,
    method_ident: &syn::Ident,
) -> syn::Result<Vec<FieldArgs<'a>>> {
    let data = match &input.data {
        syn::Data::Struct(data) => Ok(data),
        syn::Data::Enum(e) => Err(syn::Error::new(
            e.enum_token.span(),
            format!("`{trait_ident}` cannot be derived for enums"),
        )),
        syn::Data::Union(u) => Err(syn::Error::new(
            u.union_token.span(),
            format!("`{trait_ident}` cannot be derived for unions"),
        )),
    }?;

    if let Some(struct_attr) = StructAttr::parse_attrs(&input.attrs, method_ident)? {
        let mut fields = data.fields.iter();

        let field = fields.next().ok_or_else(|| {
            syn::Error::new(
                struct_attr.attr.span(),
                format!(
                    "`#[{method_ident}(...)]` can only be applied to structs with exactly one \
                     field",
                ),
            )
        })?;

        if FieldAttr::parse_attrs(&field.attrs, method_ident)?.is_some() {
            return Err(syn::Error::new(
                field.span(),
                format!("`#[{method_ident}(...)]` cannot be applied to both struct and field"),
            ));
        }

        if let Some(other_field) = fields.next() {
            return Err(syn::Error::new(
                other_field.span(),
                format!(
                    "`#[{method_ident}(...)]` can only be applied to structs with exactly one \
                     field",
                ),
            ));
        }

        let forward = matches!(struct_attr.args, StructAttrArgs::Forward);

        Ok(vec![FieldArgs::new(field, forward, 0)])
    } else {
        extract_many(&data.fields, method_ident)
    }
}

/// Extracts [`FieldArgs`] for each of the provided [`syn::Field`]s that's enabled.
///
/// See [`extract_field_args`] for enabled field semantics.
fn extract_many<'a>(
    fields: &'a syn::Fields,
    method_ident: &syn::Ident,
) -> syn::Result<Vec<FieldArgs<'a>>> {
    let attrs = fields
        .iter()
        .map(|field| FieldAttr::parse_attrs(&field.attrs, method_ident))
        .collect::<syn::Result<Vec<_>>>()?;

    let present_attrs = attrs
        .iter()
        .filter_map(|attr| attr.as_ref())
        .collect::<Vec<_>>();

    let all = present_attrs
        .iter()
        .all(|attr| matches!(attr.args, FieldAttrArgs::Ignore));

    if !all {
        if let Some(attr) = present_attrs
            .iter()
            .find(|attr| matches!(attr.args, FieldAttrArgs::Ignore))
        {
            return Err(syn::Error::new(
                attr.attr.span(),
                format!(
                    "`#[{0}(ignore)]` cannot be used in the same struct as other `#[{0}(...)]` \
                     attributes",
                    method_ident,
                ),
            ));
        }
    }

    if all {
        Ok(fields
            .iter()
            .enumerate()
            .zip(attrs)
            .filter(|(_, attr)| attr.is_none())
            .map(|((i, field), _)| FieldArgs::new(field, false, i))
            .collect())
    } else {
        Ok(fields
            .iter()
            .enumerate()
            .zip(attrs)
            .filter_map(|((i, field), attr)| match attr.map(|attr| attr.args) {
                Some(FieldAttrArgs::Empty) => Some(FieldArgs::new(field, false, i)),
                Some(FieldAttrArgs::Forward) => Some(FieldArgs::new(field, true, i)),
                Some(FieldAttrArgs::Ignore) => unreachable!(),
                None => None,
            })
            .collect())
    }
}
