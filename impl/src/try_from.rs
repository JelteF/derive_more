//! Implementation of a [`TryFrom`] derive macro.

use proc_macro2::{Literal, Span, TokenStream};
use quote::{format_ident, quote, ToTokens};
use syn::spanned::Spanned as _;

/// Expands a [`TryFrom`] derive macro.
pub fn expand(input: &syn::DeriveInput, _: &'static str) -> syn::Result<TokenStream> {
    match &input.data {
        syn::Data::Struct(data) => Err(syn::Error::new(
            data.struct_token.span(),
            "`TryFrom` cannot be derived for structs",
        )),
        syn::Data::Enum(data) => Ok(Expansion {
            repr: ReprAttribute::parse_attrs(&input.attrs)?,
            attr: ItemAttribute::parse_attrs(&input.attrs)?,
            ident: input.ident.clone(),
            variants: data.variants.clone().into_iter().collect(),
            generics: input.generics.clone(),
        }
        .into_token_stream()),
        syn::Data::Union(data) => Err(syn::Error::new(
            data.union_token.span(),
            "`TryFrom` cannot be derived for unions",
        )),
    }
}

/// Representation of a [`TryFrom`] derive macro struct item attribute.
///
/// ```rust,ignore
/// #[try_from(repr)]
/// ```
#[derive(Default)]
struct ItemAttribute {
    /// plain `repr`
    repr: bool,
}

impl ItemAttribute {
    /// Parses a [`StructAttribute`] from the provided [`syn::Attribute`]s.
    fn parse_attrs(attrs: impl AsRef<[syn::Attribute]>) -> syn::Result<Self> {
        attrs
            .as_ref()
            .iter()
            .filter(|attr| attr.path().is_ident("try_from"))
            .try_fold(ItemAttribute::default(), |mut attrs, attr| {
                attr.parse_nested_meta(|meta| {
                    if meta.path.is_ident("repr") {
                        attrs.repr = true;
                        Ok(())
                    } else {
                        Err(meta.error("only `repr` is allowed here"))
                    }
                })
                .map(|_| attrs)
            })
    }
}

/// Representation of a [`Repr`] derive macro struct container attribute.
///
/// Note: This disregards any non integer representation reprs.
///
/// ```rust,ignore
/// #[repr(<type>)]
/// ```
struct ReprAttribute(syn::Ident);

impl ReprAttribute {
    /// Parses a [`ReprAttribute`] from the provided [`syn::Attribute`]s.
    fn parse_attrs(attrs: impl AsRef<[syn::Attribute]>) -> syn::Result<Self> {
        attrs
            .as_ref()
            .iter()
            .filter(|attr| attr.path().is_ident("repr"))
            .try_fold(None, |mut repr, attr| {
                attr.parse_nested_meta(|meta| {
                    if let Some(ident) = meta.path.get_ident() {
                        if let "u8" | "u16" | "u32" | "u64" | "u128" | "usize" | "i8"
                        | "i16" | "i32" | "i64" | "i128" | "isize" =
                            ident.to_string().as_str()
                        {
                            repr = Some(ident.clone());
                            return Ok(());
                        }
                    }
                    // ignore all other attributes that could have a body e.g. `align`
                    _ = meta.input.parse::<proc_macro2::Group>();
                    Ok(())
                })
                .map(|_| repr)
            })
            // Default discriminant is interpreted as `isize` (https://doc.rust-lang.org/reference/items/enumerations.html#discriminants)
            .map(|repr| {
                repr.unwrap_or_else(|| syn::Ident::new("isize", Span::call_site()))
            })
            .map(Self)
    }
}

/// Expansion of a macro for generating [`TryFrom`] implementation of an enum
struct Expansion {
    /// Enum `#[repr(u/i*)]`
    repr: ReprAttribute,
    /// Attributes on item.
    attr: ItemAttribute,
    /// Enum [`Ident`].
    ident: syn::Ident,
    /// Variant [`Ident`] in case of enum expansion.
    variants: Vec<syn::Variant>,
    /// Struct or enum [`syn::Generics`].
    generics: syn::Generics,
}

impl ToTokens for Expansion {
    /// Expands [`TryFrom`] implementations for a struct.
    fn to_tokens(&self, tokens: &mut TokenStream) {
        if !self.attr.repr {
            return;
        }
        let ident = &self.ident;
        let (impl_generics, ty_generics, where_clause) = self.generics.split_for_impl();

        let repr = &self.repr.0;

        let mut last_discriminant = quote! {0};
        let mut inc = 0usize;
        let (consts, (discriminants, variants)): (
            Vec<syn::Ident>,
            (Vec<TokenStream>, Vec<TokenStream>),
        ) = self
            .variants
            .iter()
            .filter_map(
                |syn::Variant {
                     ident,
                     fields,
                     discriminant,
                     ..
                 }| {
                    if let Some(discriminant) = discriminant {
                        last_discriminant = discriminant.1.to_token_stream();
                        inc = 0;
                    }
                    let ret = {
                        let inc = Literal::usize_unsuffixed(inc);
                        fields.is_empty().then_some((
                            format_ident!("__DISCRIMINANT_{ident}"),
                            (
                                quote! {#last_discriminant + #inc},
                                quote! {#ident #fields},
                            ),
                        ))
                    };
                    inc += 1;
                    ret
                },
            )
            .unzip();

        quote! {
            #[automatically_derived]
            impl #impl_generics
                 ::core::convert::TryFrom<#repr #ty_generics> for #ident
                 #where_clause
            {
                type Error = ::derive_more::TryFromError<#repr>;

                #[inline]
                fn try_from(value: #repr) -> ::core::result::Result<Self, Self::Error> {
                    #(#[allow(non_upper_case_globals)] const #consts: #repr = #discriminants;)*
                    match value {
                        #(#consts => ::core::result::Result::Ok(#ident::#variants),)*
                        _ => ::core::result::Result::Err(::derive_more::TryFromError::new(value)),
                    }
                }
            }
        }.to_tokens(tokens);
    }
}
