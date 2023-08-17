//! Implementations of [`AsRef`]/[`AsMut`] derive macros.

#[cfg(feature = "as_mut")]
pub(crate) mod r#mut;
#[cfg(feature = "as_ref")]
pub(crate) mod r#ref;

use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{
    parse::{discouraged::Speculative as _, Parse, ParseStream},
    parse_quote,
    spanned::Spanned,
    Token,
};

use crate::utils::{forward, skip, Either, Spanning};

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

    let expansions = if StructAttribute::parse_attrs(&input.attrs, attr_ident)?
        .is_some()
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
            forward: true,
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
                        forward: false,
                    })
                })
                .collect()
        } else {
            data.fields
                .iter()
                .enumerate()
                .zip(attrs)
                .filter_map(|((i, field), attr)| match attr.map(Spanning::into_inner) {
                    attr @ Some(FieldAttribute::Empty | FieldAttribute::Forward(_)) => {
                        Some(Expansion {
                            trait_info,
                            ident: &input.ident,
                            generics: &input.generics,
                            field,
                            field_index: i,
                            forward: matches!(attr, Some(FieldAttribute::Forward(_))),
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

    /// Indicator whether `forward` implementation should be expanded.
    forward: bool,
}

impl<'a> ToTokens for Expansion<'a> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let field_ty = &self.field.ty;
        let field_ident = self.field.ident.as_ref().map_or_else(
            || Either::Right(syn::Index::from(self.field_index)),
            Either::Left,
        );

        let return_ty = if self.forward {
            quote! { __AsT }
        } else {
            quote! { #field_ty }
        };

        let (trait_ident, method_ident, mut_) = &self.trait_info;
        let trait_ty = quote! { ::core::convert::#trait_ident <#return_ty> };

        let ty_ident = &self.ident;
        let mut generics = self.generics.clone();
        if self.forward {
            generics.params.push(parse_quote! { #return_ty });
            generics
                .make_where_clause()
                .predicates
                .extend::<[syn::WherePredicate; 2]>([
                    parse_quote! { #return_ty: ?::core::marker::Sized },
                    parse_quote! { #field_ty: #trait_ty },
                ]);
        }
        let (impl_gens, _, where_clause) = generics.split_for_impl();
        let (_, ty_gens, _) = self.generics.split_for_impl();

        let mut body = quote! { & #mut_ self.#field_ident };
        if self.forward {
            body = quote! {
                <#field_ty as #trait_ty>::#method_ident(#body)
            };
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
        .to_tokens(tokens)
    }
}

/// Representation of an [`AsRef`]/[`AsMut`] derive macro struct container attribute.
///
/// ```rust,ignore
/// #[as_ref(forward)]
/// ```
type StructAttribute = forward::Attribute;

/// Representation of an [`AsRef`]/[`AsMut`] derive macro field attribute.
///
/// ```rust,ignore
/// #[as_ref]
/// #[as_ref(forward)]
/// #[as_ref(skip)] #[as_ref(ignore)]
/// ```
enum FieldAttribute {
    Empty,
    Forward(forward::Attribute),
    Skip(skip::Attribute),
}

impl Parse for FieldAttribute {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        if input.is_empty() {
            return Ok(Self::Empty);
        }

        let ahead = input.fork();
        if let Ok(attr) = ahead.parse::<forward::Attribute>() {
            input.advance_to(&ahead);
            return Ok(Self::Forward(attr));
        }

        let ahead = input.fork();
        if let Ok(attr) = ahead.parse::<skip::Attribute>() {
            input.advance_to(&ahead);
            return Ok(Self::Skip(attr));
        }

        Err(syn::Error::new(
            input.span(),
            "only `forward` or `skip`/`ignore` allowed here",
        ))
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
        attrs.iter()
            .filter(|attr| attr.path().is_ident(attr_ident))
            .try_fold(None, |mut attrs, attr| {
                let parsed = Spanning::new(if matches!(attr.meta, syn::Meta::Path(_)) {
                    Self::Empty
                } else {
                    attr.parse_args()?
                }, attr.span());
                if attrs.replace(parsed).is_some() {
                    Err(syn::Error::new(
                        attr.span(),
                        format!("only single `#[{attr_ident}(...)]` attribute is allowed here"),
                    ))
                } else {
                    Ok(attrs)
                }
            })
    }
}
