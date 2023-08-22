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

use crate::{
    parsing::Type,
    utils::{forward, skip, Either, Spanning},
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

    let expansions = if let Some(args) =
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
            args: Some(args.into_inner()),
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
                    attr @ Some(FieldAttribute::Empty | FieldAttribute::Args(_)) => {
                        Some(Expansion {
                            trait_info,
                            ident: &input.ident,
                            generics: &input.generics,
                            field,
                            field_index: i,
                            args: if let Some(FieldAttribute::Args(args)) = attr {
                                Some(args)
                            } else {
                                None
                            },
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

/// Expansion of a macro for generating [`AsRef`]/[`AsMut`] implementation for a single field of a
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

    /// Arguments on the attribute
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

        let (blanket, forward_impl, return_tys) = match &self.args {
            Some(AsArgs::Forward(_)) => {
                (true, true, Either::Left(std::iter::once(quote! { __AsT })))
            }
            Some(AsArgs::Types(tys)) => (
                false,
                true,
                Either::Right(tys.iter().map(ToTokens::into_token_stream)),
            ),
            None => (
                false,
                false,
                Either::Left(std::iter::once(quote! { #field_ty })),
            ),
        };

        return_tys
            .map(|return_ty| {
                let trait_ty = quote! { ::core::convert::#trait_ident <#return_ty> };

                let generics = if forward_impl {
                    let mut generics = self.generics.clone();
                    if blanket {
                        generics.params.push(parse_quote! { #return_ty });
                        generics
                            .make_where_clause()
                            .predicates
                            .push(parse_quote! { #return_ty: ?::core::marker::Sized });
                    }
                    generics
                        .make_where_clause()
                        .predicates
                        .push(parse_quote! { #field_ty: #trait_ty });

                    Cow::Owned(generics)
                } else {
                    Cow::Borrowed(self.generics)
                };

                let (impl_gens, _, where_clause) = generics.split_for_impl();
                let (_, ty_gens, _) = self.generics.split_for_impl();

                let mut body = quote! { & #mut_ self.#field_ident };
                if forward_impl {
                    body = quote! { <#field_ty as #trait_ty>::#method_ident(#body) };
                }

                quote! {
                    #[automatically_derived]
                    impl #impl_gens #trait_ty for #ty_ident #ty_gens #where_clause {
                        #[inline]
                        fn #method_ident(& #mut_ self) -> & #mut_ #return_ty {
                            #body
                        }
                    }
                }
            })
            .collect::<TokenStream>()
            .to_tokens(tokens);
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

/// Arguments specifying which conversions should be generated
enum AsArgs {
    /// Blanket impl, fully forwarding to the field type
    Forward(forward::Attribute),
    /// Forward implementation, but only impl for specified types
    Types(Punctuated<Type, Token![,]>),
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
                    match (prev.item, parsed.item) {
                        (Self::Args(AsArgs::Types(mut tys)), Self::Args(AsArgs::Types(more))) => {
                            tys.extend(more);
                            Ok(Some(Spanning::new(Self::Args(AsArgs::Types(tys)), span)))
                        }
                        _ => Err(syn::Error::new(
                            span,
                            format!("only single `#[{attr_ident}(...)]` attribute is allowed here")
                        ))
                    }
                } else {
                    Ok(Some(parsed))
                }
            })
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
            .parse_terminated(Type::parse, Token![,])
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
                    match (prev.item, parsed.item) {
                        (Self::Types(mut tys), Self::Types(more)) => {
                            tys.extend(more);
                            Ok(Some(Spanning::new(Self::Types(tys), span)))
                        },
                        _ => Err(syn::Error::new(
                            span,
                            format!("only single `#[{attr_ident}(...)]` attribute is allowed here"),
                        ))
                    }
                } else {
                    Ok(Some(parsed))
                }
            })
    }
}
