use crate::utils::{
    add_extra_generic_param, add_extra_ty_param_bound_ref, named_to_vec, unnamed_to_vec, RefType,
};
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{Data, DeriveInput, Field, Fields, Ident};

/// Provides the hook to expand `#[derive(Index)]` into an implementation of `From`
pub fn expand(input: &DeriveInput, trait_name: &str) -> TokenStream {
    let (ref_type, trait_name) = RefType::from_derive(trait_name);
    let trait_name = trait_name.trim_end_matches("Ref");
    let trait_ident = Ident::new(trait_name, Span::call_site());
    let trait_path = &quote!(::core::iter::#trait_ident);
    let input_type = &input.ident;
    let field_vec: Vec<&Field>;
    let member = match input.data {
        Data::Struct(ref data_struct) => match data_struct.fields {
            Fields::Unnamed(ref fields) => {
                field_vec = unnamed_to_vec(fields);
                tuple_from_str(trait_name, &field_vec)
            }
            Fields::Named(ref fields) => {
                field_vec = named_to_vec(fields);
                struct_from_str(trait_name, &field_vec)
            }
            Fields::Unit => panic_one_field(trait_name),
        },
        _ => panic_one_field(trait_name),
    };
    let field_type = &field_vec[0].ty;

    let reference = ref_type.reference();
    let lifetime = ref_type.lifetime();
    let reference_with_lifetime = ref_type.reference_with_lifetime();

    let generics_impl;
    let generics = add_extra_ty_param_bound_ref(&input.generics, trait_path, ref_type);
    let (_, ty_generics, where_clause) = generics.split_for_impl();
    let (impl_generics, _, _) = if ref_type.is_ref() {
        generics_impl = add_extra_generic_param(&generics, lifetime.clone());
        generics_impl.split_for_impl()
    } else {
        generics.split_for_impl()
    };
    // let generics = add_extra_ty_param_bound(&input.generics, trait_path);
    let casted_trait = &quote!(<#reference_with_lifetime #field_type as #trait_path>);
    quote! {
        impl#impl_generics #trait_path for #reference_with_lifetime #input_type#ty_generics #where_clause
        {
            type Item = #casted_trait::Item;
            type IntoIter = #casted_trait::IntoIter;
            #[inline]
            fn into_iter(self) -> Self::IntoIter {
                #casted_trait::into_iter(#reference #member)
            }
        }
    }
}

fn panic_one_field(trait_name: &str) -> ! {
    panic!(format!(
        "Only structs with one field can derive({})",
        trait_name
    ))
}

fn tuple_from_str<'a>(trait_name: &str, fields: &[&'a Field]) -> (TokenStream) {
    if fields.len() != 1 {
        panic_one_field(trait_name)
    };
    quote!(self.0)
}

fn struct_from_str<'a>(trait_name: &str, fields: &[&'a Field]) -> TokenStream {
    if fields.len() != 1 {
        panic_one_field(trait_name)
    };
    let field = &fields[0];
    let field_ident = &field.ident;
    quote!(self.#field_ident)
}
