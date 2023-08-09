use crate::utils::{
    add_where_clauses_for_new_ident, AttrParams, MultiFieldData, State,
};
use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};
use syn::{
    parse::{discouraged::Speculative, Parse, ParseStream, Result},
    spanned::Spanned,
    DeriveInput, Field, Fields,
};

pub fn expand(input: &DeriveInput, trait_name: &'static str) -> Result<TokenStream> {
    let as_ref_type = format_ident!("__AsRefT");
    let state = State::with_type_bound(
        input,
        trait_name,
        "as_ref".into(),
        AttrParams::ignore_and_forward(),
        false,
    )?;
    let MultiFieldData {
        fields,
        input_type,
        members,
        infos,
        trait_path,
        impl_generics,
        ty_generics,
        where_clause,
        ..
    } = state.enabled_fields_data();
    let sub_items: Vec<_> = infos
        .iter()
        .zip(members.iter())
        .zip(fields)
        .map(|((info, member), field)| {
            let field_type = &field.ty;
            if info.forward {
                let trait_path = quote! { #trait_path<#as_ref_type> };
                let type_where_clauses = quote! {
                    where #field_type: #trait_path
                };
                let new_generics = add_where_clauses_for_new_ident(
                    &input.generics,
                    &[field],
                    &as_ref_type,
                    type_where_clauses,
                    false,
                );
                let (impl_generics, _, where_clause) = new_generics.split_for_impl();
                let casted_trait = quote! { <#field_type as #trait_path> };
                (
                    quote! { #casted_trait::as_ref(&#member) },
                    quote! { #impl_generics },
                    quote! { #where_clause },
                    quote! { #trait_path },
                    quote! { #as_ref_type },
                )
            } else {
                (
                    quote! { &#member },
                    quote! { #impl_generics },
                    quote! { #where_clause },
                    quote! { #trait_path<#field_type> },
                    quote! { #field_type },
                )
            }
        })
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
            fn as_ref(&self) -> &#return_types {
                #bodies
            }
        }
    )*})
}

enum StructAttribute {
    Forward,
}

impl StructAttribute {
    fn parse_attrs(attrs: impl AsRef<[syn::Attribute]>) -> syn::Result<Option<Self>> {
        attrs
            .as_ref()
            .iter()
            .filter(|attr| attr.path().is_ident("as_ref"))
            .try_fold(None, |mut attrs, attr| {
                let field_attr = attr.parse_args()?;
                if attrs.replace(field_attr).is_some() {
                    Err(syn::Error::new(
                        attr.path().span(),
                        "only single #[as_ref(...)] attribute is allowed here",
                    ))
                } else {
                    Ok(attrs)
                }
            })
    }
}

impl Parse for StructAttribute {
    fn parse(input: ParseStream) -> Result<Self> {
        input.parse::<syn::Path>().and_then(|path| {
            if path.is_ident("forward") {
                Ok(Self::Forward)
            } else {
                Err(syn::Error::new(path.span(), "unknown"))
            }
        })
    }
}

enum FieldAttribute {
    AsRef,
    Forward,
    Ignore,
}

impl FieldAttribute {
    fn parse_attrs(attrs: impl AsRef<[syn::Attribute]>) -> syn::Result<Option<Self>> {
        attrs
            .as_ref()
            .iter()
            .filter(|attr| attr.path().is_ident("as_ref"))
            .try_fold(None, |mut attrs, attr| {
                let field_attr = attr.parse_args()?;
                if attrs.replace(field_attr).is_some() {
                    Err(syn::Error::new(
                        attr.path().span(),
                        "only single #[as_ref(...)] attribute is allowed here",
                    ))
                } else {
                    Ok(attrs)
                }
            })
    }
}

impl Parse for FieldAttribute {
    fn parse(input: ParseStream) -> Result<Self> {
        let ahead = input.fork();
        match ahead.parse::<syn::Path>() {
            Ok(p) if p.is_ident("forward") => {
                input.advance_to(&ahead);
                Ok(Self::Forward)
            }
            Ok(p) if p.is_ident("ignore") => {
                input.advance_to(&ahead);
                Ok(Self::Ignore)
            }
            _ => Ok(Self::AsRef),
        }
    }
}

struct FieldWithArgs<'a> {
    forward: bool,
    field: &'a Field,
}

fn extract(input: &'_ syn::DeriveInput) -> syn::Result<Vec<FieldWithArgs<'_>>> {
    let data = match &input.data {
        syn::Data::Struct(data) => Ok(data),
        syn::Data::Enum(e) => Err(syn::Error::new(
            e.enum_token.span(),
            "`AsRef` cannot be derived for enums",
        )),
        syn::Data::Union(u) => Err(syn::Error::new(
            u.union_token.span(),
            "`AsRef` cannot be derived for unions",
        )),
    }?;

    if let Some(struct_attr) = StructAttribute::parse_attrs(&input.attrs)? {
        let mut fields = data.fields.iter();

        let field = fields.next().ok_or_else(|| {
            syn::Error::new(
                Span::call_site(),
                "#[as_ref(...)] can only be applied to structs with exactly one field",
            )
        })?;

        if FieldAttribute::parse_attrs(&field.attrs)?.is_some() {
            return Err(syn::Error::new(
                field.span(),
                "#[as_ref(...)] cannot be applied to both struct and field",
            ));
        }

        if let Some(other_field) = fields.next() {
            return Err(syn::Error::new(
                other_field.span(),
                "#[as_ref(...)] can only be applied to structs with exactly one field",
            ));
        }

        let forward = matches!(struct_attr, StructAttribute::Forward);

        Ok(vec![FieldWithArgs { field, forward }])
    } else {
        extract_many(&data.fields)
    }
}

fn extract_many(fields: &'_ Fields) -> syn::Result<Vec<FieldWithArgs<'_>>> {
    let attrs = fields
        .iter()
        .map(|field| FieldAttribute::parse_attrs(&field.attrs))
        .collect::<syn::Result<Vec<_>>>()?;

    let present_attrs = attrs
        .iter()
        .filter_map(|attr| attr.as_ref())
        .collect::<Vec<_>>();

    let all = present_attrs
        .iter()
        .all(|attr| matches!(attr, FieldAttribute::Ignore));

    if !all
        && present_attrs
            .iter()
            .any(|attr| matches!(attr, FieldAttribute::Ignore))
    {
        return Err(syn::Error::new(
            Span::call_site(),
            "#[as_ref(ignore)] cannot be used with others",
        ));
    }

    if all {
        Ok(fields
            .iter()
            .zip(attrs)
            .filter(|(_, attr)| attr.is_none())
            .map(|(field, _)| FieldWithArgs {
                field,
                forward: false,
            })
            .collect())
    } else {
        Ok(fields
            .iter()
            .zip(attrs)
            .filter_map(|(field, attr)| match attr {
                Some(FieldAttribute::AsRef) => Some(FieldWithArgs {
                    field,
                    forward: false,
                }),
                Some(FieldAttribute::Forward) => Some(FieldWithArgs {
                    forward: true,
                    field,
                }),
                Some(FieldAttribute::Ignore) => unreachable!(),
                None => None,
            })
            .collect())
    }
}
