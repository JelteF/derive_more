use std::collections::{
    hash_map::{HashMap, Entry as HashMapEntry},
    HashSet,
};

use proc_macro2::TokenStream;
use syn::{
    Error,
    Result,
    punctuated::Punctuated,
    spanned::Spanned as _,
};
use quote::quote;

use crate::utils;


pub fn expand(
    input: &syn::DeriveInput,
    _trait_name: &'static str,
) -> Result<TokenStream> {
    let syn::DeriveInput {
        ident,
        generics,
        data,
        ..
    } = input;

    let type_params: HashSet<_> = generics.params.iter()
        .filter_map(|generic| match generic {
            syn::GenericParam::Type(ty) => Some(ty.ident.clone()),
            _ => None,
        })
        .collect();

    let (bounds, body) = match data {
        syn::Data::Struct(data) => render_struct(&type_params, data)?,
        syn::Data::Enum(data) => render_enum(&type_params, data)?,
        syn::Data::Union(data) => return Err(Error::new(
            data.union_token.span(),
            "Only `struct` and `enum` types supported by this macro",
        )),
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
    data: &syn::DataStruct
) -> Result<(HashSet<syn::Type>, TokenStream)> {
    let parsed_fields = parse_fields(&type_params, &data.fields)?;

    let render = parsed_fields.render_as_struct();
    let bounds = parsed_fields.bounds;

    Ok((bounds, render))
}

fn render_enum(
    type_params: &HashSet<syn::Ident>,
    data: &syn::DataEnum
) -> Result<(HashSet<syn::Type>, TokenStream)> {
    let mut bounds = HashSet::new();
    let mut match_arms = Vec::new();
    let mut render_default_wildcard = false;

    for variant in &data.variants {
        let parsed_fields = parse_fields(&type_params, &variant.fields)?;

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


#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
enum ValidAttrs {
    Source,
    // Backtrace,
}

impl ValidAttrs {
    fn str(&self) -> &'static str {
        match self {
            Self::Source => "source",
            // Self::Backtrace => "backtrace",
        }
    }
}

const VALID_ATTRS: &[ValidAttrs] = &[
    ValidAttrs::Source,
    // ValidAttrs::Backtrace,
];


#[derive(Debug)]
struct ParsedFields<'a> {
    ty: FieldsType,
    len: usize,
    fields: HashMap<ValidAttrs, ParsedField<'a>>,
    bounds: HashSet<syn::Type>,
}

#[derive(Copy, Clone, Debug)]
enum FieldsType {
    Named,
    Unnamed,
    Unit,
}

#[derive(Debug)]
struct ParsedField<'a> {
    field: &'a syn::Field,
    index: usize,
    attrs: HashMap<ValidAttrs, bool>,
}


impl<'a> ParsedFields<'a> {
    fn new(ty: FieldsType, len: usize) -> Self {
        Self {
            ty,
            len,
            fields: HashMap::new(),
            bounds: HashSet::new(),
        }
    }

    fn named(len: usize) -> Self {
        Self::new(FieldsType::Named, len)
    }

    fn unnamed(len: usize) -> Self {
        Self::new(FieldsType::Unnamed, len)
    }

    fn unit() -> Self {
        Self::new(FieldsType::Unit, 0)
    }
}

impl<'a> ParsedField<'a> {
    fn new(field: &'a syn::Field, index: usize, attrs: HashMap<ValidAttrs, bool>) -> Self {
        Self {
            field,
            index,
            attrs,
        }
    }

    fn is_attr_explicitly_set(&self, attr: ValidAttrs) -> bool {
        match self.attrs.get(&attr) {
            Some(true) => true,
            _ => false,
        }
    }

    //    fn is_attr_explicitly_unset(&self, attr: ValidAttrs) -> bool {
    //        match self.attrs.get(&attr) {
    //            Some(false) => true,
    //            _ => false,
    //        }
    //    }
}

impl<'a> ParsedFields<'a> {
    fn render_as_struct(&self) -> TokenStream {
        match self.fields.get(&ValidAttrs::Source) {
            Some(source) => {
                let ident = source.render_as_struct_field_ident();
                render_some(quote!(&self.#ident))
            }
            None => quote!(None),
        }
    }

    fn render_as_enum_variant_match_arm_tail(&self) -> Option<TokenStream> {
        self.fields
            .get(&ValidAttrs::Source)
            .map(|source| match self.ty {
                FieldsType::Named => source.render_as_enum_variant_struct_match_arm_tail(self.len),
                FieldsType::Unnamed => source.render_as_enum_variant_tuple_match_arm_tail(self.len),
                FieldsType::Unit => unreachable!(),
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
    fields: &'a syn::Fields,
) -> Result<ParsedFields<'a>> {
    match fields {
        syn::Fields::Named(fields) => {
            let fields = &fields.named;
            parse_fields_impl(
                ParsedFields::named(fields.len()),
                type_params,
                fields,
                |attr, field, _| match attr {
                    ValidAttrs::Source => {
                        let ident = field.ident.as_ref()
                            .expect("Internal error in macro implementation"); // TODO

                        ident == "source"
                    }
                },
            )
        }

        syn::Fields::Unnamed(fields) => {
            let fields = &fields.unnamed;
            parse_fields_impl(
                ParsedFields::unnamed(fields.len()),
                type_params,
                fields,
                |attr, _, len| match attr {
                    ValidAttrs::Source => {
                        len == 1
                    }
                },
            )
        }

        syn::Fields::Unit => Ok(ParsedFields::unit()),
    }
}

fn parse_fields_impl<'a, P>(
    mut parsed_fields: ParsedFields<'a>,
    type_params: &HashSet<syn::Ident>,
    fields: &'a Punctuated<syn::Field, syn::Token![,]>,
    is_valid_default_field_for_attr: P,
) -> Result<ParsedFields<'a>>
    where P: Fn(ValidAttrs, &syn::Field, usize) -> bool,
{
    for (index, field) in fields.iter().enumerate() {
        let attrs = parse_attrs(&field.attrs)?;

        for attr in VALID_ATTRS {
            match attrs.get(&attr) {
                Some(true) => process_explicitly_set_attr(
                    &mut parsed_fields,
                    type_params,
                    field,
                    index,
                    &attrs,
                    *attr,
                )?,

                None => process_if_valid_default_field_for_attr(
                    &mut parsed_fields,
                    type_params,
                    field,
                    index,
                    &attrs,
                    *attr,
                    &is_valid_default_field_for_attr,
                )?,

                _ => (),
            }
        }
    }

    Ok(parsed_fields)
}

fn process_explicitly_set_attr<'a>(
    parsed_fields: &mut ParsedFields<'a>,
    type_params: &HashSet<syn::Ident>,
    field: &'a syn::Field,
    index: usize,
    attrs: &HashMap<ValidAttrs, bool>,
    attr: ValidAttrs,
) -> Result<()> {
    let prev_parsed_field = parsed_fields.fields.insert(
        attr,
        ParsedField::new(field, index, attrs.clone())
    );

    match prev_parsed_field {
        Some(prev_parsed_field) if prev_parsed_field.is_attr_explicitly_set(attr) => {
            return Err(Error::new(
                prev_parsed_field.field.span(),
                format!(
                    "Multiple `{}` attributes specified. Single attribute per struct/enum variant allowed.",
                    attr.str()
                ),
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
    attrs: &HashMap<ValidAttrs, bool>,
    attr: ValidAttrs,
    is_valid_default_field_for_attr: &P,
) -> Result<()>
    where P: Fn(ValidAttrs, &syn::Field, usize) -> bool,
{
    if !is_valid_default_field_for_attr(attr, field, parsed_fields.len) {
        return Ok(());
    }

    match parsed_fields.fields.entry(attr) {
        HashMapEntry::Vacant(entry) => {
            entry.insert(ParsedField::new(field, index, attrs.clone()));
            add_bound_if_type_parameter_used_in_type(&mut parsed_fields.bounds, type_params, &field.ty);
        }

        HashMapEntry::Occupied(entry) => {
            let parsed_field = entry.get();
            if !parsed_field.is_attr_explicitly_set(attr) {
                return Err(Error::new(
                    parsed_field.field.span(),
                    "Conflicting fields found. Consider specifying some \
                     `#[error(...)]` attributes to resolve conflict.",
                ));
            }
        }
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


const INVALID_FORMAT_ERROR: &str =
    "Invalid format. Valid formats are `#[error(source)]` and `#[error(not(source))]`.";

fn parse_attrs(attrs: &[syn::Attribute]) -> Result<HashMap<ValidAttrs, bool>> {
    let mut iterator = attrs.iter().filter_map(|attr| {
        if attr.path.is_ident("error") {
            Some(attr.parse_meta())
        } else {
            None
        }
    });

    let meta = iterator.next();

    if meta.is_none() {
        return Ok(HashMap::new());
    }

    let meta = meta.expect("Internal error in macro implementation")?;

    {
        let meta = iterator.next();

        if meta.is_some() {
            return Err(Error::new(
                meta.unwrap()?.span(),
                "Too many `#[error(...)]` attributes specified. Single attribute per field allowed."
            ));
        }
    }

    let meta = match meta {
        syn::Meta::List(meta) => meta,
        _ => return Err(Error::new(
            meta.span(),
            INVALID_FORMAT_ERROR
        )),
    };

    let mut parsed_attrs = HashMap::new();
    parse_punctuated_nested_meta(&mut parsed_attrs, &meta.nested, true)?;
    Ok(parsed_attrs)
}

fn parse_punctuated_nested_meta(
    parsed_attrs: &mut HashMap<ValidAttrs, bool>,
    punctuated_nested_meta: &Punctuated<syn::NestedMeta, syn::Token![,]>,
    value: bool,
) -> Result<()> {
    for meta in punctuated_nested_meta {
        let meta = match meta {
            syn::NestedMeta::Meta(meta) => meta,
            _ => return Err(Error::new(meta.span(), INVALID_FORMAT_ERROR)),
        };

        match meta {
            syn::Meta::List(meta) if meta.path.is_ident("not") => {
                if value {
                    parse_punctuated_nested_meta(parsed_attrs, &meta.nested, false)?;
                } else {
                    return Err(Error::new(
                        meta.span(),
                        "`not` attributes can not be nested",
                    ));
                }
            }

            syn::Meta::Path(path) => parse_path(parsed_attrs, &path, value)?,

            _ => return Err(Error::new(meta.span(), INVALID_FORMAT_ERROR)),
        };
    }

    Ok(())
}

fn parse_path(
    parsed_attrs: &mut HashMap<ValidAttrs, bool>,
    path: &syn::Path,
    value: bool,
) -> Result<()> {
    VALID_ATTRS.iter()
        .find(|attr| {
            path.is_ident(attr.str())
        })
        .ok_or_else(|| {
            Error::new(
                path.span(),
                "Invalid attribute. The only valid attribute is `source`.",
            )
        })
        .and_then(|attr| match parsed_attrs.entry(*attr) {
            HashMapEntry::Vacant(entry) => {
                entry.insert(value);
                Ok(())
            }

            _ => {
                Err(Error::new(
                    path.span(),
                    format!(
                        "Too many `{}` attributes specified. Single attribute per field allowed.",
                        attr.str()
                    ),
                ))
            }
        })
}
