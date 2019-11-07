use std::collections::HashSet;

use proc_macro2::TokenStream;
use syn::{
    Error,
    Result,
    spanned::Spanned as _,
};
use quote::quote;

use crate::utils::{self, DeriveType, MetaInfo, State, FullMetaInfo};


pub fn expand(
    input: &syn::DeriveInput,
    trait_name: &'static str,
) -> Result<TokenStream> {
    let syn::DeriveInput {
        ident,
        generics,
        ..
    } = input;

    let state = State::new(
        input,
        trait_name,
        quote!(::std::error),
        trait_name.to_lowercase(),
    )?;

    let type_params: HashSet<_> = generics.params.iter()
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
            }
        );
    }

    if !bounds.is_empty() {
        let bounds = bounds.iter();
        generics = utils::add_extra_where_clauses(
            &generics,
            quote! {
                where
                    #(#bounds: ::std::fmt::Debug + ::std::fmt::Display + ::std::error::Error + 'static),*
            }
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
    state: &State
) -> Result<(HashSet<syn::Type>, TokenStream)> {
    let parsed_fields = parse_fields(&type_params, &state)?;

    let render = parsed_fields.render_as_struct();
    let bounds = parsed_fields.bounds;

    Ok((bounds, render))
}

fn render_enum(
    type_params: &HashSet<syn::Ident>,
    state: &State
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
            variant,
            default_info,
        )?;

        eprintln!("{} {:?}", state.variant.unwrap().ident, state.derive_type);

        let parsed_fields = parse_fields(&type_params, &state)?;

        match parsed_fields.render_as_enum_variant_match_arm_tail() {
            Some(match_arm_part_render) => {
                let ident = &variant.ident;
                match_arms.push(quote!(Self::#ident#match_arm_part_render));
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


#[derive(Debug)]
struct ParsedFields<'a> {
    derive_type: DeriveType,
    len: usize,
    source: Option<ParsedField<'a>>,
    bounds: HashSet<syn::Type>,
}

#[derive(Debug)]
struct ParsedField<'a> {
    field: &'a syn::Field,
    index: usize,
    info: MetaInfo,
}


impl<'a> ParsedFields<'a> {
    fn new(derive_type: DeriveType, len: usize) -> Self {
        Self {
            derive_type,
            len,
            source: None,
            bounds: HashSet::new(),
        }
    }

    fn named(len: usize) -> Self {
        Self::new(DeriveType::Named, len)
    }

    fn unnamed(len: usize) -> Self {
        Self::new(DeriveType::Unnamed, len)
    }
}

impl<'a> ParsedField<'a> {
    fn new(field: &'a syn::Field, index: usize, info: MetaInfo) -> Self {
        Self {
            field,
            index,
            info,
        }
    }

    fn is_source_explicitly_set(&self) -> bool {
        match self.info.source {
            Some(true) => true,
            _ => false,
        }
    }
}


impl<'a> ParsedFields<'a> {
    fn render_as_struct(&self) -> TokenStream {
        match &self.source {
            Some(source) => {
                let ident = source.render_as_struct_field_ident();
                render_some(quote!(&self.#ident))
            }
            None => quote!(None),
        }
    }

    fn render_as_enum_variant_match_arm_tail(&self) -> Option<TokenStream> {
        self.source
            .as_ref()
            .map(|source| match self.derive_type {
                DeriveType::Named => source.render_as_enum_variant_struct_match_arm_tail(self.len),
                DeriveType::Unnamed => source.render_as_enum_variant_tuple_match_arm_tail(self.len),
                _ => unreachable!(), // TODO
            })
    }
}

impl<'a> ParsedField<'a> {
    fn render_as_struct_field_ident(&self) -> TokenStream {
        match &self.field.ident {
            Some(ident) => quote!(#ident),
            None => {
                let index = syn::Index::from(self.index);
                quote!(#index)
            }
        }
    }

    fn render_as_enum_variant_struct_match_arm_tail(&self, len: usize) -> TokenStream {
        let ident = self.field.ident.as_ref()
            .expect("Internal error in macro implementation"); // TODO

        let mut bindings = quote!(#ident);

        if len > 1 {
            bindings = quote!(#bindings, ..);
        }

        let some = render_some(ident);

        quote! {
            {#bindings} => #some
        }
    }

    fn render_as_enum_variant_tuple_match_arm_tail(&self, len: usize) -> TokenStream {
        assert_ne!(len, 0, "Internal error in macro implementation"); // TODO

        let bindings = (0..len).map(|index| {
            if index == self.index {
                quote!(source)
            } else {
                quote!(_)
            }
        });

        let some = render_some(quote!(source));

        quote! {
            (#(#bindings),*) => #some
        }
    }
}

fn render_some<T>(expr: T) -> TokenStream
    where T: quote::ToTokens
{
    quote!(Some(#expr as &(dyn ::std::error::Error + 'static)))
}


fn parse_fields<'a>(
    type_params: &HashSet<syn::Ident>,
    state: &'a State
) -> Result<ParsedFields<'a>> {
    match state.derive_type {
        DeriveType::Named => {
            parse_fields_impl(
                ParsedFields::named(state.fields.len()),
                type_params,
                state,
                |field, _| {
                    let ident = field.ident.as_ref()
                        .expect("Internal error in macro implementation"); // TODO

                    ident == "source"
                },
            )
        }

        DeriveType::Unnamed => {
            parse_fields_impl(
                ParsedFields::unnamed(state.fields.len()),
                type_params,
                state,
                |_, len| len == 1,
            )
        }

        _ => unreachable!(), // TODO
    }
}

fn parse_fields_impl<'a, P>(
    mut parsed_fields: ParsedFields<'a>,
    type_params: &HashSet<syn::Ident>,
    state: &'a State,
    is_valid_default_field_for_attr: P,
) -> Result<ParsedFields<'a>>
    where P: Fn(&syn::Field, usize) -> bool,
{
    let fields = state.enabled_fields_data();

    for (index, (field, info)) in fields.fields.into_iter().zip(fields.infos.into_iter().map(|info| info.info)).enumerate() {
        match info.source {
            Some(true) => process_explicitly_set_attr(
                &mut parsed_fields,
                type_params,
                field,
                index,
                info,
            )?,

            None => process_if_valid_default_field_for_attr(
                &mut parsed_fields,
                type_params,
                field,
                index,
                info,
                &is_valid_default_field_for_attr,
            )?,

            _ => (),
        }
    }

    Ok(parsed_fields)
}

fn process_explicitly_set_attr<'a>(
    parsed_fields: &mut ParsedFields<'a>,
    type_params: &HashSet<syn::Ident>,
    field: &'a syn::Field,
    index: usize,
    info: MetaInfo,
) -> Result<()> {
    let prev_parsed_field = parsed_fields.source.replace(
        ParsedField::new(field, index, info)
    );

    match &prev_parsed_field {
        Some(prev_parsed_field) if prev_parsed_field.is_source_explicitly_set() => {
            return Err(Error::new(
                prev_parsed_field.field.span(),
                "Multiple `source` attributes specified. Single attribute per struct/enum variant allowed."
            ));
        }
        _ => (),
    };

    add_bound_if_type_parameter_used_in_type(&mut parsed_fields.bounds, type_params, &field.ty);

    Ok(())
}

fn process_if_valid_default_field_for_attr<'a, P>(
    parsed_fields: &mut ParsedFields<'a>,
    type_params: &HashSet<syn::Ident>,
    field: &'a syn::Field,
    index: usize,
    info: MetaInfo,
    is_valid_default_field_for_attr: &P,
) -> Result<()>
    where P: Fn(&syn::Field, usize) -> bool,
{
    if !is_valid_default_field_for_attr(field, parsed_fields.len) {
        return Ok(());
    }

    if let Some(parsed_field) = &mut parsed_fields.source {
        if !parsed_field.is_source_explicitly_set() {
            return Err(Error::new(
                    parsed_field.field.span(),
                "Conflicting fields found. Consider specifying some \
                     `#[error(...)]` attributes to resolve conflict.",
            ));
        }
    } else {
        eprintln!("PING");
        parsed_fields.source = Some(ParsedField::new(field, index, info));
        add_bound_if_type_parameter_used_in_type(&mut parsed_fields.bounds, type_params, &field.ty);
    }

    Ok(())
}

fn add_bound_if_type_parameter_used_in_type(
    bounds: &mut HashSet<syn::Type>,
    type_params: &HashSet<syn::Ident>,
    ty: &syn::Type,
) {
    match utils::get_if_type_parameter_used_in_type(type_params, ty) {
        Some(ty) => {
            bounds.insert(ty);
        }
        _ => (),
    }
}
