//! Implementation of a [`TryFrom`] derive macro.

use std::mem;

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
            generics: input.generics.clone(),
            variants: data.variants.clone().into_iter().collect(),
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
struct ItemAttribute;

impl ItemAttribute {
    /// Parses am [`ItemAttribute`] from the provided [`syn::Attribute`]s.
    fn parse_attrs(attrs: impl AsRef<[syn::Attribute]>) -> syn::Result<Option<Self>> {
        attrs
            .as_ref()
            .iter()
            .filter(|attr| attr.path().is_ident("try_from"))
            .try_fold(None, |mut attrs, attr| {
                let mut parsed = None;
                attr.parse_nested_meta(|meta| {
                    if meta.path.is_ident("repr") {
                        parsed = Some(ItemAttribute);
                        Ok(())
                    } else {
                        Err(meta.error("only `repr` is allowed here"))
                    }
                })?;
                if mem::replace(&mut attrs, parsed).is_some() {
                    Err(syn::Error::new(
                        attr.span(),
                        "only single `#[try_from(repr)]` attribute is allowed here",
                    ))
                } else {
                    Ok(attrs)
                }
            })
    }
}

/// Representation of a [`#[repr(u/i*)]` Rust attribute][0].
///
/// **NOTE**: Disregards any non-integer representation `#[repr]`s.
///
/// ```rust,ignore
/// #[repr(<type>)]
/// ```
///
/// [0]: https://doc.rust-lang.org/reference/type-layout.html#primitive-representations
struct ReprAttribute(syn::Ident);

impl ReprAttribute {
    /// Parses a [`ReprAttribute`] from the provided [`syn::Attribute`]s.
    ///
    /// If there is no [`ReprAttribute`], then parses a [default `isize` discriminant][0].
    ///
    /// [0]: https://doc.rust-lang.org/reference/items/enumerations.html#discriminants
    fn parse_attrs(attrs: impl AsRef<[syn::Attribute]>) -> syn::Result<Self> {
        attrs
            .as_ref()
            .iter()
            .filter(|attr| attr.path().is_ident("repr"))
            .try_fold(None, |mut repr, attr| {
                attr.parse_nested_meta(|meta| {
                    if let Some(ident) = meta.path.get_ident() {
                        if matches!(
                            ident.to_string().as_str(),
                            "u8" | "u16"
                                | "u32"
                                | "u64"
                                | "u128"
                                | "usize"
                                | "i8"
                                | "i16"
                                | "i32"
                                | "i64"
                                | "i128"
                                | "isize"
                        ) {
                            repr = Some(ident.clone());
                            return Ok(());
                        }
                    }
                    // Ignore all other attributes that could have a body, e.g. `align`.
                    _ = meta.input.parse::<proc_macro2::Group>();
                    Ok(())
                })
                .map(|_| repr)
            })
            .map(|repr| {
                // Default discriminant is interpreted as `isize`:
                // https://doc.rust-lang.org/reference/items/enumerations.html#discriminants
                repr.unwrap_or_else(|| syn::Ident::new("isize", Span::call_site()))
            })
            .map(Self)
    }
}

/// Expansion of a macro for generating [`TryFrom`] implementation of an enum.
struct Expansion {
    /// `#[repr(u/i*)]` of the enum.
    repr: ReprAttribute,

    /// [`ItemAttribute`] of the enum.
    attr: Option<ItemAttribute>,

    /// [`syn::Ident`] of the enum.
    ident: syn::Ident,

    /// [`syn::Generics`] of the enum.
    generics: syn::Generics,

    /// [`syn::Variant`]s of the enum.
    variants: Vec<syn::Variant>,
}

impl ToTokens for Expansion {
    /// Expands [`TryFrom`] implementations for a struct.
    fn to_tokens(&self, tokens: &mut TokenStream) {
        if self.attr.is_none() {
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
                    if let Some(d) = discriminant {
                        last_discriminant = d.1.to_token_stream();
                        inc = 0;
                    }
                    let ret = {
                        let inc = Literal::usize_unsuffixed(inc);
                        fields.is_empty().then_some((
                            format_ident!("__DISCRIMINANT_{ident}"),
                            (
                                quote! { #last_discriminant + #inc },
                                quote! { #ident #fields },
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
            impl #impl_generics ::core::convert::TryFrom<#repr #ty_generics> for #ident
                 #where_clause
            {
                type Error = ::derive_more::TryFromReprError<#repr>;

                #[allow(non_upper_case_globals)]
                #[inline]
                fn try_from(val: #repr) -> ::core::result::Result<Self, Self::Error> {
                    #( const #consts: #repr = #discriminants; )*
                    match val {
                        #(#consts => ::core::result::Result::Ok(#ident::#variants),)*
                        _ => ::core::result::Result::Err(::derive_more::TryFromReprError::new(val)),
                    }
                }
            }
        }.to_tokens(tokens);
    }
}
