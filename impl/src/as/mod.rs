//! Implementations of [`AsRef`]/[`AsMut`] derive macros.

pub(crate) mod r#mut;
pub(crate) mod r#ref;

use std::borrow::Cow;

use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{
    parse::{discouraged::Speculative as _, Parse, ParseStream},
    parse_quote,
    punctuated::Punctuated,
    spanned::Spanned,
    Token,
};

use crate::utils::{
    contains_any_of, forward, skip, Either, HashSet, ParamSearch, Spanning,
};

/// Expands an [`AsRef`]/[`AsMut`] derive macro.
pub fn expand(
    input: &syn::DeriveInput,
    trait_info: ExpansionCtx<'_>,
) -> syn::Result<TokenStream> {
    let (trait_ident, attr_ident, _) = trait_info;

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

    let expansions = if let Some(attr) =
        StructAttribute::parse_attrs(&input.attrs, attr_ident)?
    {
        if data.fields.len() != 1 {
            return Err(syn::Error::new(
                if data.fields.is_empty() {
                    data.struct_token.span
                } else {
                    data.fields.span()
                },
                format!(
                    "`#[{attr_ident}(...)]` attribute can only be placed on structs with exactly \
                     one field",
                ),
            ));
        }

        let field = data.fields.iter().next().unwrap();
        if FieldAttribute::parse_attrs(&field.attrs, attr_ident)?.is_some() {
            return Err(syn::Error::new(
                field.span(),
                format!("`#[{attr_ident}(...)]` cannot be placed on both struct and its field"),
            ));
        }

        vec![Expansion {
            trait_info,
            ident: &input.ident,
            generics: &input.generics,
            field,
            field_index: 0,
            args: Some(attr.into_inner()),
        }]
    } else {
        let attrs = data
            .fields
            .iter()
            .map(|field| FieldAttribute::parse_attrs(&field.attrs, attr_ident))
            .collect::<syn::Result<Vec<_>>>()?;

        let present_attrs = attrs.iter().filter_map(Option::as_ref).collect::<Vec<_>>();

        let all = present_attrs
            .iter()
            .all(|attr| matches!(attr.item, FieldAttribute::Skip(_)));

        if !all {
            if let Some(skip_attr) = present_attrs.iter().find_map(|attr| {
                if let FieldAttribute::Skip(skip) = &attr.item {
                    Some(attr.as_ref().map(|_| skip))
                } else {
                    None
                }
            }) {
                return Err(syn::Error::new(
                    skip_attr.span(),
                    format!(
                        "`#[{attr_ident}({})]` cannot be used in the same struct with other \
                         `#[{attr_ident}(...)]` attributes",
                        skip_attr.name(),
                    ),
                ));
            }
        }

        if all {
            data.fields
                .iter()
                .enumerate()
                .zip(attrs)
                .filter_map(|((i, field), attr)| {
                    attr.is_none().then_some(Expansion {
                        trait_info,
                        ident: &input.ident,
                        generics: &input.generics,
                        field,
                        field_index: i,
                        args: None,
                    })
                })
                .collect()
        } else {
            data.fields
                .iter()
                .enumerate()
                .zip(attrs)
                .filter_map(|((i, field), attr)| match attr.map(Spanning::into_inner) {
                    Some(attr @ (FieldAttribute::Empty | FieldAttribute::Args(_))) => {
                        Some(Expansion {
                            trait_info,
                            ident: &input.ident,
                            generics: &input.generics,
                            field,
                            field_index: i,
                            args: attr.into_args(),
                        })
                    }
                    Some(FieldAttribute::Skip(_)) => unreachable!(),
                    None => None,
                })
                .collect()
        }
    };
    Ok(expansions
        .into_iter()
        .map(ToTokens::into_token_stream)
        .collect())
}

/// Type alias for an expansion context:
/// - [`syn::Ident`] of the derived trait.
/// - [`syn::Ident`] of the derived trait method.
/// - Optional `mut` token indicating [`AsMut`] expansion.
type ExpansionCtx<'a> = (&'a syn::Ident, &'a syn::Ident, Option<&'a Token![mut]>);

/// Expansion of a macro for generating [`AsRef`]/[`AsMut`] implementations for a single field of a
/// struct.
struct Expansion<'a> {
    /// [`ExpansionCtx] of the derived trait.
    trait_info: ExpansionCtx<'a>,

    /// [`syn::Ident`] of the struct.
    ident: &'a syn::Ident,

    /// [`syn::Generics`] of the struct.
    generics: &'a syn::Generics,

    /// [`syn::Field`] of the struct.
    field: &'a syn::Field,

    /// Index of the [`syn::Field`].
    field_index: usize,

    /// Arguments specifying which conversions should be generated
    args: Option<AsArgs>,
}

impl<'a> ToTokens for Expansion<'a> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let field_ty = &self.field.ty;
        let field_ident = self.field.ident.as_ref().map_or_else(
            || Either::Right(syn::Index::from(self.field_index)),
            Either::Left,
        );

        let (trait_ident, method_ident, mut_) = &self.trait_info;
        let ty_ident = &self.ident;

        let field_ref = quote! { & #mut_ self.#field_ident };

        let param_tys: HashSet<_> = self
            .generics
            .type_params()
            .map(|param| &param.ident)
            .collect();

        let param_lfs: HashSet<_> = self
            .generics
            .lifetimes()
            .map(|param| &param.lifetime.ident)
            .collect();

        let param_consts: HashSet<_> = self
            .generics
            .const_params()
            .map(|param| &param.ident)
            .collect();

        let param_search = ParamSearch {
            param_tys,
            param_lfs,
            param_consts,
        };

        let field_contains_param = contains_any_of(field_ty, &param_search);

        let is_blanket = matches!(&self.args, Some(AsArgs::Forward(_)));

        let return_tys = match &self.args {
            Some(AsArgs::Forward(_)) => {
                Either::Left(std::iter::once(Cow::Owned(parse_quote! { __AsT })))
            }
            Some(AsArgs::Types(tys)) => Either::Right(tys.iter().map(Cow::Borrowed)),
            None => Either::Left(std::iter::once(Cow::Borrowed(field_ty))),
        };

        for return_ty in return_tys {
            use ImplKind::*;

            /// The kind of impl generated, chosen based on attribute args.
            enum ImplKind {
                /// Returns a reference to a field.
                Direct,
                /// Calls `as_ref`/`as_mut` on a field
                Forwarded,
                /// Uses autoderef-based specialization to determine whether
                /// to use direct or forwarded based on whether the field
                /// and return type match.
                Specialized,
            }

            let impl_kind = 'ver: {
                if is_blanket {
                    break 'ver Forwarded;
                }

                if field_ty == return_ty.as_ref() {
                    break 'ver Direct;
                }

                if field_contains_param || contains_any_of(&return_ty, &param_search) {
                    break 'ver Forwarded;
                }

                Specialized
            };

            let trait_ty = quote! { ::core::convert::#trait_ident <#return_ty> };

            let generics = match &impl_kind {
                Forwarded => {
                    let mut generics = self.generics.clone();

                    generics
                        .make_where_clause()
                        .predicates
                        .push(parse_quote! { #field_ty: #trait_ty });

                    if is_blanket {
                        generics
                            .params
                            .push(parse_quote! { #return_ty: ?::core::marker::Sized });
                    }

                    Cow::Owned(generics)
                }
                Direct | Specialized => Cow::Borrowed(self.generics),
            };

            let (impl_gens, _, where_clause) = generics.split_for_impl();
            let (_, ty_gens, _) = self.generics.split_for_impl();

            let body = match &impl_kind {
                Direct => Cow::Borrowed(&field_ref),
                Forwarded => Cow::Owned(quote! {
                    <#field_ty as #trait_ty>::#method_ident(#field_ref)
                }),
                Specialized => Cow::Owned(quote! {
                    use ::derive_more::__private::ExtractRef as _;

                    let conv =
                        <::derive_more::__private::Conv<& #mut_ #field_ty, #return_ty>
                         as ::core::default::Default>::default();
                    (&&conv).__extract_ref(#field_ref)
                }),
            };

            quote! {
                #[automatically_derived]
                impl #impl_gens #trait_ty for #ty_ident #ty_gens #where_clause {
                    #[inline]
                    fn #method_ident(& #mut_ self) -> & #mut_ #return_ty {
                        #body
                    }
                }
            }
            .to_tokens(tokens);
        }
    }
}

/// Representation of an [`AsRef`]/[`AsMut`] derive macro struct container attribute.
///
/// ```rust,ignore
/// #[as_ref(forward)]
/// #[as_ref(<types>)]
/// ```
type StructAttribute = AsArgs;

/// Representation of an [`AsRef`]/[`AsMut`] derive macro field attribute.
///
/// ```rust,ignore
/// #[as_ref]
/// #[as_ref(forward)]
/// #[as_ref(<types>)]
/// #[as_ref(skip)] #[as_ref(ignore)]
/// ```
enum FieldAttribute {
    Empty,
    Args(AsArgs),
    Skip(skip::Attribute),
}

impl Parse for FieldAttribute {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        if input.is_empty() {
            return Ok(Self::Empty);
        }

        let ahead = input.fork();
        if let Ok(attr) = ahead.parse::<skip::Attribute>() {
            input.advance_to(&ahead);
            return Ok(Self::Skip(attr));
        }

        input.parse::<AsArgs>().map(Self::Args)
    }
}

impl FieldAttribute {
    /// Parses a [`FieldAttribute`] from the provided [`syn::Attribute`]s, preserving its [`Span`].
    ///
    /// [`Span`]: proc_macro2::Span
    fn parse_attrs(
        attrs: &[syn::Attribute],
        attr_ident: &syn::Ident,
    ) -> syn::Result<Option<Spanning<Self>>> {
        attrs
            .iter()
            .filter(|attr| attr.path().is_ident(attr_ident))
            .try_fold(None, |attrs: Option<Spanning<Self>>, attr| {
                let parsed = Spanning::new(
                    if matches!(attr.meta, syn::Meta::Path(_)) {
                        Self::Empty
                    } else {
                        attr.parse_args()?
                    },
                    attr.span(),
                );


                if let Some(prev) = attrs {
                    let span = prev.span.join(parsed.span).unwrap_or(prev.span);
                    if let Some(new) = prev.item.merge(parsed.item) {
                        Ok(Some(Spanning::new(new, span)))
                    } else {
                        Err(syn::Error::new(
                            parsed.span,
                            format!("only single `#[{attr_ident}(...)]` attribute is allowed here")
                        ))
                    }
                } else {
                    Ok(Some(parsed))
                }
            })
    }

    /// Extracts conversion arguments on the attribute, if any.
    fn into_args(self) -> Option<AsArgs> {
        match self {
            Self::Args(args) => Some(args),
            Self::Empty | Self::Skip(_) => None,
        }
    }

    /// Merges two [`FieldAttribute`]s, if possible
    fn merge(self, other: Self) -> Option<Self> {
        if let (Self::Args(args), Self::Args(more)) = (self, other) {
            args.merge(more).map(Self::Args)
        } else {
            None
        }
    }
}

/// Arguments specifying which conversions should be generated.
///
/// ```rust,ignore
/// #[as_ref(forward)]
/// #[as_ref(<types>)]
/// ```
enum AsArgs {
    /// Blanket impl, fully forwarding to the field type.
    Forward(forward::Attribute),

    /// Impl for specified types, which can include both the type of the field,
    /// and types for which the field type implements `AsRef`
    Types(Punctuated<syn::Type, Token![,]>),
}

impl AsArgs {
    /// Merges two [`AsArgs`], if possible
    fn merge(self, other: Self) -> Option<Self> {
        if let (Self::Types(mut tys), Self::Types(more)) = (self, other) {
            tys.extend(more);
            Some(Self::Types(tys))
        } else {
            None
        }
    }
}

impl Parse for AsArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let ahead = input.fork();
        if let Ok(attr) = ahead.parse::<forward::Attribute>() {
            input.advance_to(&ahead);
            return Ok(Self::Forward(attr));
        }

        input
            .parse_terminated(syn::Type::parse, Token![,])
            .map(Self::Types)
    }
}

impl StructAttribute {
    /// Parses a [`StructAttribute`] from the provided [`syn::Attribute`]s, preserving its [`Span`].
    ///
    /// [`Span`]: proc_macro2::Span
    fn parse_attrs(
        attrs: &[syn::Attribute],
        attr_ident: &syn::Ident,
    ) -> syn::Result<Option<Spanning<Self>>> {
        attrs.iter().filter(|attr| attr.path().is_ident(attr_ident))
            .try_fold(None, |attrs: Option<Spanning<Self>>, attr| {
                let parsed: Spanning<Self> = Spanning::new(attr.parse_args()?, attr.span());

                if let Some(prev) = attrs {
                    let span = prev.span.join(parsed.span).unwrap_or(prev.span);
                    if let Some(new) = prev.item.merge(parsed.item) {
                        Ok(Some(Spanning::new(new, span)))
                    } else {
                        Err(syn::Error::new(
                            parsed.span,
                            format!("only single `#[{attr_ident}(...)]` attribute is allowed here"),
                        ))
                    }
                } else {
                    Ok(Some(parsed))
                }
            })
    }
}
