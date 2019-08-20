#![allow(dead_code)]

use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{
    parse_str, Field, FieldsNamed, FieldsUnnamed, GenericParam, Generics, Ident, Index, Type,
    TypeParamBound, WhereClause,
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
