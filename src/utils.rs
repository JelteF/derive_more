use syn::{Ident, Ty, Field, Generics};
use syn::parse_ty_param_bound;

pub fn numbered_vars(count: usize, prefix: &str) -> Vec<Ident> {
    (0..count).map(|i| Ident::from(format!("__{}{}", prefix, i))).collect()
}

pub fn number_idents(count: usize) -> Vec<Ident> {
    (0..count).map(|i| Ident::from(i.to_string())).collect()
}

pub fn get_field_types_iter<'a>(fields: &'a Vec<Field>) -> Box<Iterator<Item = &'a Ty> + 'a> {
    Box::new(fields.iter().map(|f| &f.ty))
}

pub fn get_field_types<'a>(fields: &'a Vec<Field>) -> Vec<&'a Ty> {
    get_field_types_iter(fields).collect()
}

pub fn add_extra_ty_param_bound_rhs<'a>(generics: &'a Generics,
                                        trait_ident: &'a Ident,
                                        rhs_ident: &'a Ident)
                                        -> Generics {
    let mut generics = generics.clone();
    for ref mut ty_param in &mut generics.ty_params {
        let ty_ident = &ty_param.ident;
        ty_param.bounds
            .push(parse_ty_param_bound(&quote!(::std::ops::#trait_ident<#rhs_ident, Output=#ty_ident>)
                    .to_string())
                .unwrap());
    }

    generics
}


pub fn add_extra_ty_param_bound<'a>(generics: &'a Generics, trait_ident: &'a Ident) -> Generics {
    let mut generics = generics.clone();
    for ref mut ty_param in &mut generics.ty_params {
        let ty_ident = &ty_param.ident;
        ty_param.bounds
            .push(parse_ty_param_bound(&quote!(::std::ops::#trait_ident<Output=#ty_ident>)
                    .to_string())
                .unwrap());
    }

    generics
}

pub fn add_extra_ty_param_bound_simple<'a>(generics: &'a Generics,
                                           trait_ident: &'a Ident)
                                           -> Generics {
    let mut generics = generics.clone();
    for ref mut ty_param in &mut generics.ty_params {
        ty_param.bounds
            .push(parse_ty_param_bound(&quote!(::std::ops::#trait_ident).to_string()).unwrap());
    }

    generics
}
