#![cfg_attr(not(feature = "default"), allow(dead_code))]
use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{
    parse::{Error, Result},
    parse_str,
    spanned::Spanned,
    Attribute, Data, DeriveInput, Field, Fields, FieldsNamed, FieldsUnnamed, GenericParam,
    Generics, Ident, ImplGenerics, Index, Meta, NestedMeta, Type, TypeGenerics, TypeParamBound,
    WhereClause,
};

#[derive(Clone, Copy)]
pub enum RefType {
    No,
    Ref,
    Mut,
}

impl RefType {
    pub fn from_derive(trait_name: &str) -> (Self, &str) {
        if trait_name.ends_with("RefMut") {
            (RefType::Mut, trait_name.trim_end_matches("RefMut"))
        } else if trait_name.ends_with("Ref") {
            (RefType::Ref, trait_name.trim_end_matches("Ref"))
        } else {
            (RefType::No, trait_name)
        }
    }

    pub fn lifetime(self) -> TokenStream {
        match self {
            RefType::No => quote!(),
            _ => quote!('__deriveMoreLifetime),
        }
    }

    pub fn reference(self) -> TokenStream {
        match self {
            RefType::No => quote!(),
            RefType::Ref => quote!(&),
            RefType::Mut => quote!(&mut),
        }
    }

    pub fn mutability(self) -> TokenStream {
        match self {
            RefType::Mut => quote!(mut),
            _ => quote!(),
        }
    }

    pub fn pattern_ref(self) -> TokenStream {
        match self {
            RefType::Ref => quote!(ref),
            RefType::Mut => quote!(ref mut),
            RefType::No => quote!(),
        }
    }

    pub fn reference_with_lifetime(self) -> TokenStream {
        if !self.is_ref() {
            return quote!();
        }
        let lifetime = self.lifetime();
        let mutability = self.mutability();
        quote!(&#lifetime #mutability)
    }

    pub fn is_ref(self) -> bool {
        match self {
            RefType::No => false,
            _ => true,
        }
    }
    pub fn attr_suffix(self) -> &'static str {
        match self {
            RefType::Ref => "_ref",
            RefType::Mut => "_ref_mut",
            RefType::No => "",
        }
    }
}

pub fn numbered_vars(count: usize, prefix: &str) -> Vec<Ident> {
    (0..count)
        .map(|i| Ident::new(&format!("__{}{}", prefix, i), Span::call_site()))
        .collect()
}

pub fn number_idents(count: usize) -> Vec<Index> {
    (0..count).map(Index::from).collect()
}

pub fn field_idents<'a>(fields: &'a [&'a Field]) -> Vec<&'a Ident> {
    fields
        .iter()
        .map(|f| {
            f.ident
                .as_ref()
                .expect("Tried to get field names of a tuple struct")
        })
        .collect()
}

pub fn get_field_types_iter<'a>(
    fields: &'a [&'a Field],
) -> Box<dyn Iterator<Item = &'a Type> + 'a> {
    Box::new(fields.iter().map(|f| &f.ty))
}

pub fn get_field_types<'a>(fields: &'a [&'a Field]) -> Vec<&'a Type> {
    get_field_types_iter(fields).collect()
}

pub fn add_extra_type_param_bound_op_output<'a>(
    generics: &'a Generics,
    trait_ident: &'a Ident,
) -> Generics {
    let mut generics = generics.clone();
    for type_param in &mut generics.type_params_mut() {
        let type_ident = &type_param.ident;
        let bound: TypeParamBound =
            parse_str(&quote!(::core::ops::#trait_ident<Output=#type_ident>).to_string()).unwrap();
        type_param.bounds.push(bound)
    }

    generics
}

pub fn add_extra_ty_param_bound_op<'a>(generics: &'a Generics, trait_ident: &'a Ident) -> Generics {
    add_extra_ty_param_bound(generics, &quote!(::core::ops::#trait_ident))
}

pub fn add_extra_ty_param_bound<'a>(generics: &'a Generics, bound: &'a TokenStream) -> Generics {
    let mut generics = generics.clone();
    let bound: TypeParamBound = parse_str(&bound.to_string()).unwrap();
    for type_param in &mut generics.type_params_mut() {
        type_param.bounds.push(bound.clone())
    }

    generics
}

pub fn add_extra_ty_param_bound_ref<'a>(
    generics: &'a Generics,
    bound: &'a TokenStream,
    ref_type: RefType,
) -> Generics {
    match ref_type {
        RefType::No => add_extra_ty_param_bound(generics, bound),
        _ => {
            let generics = generics.clone();
            let idents = generics.type_params().map(|x| &x.ident);
            let ref_with_lifetime = ref_type.reference_with_lifetime();
            add_extra_where_clauses(
                &generics,
                quote!(
                    where #(#ref_with_lifetime #idents: #bound),*
                ),
            )
        }
    }
}

pub fn add_extra_generic_param(generics: &Generics, generic_param: TokenStream) -> Generics {
    let generic_param: GenericParam = parse_str(&generic_param.to_string()).unwrap();
    let mut generics = generics.clone();
    generics.params.push(generic_param);

    generics
}

pub fn add_extra_where_clauses(generics: &Generics, type_where_clauses: TokenStream) -> Generics {
    let mut type_where_clauses: WhereClause = parse_str(&type_where_clauses.to_string()).unwrap();
    let mut new_generics = generics.clone();
    if let Some(old_where) = new_generics.where_clause {
        type_where_clauses.predicates.extend(old_where.predicates)
    }
    new_generics.where_clause = Some(type_where_clauses);

    new_generics
}

pub fn add_where_clauses_for_new_ident<'a>(
    generics: &'a Generics,
    fields: &[&'a Field],
    type_ident: &Ident,
    type_where_clauses: TokenStream,
) -> Generics {
    let generic_param = if fields.len() > 1 {
        quote!(#type_ident: ::core::marker::Copy)
    } else {
        quote!(#type_ident)
    };

    let generics = add_extra_where_clauses(generics, type_where_clauses);
    add_extra_generic_param(&generics, generic_param)
}

pub fn unnamed_to_vec(fields: &FieldsUnnamed) -> Vec<&Field> {
    fields.unnamed.iter().collect()
}

pub fn named_to_vec(fields: &FieldsNamed) -> Vec<&Field> {
    fields.named.iter().collect()
}

fn panic_one_field(trait_name: &str, trait_attr: &str) -> ! {
    panic!(format!(
        "derive({}) only works when forwarding to a single field. Try putting #[{}] or #[{}(ignore)] on the fields in the struct",
        trait_name, trait_attr, trait_attr,
    ))
}

pub struct State<'input> {
    pub input: &'input DeriveInput,
    pub trait_name: &'static str,
    pub trait_module: TokenStream,
    pub trait_path: TokenStream,
    pub trait_attr: String,
    pub named: bool,
    pub fields: Vec<&'input Field>,
    pub generics: Generics,
    full_meta_infos: Vec<FullMetaInfo>,
}

impl<'input> State<'input> {
    pub fn new<'arg_input>(
        input: &'arg_input DeriveInput,
        trait_name: &'static str,
        trait_module: TokenStream,
        trait_attr: String,
    ) -> Result<State<'arg_input>> {
        let trait_name = trait_name.trim_end_matches("ToInner");
        let trait_ident = Ident::new(trait_name, Span::call_site());
        let trait_path = quote!(#trait_module::#trait_ident);
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
            _ => {
                panic_one_field(&trait_name, &trait_attr);
            }
        };
        let struct_meta_info = get_meta_info(&trait_attr, &input.attrs)?;
        let meta_infos: Result<Vec<_>> = fields
            .iter()
            .map(|f| &f.attrs)
            .map(|attrs| get_meta_info(&trait_attr, attrs))
            .collect();
        let meta_infos = meta_infos?;
        let first_match = meta_infos.iter().filter_map(|info| info.enabled).next();
        let defaults = struct_meta_info.to_full(FullMetaInfo {
            enabled: first_match.map_or(true, |enabled| !enabled),
            forward: false,
        });
        let full_meta_infos: Vec<_> = meta_infos
            .iter()
            .map(|info| info.to_full(defaults))
            .collect();
        let generics = add_extra_ty_param_bound(&input.generics, &trait_path);

        Ok(State {
            input,
            trait_name,
            trait_module,
            trait_path,
            trait_attr,
            // input,
            fields,
            named,
            generics,
            full_meta_infos,
        })
    }
    pub fn add_trait_path_type_param(&mut self, params: TokenStream) {
        let trait_path = &self.trait_path;
        self.trait_path = quote!(#trait_path<#params>)
    }

    pub fn assert_single_enabled_field<'state>(&'state self) -> (SingleFieldData<'input, 'state>) {
        let enabled_fields = self.enabled_fields();
        if enabled_fields.len() != 1 {
            panic_one_field(self.trait_name, &self.trait_attr);
        };
        let mut field_idents = self.enabled_fields_idents();
        let field_type = &enabled_fields[0].ty;
        let trait_path = &self.trait_path;
        let (impl_generics, ty_generics, where_clause) = self.generics.split_for_impl();
        let field_ident = field_idents.remove(0);
        let field_ident_ref = &field_ident;
        SingleFieldData {
            input_type: &self.input.ident,
            field: enabled_fields[0],
            field_type,
            member: quote!(self.#field_ident_ref),
            info: self.enabled_fields_infos()[0],
            field_ident,
            trait_path,
            casted_trait: quote!(<#field_type as #trait_path>),
            impl_generics,
            ty_generics,
            where_clause,
        }
    }

    pub fn enabled_fields_data<'state>(&'state self) -> (MultiFieldData<'input, 'state>) {
        let fields = self.enabled_fields();
        let field_idents = self.enabled_fields_idents();
        let field_types: Vec<_> = fields.iter().map(|f| &f.ty).collect();
        let members: Vec<_> = field_idents
            .iter()
            .map(|ident| quote!(self.#ident))
            .collect();
        let trait_path = &self.trait_path;
        let casted_traits: Vec<_> = field_types
            .iter()
            .map(|field_type| quote!(<#field_type as #trait_path>))
            .collect();
        let (impl_generics, ty_generics, where_clause) = self.generics.split_for_impl();
        MultiFieldData {
            input_type: &self.input.ident,
            fields,
            field_types,
            members,
            infos: self.enabled_fields_infos(),
            field_idents,
            trait_path,
            casted_traits,
            impl_generics,
            ty_generics,
            where_clause,
        }
    }

    fn enabled_fields(&self) -> Vec<&'input Field> {
        self.fields
            .iter()
            .zip(self.full_meta_infos.iter().map(|info| info.enabled))
            .filter(|(_, ig)| *ig)
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
                    ) as Box<dyn ToTokens>
                })
                .collect()
        } else {
            let count = self.fields.len();
            (0..count)
                .map(|i| Box::new(Index::from(i)) as Box<dyn ToTokens>)
                .collect()
        }
    }

    fn enabled_fields_idents(&self) -> Vec<Box<dyn ToTokens>> {
        self.field_idents()
            .into_iter()
            .zip(self.full_meta_infos.iter().map(|info| info.enabled))
            .filter(|(_, ig)| *ig)
            .map(|(f, _)| f)
            .collect()
    }
    fn enabled_fields_infos(&self) -> Vec<FullMetaInfo> {
        self.full_meta_infos
            .iter()
            .filter(|info| info.enabled)
            .copied()
            .collect()
    }
}

pub struct SingleFieldData<'input, 'state> {
    pub input_type: &'input Ident,
    pub field: &'input Field,
    pub field_type: &'input Type,
    pub field_ident: Box<dyn ToTokens>,
    pub member: TokenStream,
    pub info: FullMetaInfo,
    pub trait_path: &'state TokenStream,
    pub casted_trait: TokenStream,
    pub impl_generics: ImplGenerics<'state>,
    pub ty_generics: TypeGenerics<'state>,
    pub where_clause: Option<&'state WhereClause>,
}

pub struct MultiFieldData<'input, 'state> {
    pub input_type: &'input Ident,
    pub fields: Vec<&'input Field>,
    pub field_types: Vec<&'input Type>,
    pub field_idents: Vec<Box<dyn ToTokens>>,
    pub members: Vec<TokenStream>,
    pub infos: Vec<FullMetaInfo>,
    pub trait_path: &'state TokenStream,
    pub casted_traits: Vec<TokenStream>,
    pub impl_generics: ImplGenerics<'state>,
    pub ty_generics: TypeGenerics<'state>,
    pub where_clause: Option<&'state WhereClause>,
}

fn get_meta_info(trait_attr: &str, attrs: &[Attribute]) -> Result<MetaInfo> {
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
        return Ok(MetaInfo {
            enabled: None,
            forward: None,
        });
    };
    if let Some(meta2) = it.next() {
        return Err(Error::new(meta2.span(), "Too many formats given"));
    }
    let list = match meta.clone() {
        Meta::Path(_) => {
            return Ok(MetaInfo {
                enabled: Some(true),
                forward: None,
            })
        }
        Meta::List(list) => list,
        _ => {
            return Err(Error::new(meta.span(), "Attribute format not supported1"));
        }
    };
    let mut info = MetaInfo {
        enabled: Some(true),
        forward: None,
    };
    for element in list.nested.into_iter() {
        let nested_meta = if let NestedMeta::Meta(meta) = element {
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
        if ident == "ignore" {
            info.enabled = Some(false);
        } else if ident == "forward" {
            info.forward = Some(true);
        } else {
            return Err(Error::new(meta.span(), "Attribute format not supported6"));
        };
    }
    Ok(info)
}

#[derive(Copy, Clone, Debug)]
pub struct FullMetaInfo {
    pub enabled: bool,
    pub forward: bool,
}

#[derive(Copy, Clone)]
struct MetaInfo {
    enabled: Option<bool>,
    forward: Option<bool>,
}

impl MetaInfo {
    fn to_full(self, defaults: FullMetaInfo) -> FullMetaInfo {
        FullMetaInfo {
            enabled: self.enabled.unwrap_or(defaults.enabled),
            forward: self.forward.unwrap_or(defaults.forward),
        }
    }
}
