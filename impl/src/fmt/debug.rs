//! Implementation of a [`fmt::Debug`] derive macro.
//!
//! [`fmt::Debug`]: std::fmt::Debug

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{
    parse::{Parse, ParseStream},
    parse_quote,
    spanned::Spanned as _,
    Ident,
};

use super::{ContainerAttributes, FmtAttribute};

/// Expands a [`fmt::Debug`] derive macro.
///
/// [`fmt::Debug`]: std::fmt::Debug
pub fn expand(input: &syn::DeriveInput, _: &str) -> syn::Result<TokenStream> {
    let attrs = ContainerAttributes::parse_attrs(&input.attrs, "Debug")?;
    let ident = &input.ident;

    let (bounds, body) = match &input.data {
        syn::Data::Struct(s) => expand_struct(attrs, ident, s),
        syn::Data::Enum(e) => expand_enum(attrs, e),
        syn::Data::Union(_) => {
            return Err(syn::Error::new(
                input.span(),
                "`Debug` cannot be derived for unions",
            ));
        }
    }?;

    let (impl_gens, ty_gens, where_clause) = {
        let (impl_gens, ty_gens, where_clause) = input.generics.split_for_impl();
        let mut where_clause = where_clause
            .cloned()
            .unwrap_or_else(|| parse_quote! { where });
        where_clause.predicates.extend(bounds);
        (impl_gens, ty_gens, where_clause)
    };

    Ok(quote! {
        #[automatically_derived]
        impl #impl_gens ::core::fmt::Debug for #ident #ty_gens
             #where_clause
        {
            fn fmt(
                &self, __derive_more_f: &mut ::core::fmt::Formatter<'_>
            ) -> ::core::fmt::Result {
                #body
            }
        }
    })
}

/// Expands a [`fmt::Debug`] derive macro for the provided struct.
///
/// [`fmt::Debug`]: std::fmt::Debug
fn expand_struct(
    attrs: ContainerAttributes,
    ident: &Ident,
    s: &syn::DataStruct,
) -> syn::Result<(Vec<syn::WherePredicate>, TokenStream)> {
    let s = Expansion {
        attr: &attrs,
        fields: &s.fields,
        ident,
    };
    s.validate_attrs()?;
    let bounds = s.generate_bounds()?;
    let body = s.generate_body()?;

    let vars = s.fields.iter().enumerate().map(|(i, f)| {
        let var = f.ident.clone().unwrap_or_else(|| format_ident!("_{i}"));
        let member = f
            .ident
            .clone()
            .map_or_else(|| syn::Member::Unnamed(i.into()), syn::Member::Named);
        quote! { let #var = &&self.#member; }
    });

    let body = quote! {
        #( #vars )*
        #body
    };

    Ok((bounds, body))
}

/// Expands a [`fmt::Debug`] derive macro for the provided enum.
///
/// [`fmt::Debug`]: std::fmt::Debug
fn expand_enum(
    mut attrs: ContainerAttributes,
    e: &syn::DataEnum,
) -> syn::Result<(Vec<syn::WherePredicate>, TokenStream)> {
    if let Some(enum_fmt) = attrs.fmt.as_ref() {
        return Err(syn::Error::new_spanned(
            enum_fmt,
            "`#[debug(\"...\", ...)]` attribute is not allowed on enum, place it on its variants \
             instead",
        ));
    }

    let (bounds, match_arms) = e.variants.iter().try_fold(
        (Vec::new(), TokenStream::new()),
        |(mut bounds, mut arms), variant| {
            let ident = &variant.ident;

            attrs.fmt = variant
                .attrs
                .iter()
                .filter(|attr| attr.path().is_ident("debug"))
                .try_fold(None, |mut attrs, attr| {
                    let attr = attr.parse_args::<FmtAttribute>()?;
                    attrs.replace(attr).map_or(Ok(()), |dup| {
                        Err(syn::Error::new(
                            dup.span(),
                            "multiple `#[debug(\"...\", ...)]` attributes aren't allowed",
                        ))
                    })?;
                    Ok::<_, syn::Error>(attrs)
                })?;

            let v = Expansion {
                attr: &attrs,
                fields: &variant.fields,
                ident,
            };
            v.validate_attrs()?;
            let arm_body = v.generate_body()?;
            bounds.extend(v.generate_bounds()?);

            let fields_idents =
                variant.fields.iter().enumerate().map(|(i, f)| {
                    f.ident.clone().unwrap_or_else(|| format_ident!("_{i}"))
                });
            let matcher = match variant.fields {
                syn::Fields::Named(_) => {
                    quote! { Self::#ident { #( #fields_idents ),* } }
                }
                syn::Fields::Unnamed(_) => {
                    quote! { Self::#ident ( #( #fields_idents ),* ) }
                }
                syn::Fields::Unit => quote! { Self::#ident },
            };

            arms.extend([quote! { #matcher => { #arm_body }, }]);

            Ok::<_, syn::Error>((bounds, arms))
        },
    )?;

    let body = match_arms
        .is_empty()
        .then(|| quote! { match *self {} })
        .unwrap_or_else(|| quote! { match self { #match_arms } });

    Ok((bounds, body))
}

/// Representation of a [`fmt::Debug`] derive macro field attribute.
///
/// ```rust,ignore
/// #[debug("<fmt_literal>", <fmt_args>)]
/// #[debug(skip)]
/// ```
///
/// [`fmt::Debug`]: std::fmt::Debug
enum FieldAttribute {
    /// [`fmt`] attribute.
    ///
    /// [`fmt`]: std::fmt
    Fmt(FmtAttribute),

    /// Attribute for skipping field.
    Skip,
}

impl FieldAttribute {
    /// Parses a [`FieldAttribute`] from the provided [`syn::Attribute`]s.
    fn parse_attrs(attrs: impl AsRef<[syn::Attribute]>) -> syn::Result<Option<Self>> {
        Ok(attrs
            .as_ref()
            .iter()
            .filter(|attr| attr.path().is_ident("debug"))
            .try_fold(None, |mut attrs, attr| {
                let field_attr = attr.parse_args::<FieldAttribute>()?;
                if let Some((path, _)) = attrs.replace((attr.path(), field_attr)) {
                    Err(syn::Error::new(
                        path.span(),
                        "only single `#[debug(...)]` attribute is allowed here",
                    ))
                } else {
                    Ok(attrs)
                }
            })?
            .map(|(_, attr)| attr))
    }
}

impl Parse for FieldAttribute {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        FmtAttribute::check_legacy_fmt(input)?;

        if input.peek(syn::LitStr) {
            input.parse().map(Self::Fmt)
        } else {
            let _ = input.parse::<syn::Path>().and_then(|p| {
                if ["skip", "ignore"].into_iter().any(|i| p.is_ident(i)) {
                    Ok(p)
                } else {
                    Err(syn::Error::new(
                        p.span(),
                        "unknown attribute, expected `skip` or `ignore`",
                    ))
                }
            })?;
            Ok(Self::Skip)
        }
    }
}

/// Helper struct to generate [`Debug::fmt()`] implementation body and trait
/// bounds for a struct or an enum variant.
///
/// [`Debug::fmt()`]: std::fmt::Debug::fmt()
#[derive(Debug)]
struct Expansion<'a> {
    attr: &'a ContainerAttributes,

    /// Struct or enum [`Ident`](struct@Ident).
    ident: &'a Ident,

    /// Struct or enum [`syn::Fields`].
    fields: &'a syn::Fields,
}

impl<'a> Expansion<'a> {
    /// Validates attributes of this [`Expansion`] to be consistent.
    fn validate_attrs(&self) -> syn::Result<()> {
        if self.attr.fmt.is_some() {
            for field_attr in self
                .fields
                .iter()
                .map(|f| FieldAttribute::parse_attrs(&f.attrs))
            {
                if let Some(FieldAttribute::Fmt(fmt)) = field_attr? {
                    return Err(syn::Error::new_spanned(
                        fmt,
                        "`#[debug(...)]` attributes are not allowed on fields when \
                         `#[debug(\"...\", ...)]` is specified on struct or variant",
                    ));
                }
            }
        }
        Ok(())
    }

    /// Generates [`Debug::fmt()`] implementation for a struct or an enum variant.
    ///
    /// [`Debug::fmt()`]: std::fmt::Debug::fmt()
    fn generate_body(&self) -> syn::Result<TokenStream> {
        if let Some(fmt_attr) = &self.attr.fmt {
            return Ok(quote! { ::core::write!(__derive_more_f, #fmt_attr) });
        };

        match self.fields {
            syn::Fields::Unit => {
                let ident = self.ident.to_string();
                Ok(quote! {
                    ::core::fmt::Formatter::write_str(
                        __derive_more_f,
                        #ident,
                    )
                })
            }
            syn::Fields::Unnamed(unnamed) => {
                let mut exhaustive = true;
                let ident_str = self.ident.to_string();

                let out = quote! {
                    &mut ::derive_more::__private::debug_tuple(
                        __derive_more_f,
                        #ident_str,
                    )
                };
                let out = unnamed.unnamed.iter().enumerate().try_fold(
                    out,
                    |out, (i, field)| match FieldAttribute::parse_attrs(&field.attrs)? {
                        Some(FieldAttribute::Skip) => {
                            exhaustive = false;
                            Ok::<_, syn::Error>(out)
                        }
                        Some(FieldAttribute::Fmt(fmt)) => Ok(quote! {
                            ::derive_more::__private::DebugTuple::field(
                                #out,
                                &::core::format_args!(#fmt),
                            )
                        }),
                        None => {
                            let ident = format_ident!("_{i}");
                            Ok(quote! {
                                ::derive_more::__private::DebugTuple::field(#out, #ident)
                            })
                        }
                    },
                )?;
                Ok(if exhaustive {
                    quote! { ::derive_more::__private::DebugTuple::finish(#out) }
                } else {
                    quote! { ::derive_more::__private::DebugTuple::finish_non_exhaustive(#out) }
                })
            }
            syn::Fields::Named(named) => {
                let mut exhaustive = true;
                let ident = self.ident.to_string();

                let out = quote! {
                    &mut ::core::fmt::Formatter::debug_struct(
                        __derive_more_f,
                        #ident,
                    )
                };
                let out = named.named.iter().try_fold(out, |out, field| {
                        let field_ident = field.ident.as_ref().unwrap_or_else(|| {
                            unreachable!("`syn::Fields::Named`");
                        });
                        let field_str = field_ident.to_string();
                        match FieldAttribute::parse_attrs(&field.attrs)? {
                            Some(FieldAttribute::Skip) => {
                                exhaustive = false;
                                Ok::<_, syn::Error>(out)
                            }
                            Some(FieldAttribute::Fmt(fmt)) => Ok(quote! {
                                ::core::fmt::DebugStruct::field(
                                    #out,
                                    #field_str,
                                    &::core::format_args!(#fmt),
                                )
                            }),
                            None => Ok(quote! {
                                ::core::fmt::DebugStruct::field(#out, #field_str, #field_ident)
                            }),
                        }
                    })?;
                Ok(if exhaustive {
                    quote! { ::core::fmt::DebugStruct::finish(#out) }
                } else {
                    quote! { ::core::fmt::DebugStruct::finish_non_exhaustive(#out) }
                })
            }
        }
    }

    /// Generates trait bounds for a struct or an enum variant.
    fn generate_bounds(&self) -> syn::Result<Vec<syn::WherePredicate>> {
        let mut out = self.attr.bounds.0.clone().into_iter().collect::<Vec<_>>();

        if let Some(fmt) = self.attr.fmt.as_ref() {
            out.extend(fmt.bounded_types(self.fields).map(|(ty, trait_name)| {
                let trait_name = format_ident!("{trait_name}");
                parse_quote! { #ty: ::core::fmt::#trait_name }
            }));
            Ok(out)
        } else {
            self.fields.iter().try_fold(out, |mut out, field| {
                let ty = &field.ty;
                match FieldAttribute::parse_attrs(&field.attrs)? {
                    Some(FieldAttribute::Fmt(attr)) => {
                        out.extend(attr.bounded_types(self.fields).map(
                            |(ty, trait_name)| {
                                let trait_name = format_ident!("{trait_name}");
                                parse_quote! { #ty: ::core::fmt::#trait_name }
                            },
                        ));
                    }
                    Some(FieldAttribute::Skip) => {}
                    None => out.extend([parse_quote! { #ty: ::core::fmt::Debug }]),
                }
                Ok(out)
            })
        }
    }
}
