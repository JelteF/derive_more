use crate::utils::{
    add_extra_ty_param_bound, add_extra_where_clauses, field_idents, get_field_types,
    named_to_vec, unnamed_to_vec,
};
use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{Data, DeriveInput, Field, Fields, Ident};

pub fn expand(input: &DeriveInput, trait_name: &str) -> TokenStream {
    let trait_ident = Ident::new(trait_name, Span::call_site());
    let method_name = trait_name.to_string().to_lowercase();
    let method_ident = Ident::new(&(method_name.to_string()), Span::call_site());
    let input_type = &input.ident;
    let trait_path = quote!(::core::iter::#trait_ident);
    let op_trait_name = if trait_name == "Sum" { "Add" } else { "Mul" };
    let op_trait_ident = Ident::new(op_trait_name, Span::call_site());
    let op_path = quote!(::core::ops::#op_trait_ident);
    let op_method_ident = Ident::new(
        &(op_trait_name.to_string().to_lowercase()),
        Span::call_site(),
    );
    let type_params: Vec<_> = input
        .generics
        .type_params()
        .map(|t| t.ident.clone())
        .collect();
    let generics = if type_params.is_empty() {
        input.generics.clone()
    } else {
        let generic_type = quote!(<#(#type_params),*>);
        let generics = add_extra_ty_param_bound(&input.generics, &trait_path);
        let operator_where_clause = quote! {
            where #input_type#generic_type: #op_path<Output=#input_type#generic_type>
        };
        add_extra_where_clauses(&generics, operator_where_clause)
    };
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let identity = match input.data {
        Data::Struct(ref data_struct) => match data_struct.fields {
            Fields::Unnamed(ref fields) => {
                tuple_identity(input_type, &unnamed_to_vec(fields), &method_ident)
            }
            Fields::Named(ref fields) => {
                struct_identity(input_type, &named_to_vec(fields), &method_ident)
            }
            _ => panic!(format!("Unit structs cannot use derive({})", trait_name)),
        },

        _ => panic!(format!("Only structs can use derive({})", trait_name)),
    };

    quote!(
        impl#impl_generics #trait_path for #input_type#ty_generics #where_clause {
            #[inline]
            fn #method_ident<I: ::core::iter::Iterator<Item = Self>>(iter: I) -> Self {
                iter.fold(#identity, #op_path::#op_method_ident)
            }
        }
    )
}

fn tuple_identity<T: ToTokens>(
    input_type: &T,
    fields: &[&Field],
    method_ident: &Ident,
) -> TokenStream {
    let types = &get_field_types(fields);
    quote!(#input_type(#(::core::iter::empty::<#types>().#method_ident()),*))
}

fn struct_identity(
    input_type: &Ident,
    fields: &[&Field],
    method_ident: &Ident,
) -> TokenStream {
    let field_names = field_idents(fields);
    let types = &get_field_types(fields);

    quote!(#input_type{#(#field_names: ::core::iter::empty::<#types>().#method_ident()),*})
}
