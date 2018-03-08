use syn::{parse_str, Field, Generics, Ident, Type, TypeParamBound, FieldsUnnamed, FieldsNamed, Index};

pub fn numbered_vars(count: usize, prefix: &str) -> Vec<Ident> {
    (0..count)
        .map(|i| Ident::from(format!("__{}{}", prefix, i)))
        .collect()
}

pub fn number_idents(count: usize) -> Vec<Index> {
    (0..count).map(|i| Index::from(i)).collect()
}

pub fn field_idents<'a>(fields: &'a Vec<&'a Field>) -> Vec<&'a Ident> {
    fields
        .iter()
        .map(|f| {
            f.ident
                .as_ref()
                .expect("Tried to get field names of a tuple struct")
        })
        .collect()
}

pub fn get_field_types_iter<'a>(fields: &'a Vec<&'a Field>) -> Box<Iterator<Item = &'a Type> + 'a> {
    Box::new(fields.iter().map(|f| &f.ty))
}

pub fn get_field_types<'a>(fields: &'a Vec<&'a Field>) -> Vec<&'a Type> {
    get_field_types_iter(fields).collect()
}

pub fn add_extra_type_param_bound<'a>(generics: &'a Generics, trait_ident: &'a Ident) -> Generics {
    let mut generics = generics.clone();
    for ref mut type_param in &mut generics.type_params_mut() {
        let type_ident = &type_param.ident;
        let bound: TypeParamBound = parse_str(&quote!(::std::ops::#trait_ident<Output=#type_ident>).to_string()).unwrap();
        type_param.bounds.push(bound)
    }

    generics
}

pub fn add_extra_ty_param_bound_simple<'a>(
    generics: &'a Generics,
    trait_ident: &'a Ident,
) -> Generics {
    let mut generics = generics.clone();
    let bound: TypeParamBound = parse_str(&quote!(::std::ops::#trait_ident).to_string()).unwrap();
    for ref mut type_param in &mut generics.type_params_mut() {
        type_param.bounds.push(bound.clone())
    }


    generics
}

pub fn unnamed_to_vec<'a>(fields: &'a FieldsUnnamed) -> Vec<&'a Field>{
    fields.unnamed.iter().collect()
}

pub fn named_to_vec<'a>(fields: &'a FieldsNamed) -> Vec<&'a Field>{
    fields.named.iter().collect()
}
