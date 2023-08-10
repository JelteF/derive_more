use crate::utils::{add_where_clauses_for_new_ident, Either};
use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::{
    parse::{Parse, ParseStream, Result},
    spanned::Spanned,
    DeriveInput, Field, Fields, Index, Token,
};

pub fn expand(
    input: &DeriveInput,
    trait_ident: &Ident,
    method_ident: &Ident,
    conv_type: &Ident,
    mutability: Option<&Token![mut]>,
) -> syn::Result<TokenStream> {
    let trait_path = quote! { ::derive_more::#trait_ident };

    let field_args = extract_field_args(input, trait_ident, method_ident)?;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let input_type = &input.ident;

    let sub_items: Vec<_> = field_args
        .into_iter()
        .map(
            |FieldArgs {
                 forward,
                 field,
                 ident,
             }| {
                let member = quote! { self.#ident };
                let field_type = &field.ty;
                if forward {
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
                }
            },
        )
        .collect();
    let bodies = sub_items.iter().map(|i| &i.0);
    let impl_generics = sub_items.iter().map(|i| &i.1);
    let where_clauses = sub_items.iter().map(|i| &i.2);
    let trait_paths = sub_items.iter().map(|i| &i.3);
    let return_types = sub_items.iter().map(|i| &i.4);

    Ok(quote! {#(
        #[automatically_derived]
        impl #impl_generics #trait_paths for #input_type #ty_generics #where_clauses {
            #[inline]
            fn #method_ident(& #mutability self) -> #return_types {
                #bodies
            }
        }
    )*})
}

struct StructAttr<'a> {
    args: StructAttrArgs,
    attr: &'a syn::Attribute,
}

enum StructAttrArgs {
    Forward,
}

impl<'a> StructAttr<'a> {
    fn parse_attrs(
        attrs: &'a [syn::Attribute],
        method_ident: &Ident,
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
                        format!(
                            "only single `#[{}(...)]` attribute is allowed here",
                            method_ident
                        ),
                    ))
                } else {
                    Ok(attrs)
                }
            })
    }
}

impl Parse for StructAttrArgs {
    fn parse(input: ParseStream) -> Result<Self> {
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

struct FieldAttr<'a> {
    attr: &'a syn::Attribute,
    args: FieldAttrArgs,
}

enum FieldAttrArgs {
    AsRef,
    Forward,
    Ignore,
}

impl<'a> FieldAttr<'a> {
    fn parse_attrs(
        attrs: &'a [syn::Attribute],
        method_ident: &Ident,
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
                        format!(
                            "only single `#[{}(...)]` attribute is allowed here",
                            method_ident
                        ),
                    ))
                } else {
                    Ok(attrs)
                }
            })
    }
}

impl FieldAttrArgs {
    fn parse_attr(attr: &syn::Attribute) -> syn::Result<Self> {
        if matches!(attr.meta, syn::Meta::Path(_)) {
            return Ok(Self::AsRef);
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

struct FieldArgs<'a> {
    forward: bool,
    field: &'a Field,
    ident: Either<&'a Ident, Index>,
}

impl<'a> FieldArgs<'a> {
    fn new(field: &'a Field, forward: bool, index: usize) -> Self {
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

fn extract_field_args<'a>(
    input: &'a syn::DeriveInput,
    trait_ident: &Ident,
    method_ident: &Ident,
) -> syn::Result<Vec<FieldArgs<'a>>> {
    let data = match &input.data {
        syn::Data::Struct(data) => Ok(data),
        syn::Data::Enum(e) => Err(syn::Error::new(
            e.enum_token.span(),
            format!("`{}` cannot be derived for enums", trait_ident),
        )),
        syn::Data::Union(u) => Err(syn::Error::new(
            u.union_token.span(),
            format!("`{}` cannot be derived for unions", trait_ident),
        )),
    }?;

    if let Some(struct_attr) = StructAttr::parse_attrs(&input.attrs, method_ident)? {
        let mut fields = data.fields.iter();

        let field = fields.next().ok_or_else(|| {
            syn::Error::new(
                struct_attr.attr.span(),
                format!("`#[{}(...)]` can only be applied to structs with exactly one field", method_ident),
            )
        })?;

        if FieldAttr::parse_attrs(&field.attrs, method_ident)?.is_some() {
            return Err(syn::Error::new(
                field.span(),
                format!(
                    "`#[{}(...)]` cannot be applied to both struct and field",
                    method_ident
                ),
            ));
        }

        if let Some(other_field) = fields.next() {
            return Err(syn::Error::new(
                other_field.span(),
                format!("`#[{}(...)]` can only be applied to structs with exactly one field", method_ident),
            ));
        }

        let forward = matches!(struct_attr.args, StructAttrArgs::Forward);

        Ok(vec![FieldArgs::new(field, forward, 0)])
    } else {
        extract_many(&data.fields, method_ident)
    }
}

fn extract_many<'a>(
    fields: &'a Fields,
    method_ident: &Ident,
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
            return Err(syn::Error::new(attr.attr.span(), format!("`#[{0}(ignore)]` cannot be used in the same struct as other `#[{0}(...)]` attributes", method_ident)));
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
                Some(FieldAttrArgs::AsRef) => Some(FieldArgs::new(field, false, i)),
                Some(FieldAttrArgs::Forward) => Some(FieldArgs::new(field, true, i)),
                Some(FieldAttrArgs::Ignore) => unreachable!(),
                None => None,
            })
            .collect())
    }
}
