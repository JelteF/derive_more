use crate::utils::{add_extra_ty_param_bound, named_to_vec, unnamed_to_vec};
use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{
    parse::{Error, Result},
    spanned::Spanned,
    Attribute, Data, DeriveInput, Field, Fields, Ident, Index, Meta, NestedMeta, Type,
};

/// Provides the hook to expand `#[derive(Index)]` into an implementation of `From`
pub fn expand(input: &DeriveInput, trait_name: &'static str) -> Result<TokenStream> {
    let input_type = &input.ident;
    let state = State::new(input, trait_name, trait_name.to_lowercase())?;
    let (_, field_type, field_ident) = state.assert_single_enabled_field();
    let trait_path = &state.trait_path;

    let generics = add_extra_ty_param_bound(&input.generics, trait_path);
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let casted_trait = &quote!(<#field_type as #trait_path>);
    Ok(quote! {
        impl#impl_generics #trait_path for #input_type#ty_generics #where_clause
        {
            type Item = #casted_trait::Item;
            #[inline]
            fn next(&mut self) -> Option<Self::Item> {
                #casted_trait::next(&mut self.#field_ident)
            }
        }
    })
}

fn panic_one_field(trait_name: &str, trait_attr: &str) -> ! {
    panic!(format!(
        "derive({}) only works when forwarding to a single field. Try putting #[{}] or #[{}(ignore)] on the fields in the struct",
        trait_name, trait_attr, trait_attr,
    ))
}

struct State<'a> {
    trait_name: &'static str,
    trait_path: TokenStream,
    trait_attr: String,
    // input: &'a DeriveInput,
    named: bool,
    fields: Vec<&'a Field>,
    enabled: Vec<bool>,
}

impl<'a> State<'a> {
    fn new<'b>(
        input: &'b DeriveInput,
        trait_name: &'static str,
        trait_attr: String,
    ) -> Result<State<'b>> {
        let trait_ident = Ident::new(trait_name, Span::call_site());
        let trait_path = quote!(::core::iter::#trait_ident);
        let named;
        let fields: Vec<_> = match input.data {
            Data::Struct(ref data_struct) => match data_struct.fields {
                Fields::Unnamed(ref fields) => {
                    named = false;
                    unnamed_to_vec(fields)
                }
                Fields::Named(ref fields) => {
                    named = true;
                    named_to_vec(fields)
                }
                Fields::Unit => {
                    named = false;
                    vec![]
                }
            },
            _ => panic_one_field(&trait_name, &trait_attr),
        };

        let ignore_attrs: Result<Vec<_>> = fields
            .iter()
            .map(|f| get_ignore_meta(&trait_attr, &f.attrs))
            .collect();
        let first_match = ignore_attrs?.into_iter().filter_map(|at| at).next();
        let enabled: Result<Vec<_>> = if let Some(first_match) = first_match {
            fields
                .iter()
                .map(|f| is_enabled(&trait_attr, &f.attrs, first_match))
                .collect()
        } else {
            Ok(vec![true; fields.len()])
        };

        Ok(State {
            trait_name,
            trait_path,
            trait_attr,
            // input,
            fields,
            named,
            enabled: enabled?,
        })
    }

    fn assert_single_enabled_field(&self) -> (&'a Field, &'a Type, Box<dyn ToTokens>) {
        let enabled_fields = self.enabled_fields();
        if enabled_fields.len() != 1 {
            panic_one_field(self.trait_name, &self.trait_attr);
        };
        let mut field_idents = self.enabled_fields_idents();
        (
            enabled_fields[0],
            &enabled_fields[0].ty,
            field_idents.remove(0),
        )
    }

    fn enabled_fields(&self) -> Vec<&'a Field> {
        self.fields
            .iter()
            .zip(&self.enabled)
            .filter(|(_, ig)| **ig)
            .map(|(f, _)| *f)
            .collect()
    }

    fn field_idents(&self) -> Vec<Box<dyn ToTokens>> {
        if self.named {
            self.fields
                .iter()
                .map(|f| {
                    Box::new(
                        f.ident
                            .as_ref()
                            .expect("Tried to get field names of a tuple struct")
                            .clone(),
                    ) as Box<ToTokens>
                })
                .collect()
        } else {
            let count = self.fields.len();
            (0..count)
                .map(|i| Box::new(Index::from(i)) as Box<ToTokens>)
                .collect()
        }
    }

    fn enabled_fields_idents(&self) -> Vec<Box<dyn ToTokens>> {
        self.field_idents()
            .into_iter()
            .zip(&self.enabled)
            .filter(|(_, ig)| **ig)
            .map(|(f, _)| f)
            .collect()
    }
}

fn get_ignore_meta(trait_attr: &str, attrs: &[Attribute]) -> Result<Option<IgnoreMeta>> {
    let mut it = attrs
        .iter()
        .filter_map(|m| m.parse_meta().ok())
        .filter(|m| {
            if let Some(ident) = m.path().segments.first().map(|p| &p.ident) {
                ident == trait_attr
            } else {
                false
            }
        });

    let meta = if let Some(meta) = it.next() {
        meta
    } else {
        return Ok(None);
    };
    if let Some(meta2) = it.next() {
        return Err(Error::new(meta2.span(), "Too many formats given"));
    }
    let mut list = match meta.clone() {
        Meta::Path(_) => {
            return Ok(Some(IgnoreMeta::Enabled));
        }
        Meta::List(list) => list,
        _ => {
            return Err(Error::new(meta.span(), "Attribute format not supported1"));
        }
    };
    if list.nested.len() != 1 {
        return Err(Error::new(meta.span(), "Attribute format not supported2"));
    }
    let element = list.nested.pop().unwrap();
    let nested_meta = if let NestedMeta::Meta(meta) = element.value() {
        meta
    } else {
        return Err(Error::new(meta.span(), "Attribute format not supported3"));
    };
    if let Meta::Path(_) = nested_meta {
    } else {
        return Err(Error::new(meta.span(), "Attribute format not supported4"));
    }
    let ident = if let Some(ident) = nested_meta.path().segments.first().map(|p| &p.ident) {
        ident
    } else {
        return Err(Error::new(meta.span(), "Attribute format not supported5"));
    };
    if ident != "ignore" {
        return Err(Error::new(meta.span(), "Attribute format not supported6"));
    }
    Ok(Some(IgnoreMeta::Ignored))
}

fn is_enabled(trait_attr: &str, attrs: &[Attribute], first_match: IgnoreMeta) -> Result<bool> {
    let ignore_meta = if let Some(ignore_meta) = get_ignore_meta(trait_attr, attrs)? {
        ignore_meta
    } else {
        if first_match == IgnoreMeta::Enabled {
            return Ok(false);
        }
        return Ok(true);
    };
    Ok(ignore_meta == IgnoreMeta::Enabled)
}

#[derive(PartialEq, Eq, Clone, Copy)]
enum IgnoreMeta {
    Enabled,
    Ignored,
}
