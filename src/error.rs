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

    let (bounds, source, backtrace) = match state.derive_type {
        DeriveType::Named | DeriveType::Unnamed => render_struct(&type_params, &state)?,
        DeriveType::Enum => render_enum(&type_params, &state)?,
    };

    let source = source.map(|source| {
        quote! {
            fn source(&self) -> Option<&(dyn ::std::error::Error + 'static)> {
                #source
            }
        }
    });

    let backtrace = backtrace.map(|backtrace| {
        quote! {
            fn backtrace(&self) -> Option<&::std::backtrace::Backtrace> {
                #backtrace
            }
        }
    });

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
            #source
            #backtrace
        }
    };

    Ok(render)
}

fn render_struct(
    type_params: &HashSet<syn::Ident>,
    state: &State,
) -> Result<(HashSet<syn::Type>, Option<TokenStream>, Option<TokenStream>)> {
    let parsed_fields = parse_fields(&type_params, &state)?;

    let source = parsed_fields.render_source_as_struct();
    let backtrace = parsed_fields.render_backtrace_as_struct();

    Ok((parsed_fields.bounds, source, backtrace))
}

fn render_enum(
    type_params: &HashSet<syn::Ident>,
    state: &State,
) -> Result<(HashSet<syn::Type>, Option<TokenStream>, Option<TokenStream>)> {
    let mut bounds = HashSet::new();
    let mut source_match_arms = Vec::new();
    let mut backtrace_match_arms = Vec::new();

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

        if let Some(expr) = parsed_fields.render_source_as_enum_variant_match_arm() {
            source_match_arms.push(expr);
        }

        if let Some(expr) = parsed_fields.render_backtrace_as_enum_variant_match_arm() {
            backtrace_match_arms.push(expr);
        }

        bounds.extend(parsed_fields.bounds.into_iter());
    }

    let render = |match_arms: &mut Vec<TokenStream>| {
        if !match_arms.is_empty() && match_arms.len() < state.variants.len() {
            match_arms.push(quote!(_ => None));
        }

        if !match_arms.is_empty() {
            let expr = quote! {
                match self {
                    #(#match_arms),*
                }
            };

            Some(expr)
        } else {
            None
        }
    };

    let source = render(&mut source_match_arms);
    let backtrace = render(&mut backtrace_match_arms);

    Ok((bounds, source, backtrace))
}

fn allowed_attr_params() -> AttrParams {
    AttrParams {
        enum_: vec!["ignore"],
        struct_: vec!["ignore"],
        variant: vec!["ignore"],
        field: vec!["ignore", "source", "backtrace"],
    }
}

struct ParsedFields<'input, 'state> {
    data: MultiFieldData<'input, 'state>,
    source: Option<usize>,
    backtrace: Option<usize>,
    bounds: HashSet<syn::Type>,
}

impl<'input, 'state> ParsedFields<'input, 'state> {
    fn new(data: MultiFieldData<'input, 'state>) -> Self {
        Self {
            data,
            source: None,
            backtrace: None,
            bounds: HashSet::new(),
        }
    }
}

impl<'input, 'state> ParsedFields<'input, 'state> {
    fn render_source_as_struct(&self) -> Option<TokenStream> {
        self.source.map(|source| {
            let ident = &self.data.members[source];
            render_some(quote!(&#ident))
        })
    }

    fn render_source_as_enum_variant_match_arm(&self) -> Option<TokenStream> {
        self.source.map(|source| {
            let pattern = self.data.matcher(&[source], &[quote!(source)]);
            let expr = render_some(quote!(source));
            quote!(#pattern => #expr)
        })
    }

    fn render_backtrace_as_struct(&self) -> Option<TokenStream> {
        self.render_backtrace_impl(
            |fields, index| {
                let expr = &fields.data.members[index];
                quote!(#expr)
            },
            |fields, index| {
                let expr = &fields.data.members[index];
                quote!(&#expr)
            },
            false,
        )
        .map(|(_, expr)| expr)
    }

    fn render_backtrace_as_enum_variant_match_arm(&self) -> Option<TokenStream> {
        self.render_backtrace_impl(
            |_, _| quote!(source),
            |_, _| quote!(backtrace),
            true,
        )
        .map(|(pattern, expr)| quote!(#pattern => #expr))
    }

    fn render_backtrace_impl<S, B>(
        &self,
        source_expr: S,
        backtrace_expr: B,
        is_rendering_enum_match_arm: bool,
    ) -> Option<(TokenStream, TokenStream)>
    where
        S: Fn(&ParsedFields, usize) -> TokenStream,
        B: Fn(&ParsedFields, usize) -> TokenStream,
    {
        // Disable proxying call to `backtrace` to `source` if
        // `source` explicitly marked with `#[error(not(backtrace))]`.
        let source = self.source.and_then(|source| {
            match self.data.infos[source].info.backtrace {
                Some(false) => None,
                _ => Some(source),
            }
        });

        let mut indices = Vec::new();
        let mut bindings = Vec::new();

        let mut bind = |index: usize, binding: &TokenStream| {
            if is_rendering_enum_match_arm {
                indices.push(index);
                bindings.push(binding.clone());
            }
        };

        let expr = match (source, self.backtrace) {
            (Some(source), backtrace) => {
                let source_expr = source_expr(self, source);
                bind(source, &source_expr);

                let backtrace_expr = backtrace
                    .filter(|backtrace| source != *backtrace)
                    .map(|backtrace| {
                        let backtrace_expr = backtrace_expr(self, backtrace);
                        bind(backtrace, &backtrace_expr);

                        quote!(.or_else(|| Some(#backtrace_expr)))
                    });

                quote!(#source_expr.backtrace()#backtrace_expr)
            }

            (None, Some(backtrace)) => {
                let backtrace_expr = backtrace_expr(self, backtrace);
                bind(backtrace, &backtrace_expr);

                quote!(Some(#backtrace_expr))
            }

            _ => return None,
        };

        let pattern = if is_rendering_enum_match_arm {
            self.data.matcher(&indices, &bindings)
        } else {
            TokenStream::new()
        };

        Some((pattern, expr))
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
            parse_fields_impl(type_params, state, |attr, field, _| {
                // Unwrapping is safe, cause fields in named struct
                // always have an ident
                let ident = field.ident.as_ref().unwrap();

                match attr {
                    "source" => ident == "source",
                    "backtrace" => {
                        ident == "backtrace"
                            || is_type_ends_with(&field.ty, "Backtrace")
                    }
                    _ => unreachable!(),
                }
            })
        }

        DeriveType::Unnamed => {
            parse_fields_impl(type_params, state, |attr, field, len| match attr {
                "source" => len == 1,
                "backtrace" => is_type_ends_with(&field.ty, "Backtrace"),
                _ => unreachable!(),
            })
        }

        _ => unreachable!(), // TODO
    }
}

fn is_type_ends_with(ty: &syn::Type, tail: &str) -> bool {
    let ty = match ty {
        syn::Type::Path(ty) => ty,
        _ => return false,
    };

    // Unwrapping is safe, cause 'syn::TypePath.path.segments'
    // have to have at least one segment
    let ty = ty.path.segments.last().unwrap();

    match ty.arguments {
        syn::PathArguments::None => (),
        _ => return false,
    };

    ty.ident == tail
}

fn parse_fields_impl<'input, 'state, P>(
    type_params: &HashSet<syn::Ident>,
    state: &'state State<'input>,
    is_valid_default_field_for_attr: P,
) -> Result<ParsedFields<'input, 'state>>
where
    P: Fn(&str, &syn::Field, usize) -> bool,
{
    let MultiFieldData { fields, infos, .. } = state.enabled_fields_data();

    let iter = fields
        .iter()
        .zip(infos.iter().map(|info| &info.info))
        .enumerate()
        .map(|(index, (field, info))| (index, *field, info));

    let mut source = None;
    parse_field_impl(
        &is_valid_default_field_for_attr,
        state.fields.len(),
        iter.clone(),
        "source",
        |info| info.source,
        &mut source,
    )?;

    let mut backtrace = None;
    parse_field_impl(
        &is_valid_default_field_for_attr,
        state.fields.len(),
        iter.clone(),
        "backtrace",
        |info| info.backtrace,
        &mut backtrace,
    )?;

    let mut parsed_fields = ParsedFields::new(state.enabled_fields_data());

    if let Some((index, field, _)) = source {
        parsed_fields.source = Some(index);
        add_bound_if_type_parameter_used_in_type(
            &mut parsed_fields.bounds,
            type_params,
            &field.ty,
        );
    }

    if let Some((index, _, _)) = backtrace {
        parsed_fields.backtrace = Some(index);
    }

    Ok(parsed_fields)
}

fn parse_field_impl<'a, P, V>(
    is_valid_default_field_for_attr: &P,
    len: usize,
    iter: impl Iterator<Item = (usize, &'a syn::Field, &'a MetaInfo)> + Clone,
    attr: &str,
    value: V,
    out: &mut Option<(usize, &'a syn::Field, &'a MetaInfo)>,
) -> Result<()>
where
    P: Fn(&str, &syn::Field, usize) -> bool,
    V: Fn(&MetaInfo) -> Option<bool>,
{
    let explicit_fields = iter.clone().filter(|(_, _, info)| match value(info) {
        Some(true) => true,
        _ => false,
    });

    let inferred_fields = iter.filter(|(_, field, info)| match value(info) {
        None => is_valid_default_field_for_attr(attr, field, len),
        _ => false,
    });

    let field = assert_iter_contains_zero_or_one_item(
        explicit_fields,
        &format!(
            "Multiple `{}` attributes specified. \
             Single attribute per struct/enum variant allowed.",
            attr
        ),
    )?;

    let field = match field {
        field @ Some(_) => field,
        None => assert_iter_contains_zero_or_one_item(
            inferred_fields,
            "Conflicting fields found. Consider specifying some \
             `#[error(...)]` attributes to resolve conflict.",
        )?,
    };

    *out = field;

    Ok(())
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
