use syn::{Ident, Ty, Field};

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
