use std::collections::HashSet;

use proc_macro2::TokenStream;
use quote::quote;
use syn::{spanned::Spanned as _, Error, Result};

use crate::utils::{
    self, AttrParams, DeriveType, FullMetaInfo, MetaInfo, MultiFieldData, State,
};

pub fn expand(
    input: &syn::DeriveInput,
    trait_name: &'static str,
) -> Result<TokenStream> {
    let syn::DeriveInput {
        ident, generics, ..
    } = input;

    let state = State::with_attr_params(
        input,
        trait_name,
        quote!(::std::error),
        trait_name.to_lowercase(),
        allowed_attr_params(),
    )?;

    let type_params: HashSet<_> = generics
        .params
        .iter()
        .filter_map(|generic| match generic {
            syn::GenericParam::Type(ty) => Some(ty.ident.clone()),
            _ => None,
        })
        .collect();

    let (bounds, body) = match state.derive_type {
        DeriveType::Named | DeriveType::Unnamed => render_struct(&type_params, &state)?,
        DeriveType::Enum => render_enum(&type_params, &state)?,
    };

    let mut generics = generics.clone();

    if !type_params.is_empty() {
        let generic_parameters = generics.params.iter();
        generics = utils::add_extra_where_clauses(
            &generics,
            quote! {
                where
                    #ident<#(#generic_parameters),*>: ::std::fmt::Debug + ::std::fmt::Display
            },
        );
    }

    if !bounds.is_empty() {
        let bounds = bounds.iter();
        generics = utils::add_extra_where_clauses(
            &generics,
            quote! {
                where
                    #(#bounds: ::std::fmt::Debug + ::std::fmt::Display + ::std::error::Error + 'static),*
            },
        );
    }

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let render = quote! {
        impl#impl_generics ::std::error::Error for #ident#ty_generics #where_clause {
            fn source(&self) -> Option<&(dyn ::std::error::Error + 'static)> {
                #body
            }
        }
    };

    Ok(render)
}

fn render_struct(
    type_params: &HashSet<syn::Ident>,
    state: &State,
) -> Result<(HashSet<syn::Type>, TokenStream)> {
    let parsed_fields = parse_fields(&type_params, &state)?;

    let render = parsed_fields.render_as_struct();
    let bounds = parsed_fields.bounds;

    Ok((bounds, render))
}

fn render_enum(
    type_params: &HashSet<syn::Ident>,
    state: &State,
) -> Result<(HashSet<syn::Type>, TokenStream)> {
    let mut bounds = HashSet::new();
    let mut match_arms = Vec::new();
    let mut render_default_wildcard = false;

    for variant in state.enabled_variant_data().variants {
        let mut default_info = FullMetaInfo::default();
        default_info.enabled = true;

        let state = State::from_variant(
            state.input,
            state.trait_name,
            state.trait_module.clone(),
            state.trait_attr.clone(),
            allowed_attr_params(),
            variant,
            default_info,
        )?;

        let parsed_fields = parse_fields(&type_params, &state)?;

        match parsed_fields.render_as_enum_variant_match_arm() {
            Some(match_arm) => {
                match_arms.push(match_arm);
            }

            None => {
                render_default_wildcard = true;
            }
        }

        bounds.extend(parsed_fields.bounds.into_iter());
    }

    if !match_arms.is_empty() && render_default_wildcard {
        match_arms.push(quote!(_ => None));
    }

    let render = if !match_arms.is_empty() {
        quote! {
            match self {
                #(#match_arms),*
            }
        }
    } else {
        quote!(None)
    };

    Ok((bounds, render))
}

fn allowed_attr_params() -> AttrParams {
    AttrParams {
        enum_: vec!["ignore"],
        struct_: vec!["ignore"],
        variant: vec!["ignore"],
        field: vec!["ignore", "source"],
    }
}

struct ParsedFields<'input, 'state> {
    data: MultiFieldData<'input, 'state>,
    source: Option<usize>,
    bounds: HashSet<syn::Type>,
}

impl<'input, 'state> ParsedFields<'input, 'state> {
    fn new(data: MultiFieldData<'input, 'state>) -> Self {
        Self {
            data,
            source: None,
            bounds: HashSet::new(),
        }
    }
}

impl<'input, 'state> ParsedFields<'input, 'state> {
    fn render_as_struct(&self) -> TokenStream {
        match self.source {
            Some(source) => {
                let ident = &self.data.members[source];
                render_some(quote!(&#ident))
            }
            None => quote!(None),
        }
    }

    fn render_as_enum_variant_match_arm(&self) -> Option<TokenStream> {
        self.source.map(|source| {
            let pattern = self.data.matcher(&[source], &[quote!(source)]);
            let expr = render_some(quote!(source));

            quote!(#pattern => #expr)
        })
    }
}

fn render_some<T>(expr: T) -> TokenStream
where
    T: quote::ToTokens,
{
    quote!(Some(#expr as &(dyn ::std::error::Error + 'static)))
}

fn parse_fields<'input, 'state>(
    type_params: &HashSet<syn::Ident>,
    state: &'state State<'input>,
) -> Result<ParsedFields<'input, 'state>> {
    match state.derive_type {
        DeriveType::Named => {
            parse_fields_impl(type_params, state, |field, _| {
                let ident = field
                    .ident
                    .as_ref()
                    .expect("Internal error in macro implementation"); // TODO

                ident == "source"
            })
        }

        DeriveType::Unnamed => parse_fields_impl(type_params, state, |_, len| len == 1),

        _ => unreachable!(), // TODO
    }
}

fn parse_fields_impl<'input, 'state, P>(
    type_params: &HashSet<syn::Ident>,
    state: &'state State<'input>,
    is_valid_default_field_for_attr: P,
) -> Result<ParsedFields<'input, 'state>>
where
    P: Fn(&syn::Field, usize) -> bool,
{
    let MultiFieldData { fields, infos, .. } = state.enabled_fields_data();

    let iter = fields
        .iter()
        .zip(infos.iter().map(|info| &info.info))
        .enumerate()
        .map(|(index, (field, info))| (index, *field, info));

    let explicit_sources = iter.clone().filter(|(_, _, info)| match info.source {
        Some(true) => true,
        _ => false,
    });

    let inferred_sources = iter.filter(|(_, field, info)| match info.source {
        None => is_valid_default_field_for_attr(field, fields.len()),
        _ => false,
    });

    let source = assert_iter_contains_zero_or_one_item(
        explicit_sources,
        "Multiple `source` attributes specified. \
         Single attribute per struct/enum variant allowed.",
    )?;

    let source = match source {
        source @ Some(_) => source,
        None => assert_iter_contains_zero_or_one_item(
            inferred_sources,
            "Conflicting fields found. Consider specifying some \
             `#[error(...)]` attributes to resolve conflict.",
        )?,
    };

    let mut parsed_fields = ParsedFields::new(state.enabled_fields_data());

    if let Some((index, field, _)) = source {
        parsed_fields.source = Some(index);
        add_bound_if_type_parameter_used_in_type(
            &mut parsed_fields.bounds,
            type_params,
            &field.ty,
        );
    }

    Ok(parsed_fields)
}

fn assert_iter_contains_zero_or_one_item<'a>(
    mut iter: impl Iterator<Item = (usize, &'a syn::Field, &'a MetaInfo)>,
    error_msg: &str,
) -> Result<Option<(usize, &'a syn::Field, &'a MetaInfo)>> {
    let item = match iter.next() {
        Some(item) => item,
        None => return Ok(None),
    };

    if let Some((_, field, _)) = iter.next() {
        return Err(Error::new(field.span(), error_msg));
    }

    Ok(Some(item))
}

fn add_bound_if_type_parameter_used_in_type(
    bounds: &mut HashSet<syn::Type>,
    type_params: &HashSet<syn::Ident>,
    ty: &syn::Type,
) {
    if let Some(ty) = utils::get_if_type_parameter_used_in_type(type_params, ty) {
        bounds.insert(ty);
    }
}
