use std::{collections::HashSet, fmt::Display};

use proc_macro2::{Ident, Span, TokenStream};
use syn::{
    parse::{Error, Result},
    punctuated::Pair,
    spanned::Spanned,
    Attribute, Data, DeriveInput, Fields, Lit, Meta, MetaNameValue, NestedMeta, Type,
};
use utils::{add_extra_where_clauses, get_import_root};

/// Provides the hook to expand `#[derive(Display)]` into an implementation of `From`
pub fn expand(input: &DeriveInput, trait_name: &str) -> Result<TokenStream> {
    let import_root = get_import_root();
    let trait_ident = Ident::new(trait_name, Span::call_site());
    let trait_path = &quote!(#import_root::fmt::#trait_ident);
    let trait_attr = match trait_name {
        "Display" => "display",
        "Binary" => "binary",
        "Octal" => "octal",
        "LowerHex" => "lower_hex",
        "UpperHex" => "upper_hex",
        "LowerExp" => "lower_exp",
        "UpperExp" => "upper_exp",
        "Pointer" => "pointer",
        _ => unimplemented!(),
    };
    let type_params = input
        .generics
        .type_params()
        .map(|t| t.ident.clone())
        .collect();

    let (arms, bound_type_params) = State {
        trait_path,
        trait_attr,
        input,
        type_params,
    }
    .get_match_arms_and_bound_type_params()?;

    let mut generics = input.generics.clone();
    if !bound_type_params.is_empty() {
        let trait_path = vec![trait_path; bound_type_params.len()];
        let where_clause =
            quote_spanned!(input.span()=> where #(#bound_type_params: #trait_path),*);
        generics = add_extra_where_clauses(&input.generics, where_clause);
    }
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let name = &input.ident;

    Ok(quote! {
        impl #impl_generics #trait_path for #name #ty_generics #where_clause
        {
            #[allow(unused_variables)]
            #[inline]
            fn fmt(&self, _derive_more_Display_formatter: &mut #import_root::fmt::Formatter) -> #import_root::fmt::Result {
                match self {
                    #arms
                    _ => Ok(()) // This is needed for empty enums
                }
            }
        }
    })
}

struct State<'a, 'b> {
    trait_path: &'b TokenStream,
    trait_attr: &'static str,
    input: &'a DeriveInput,
    type_params: HashSet<Ident>,
}

impl<'a, 'b> State<'a, 'b> {
    fn get_proper_syntax(&self) -> impl Display {
        format!(
            r#"Proper syntax: #[{}(fmt = "My format", "arg1", "arg2")]"#,
            self.trait_attr
        )
    }
    fn get_matcher(&self, fields: &Fields) -> TokenStream {
        match fields {
            Fields::Unit => TokenStream::new(),
            Fields::Unnamed(fields) => {
                let fields: TokenStream = (0..fields.unnamed.len())
                    .map(|n| {
                        let i = Ident::new(&format!("_{}", n), Span::call_site());
                        quote!(#i,)
                    })
                    .collect();
                quote!((#fields))
            }
            Fields::Named(fields) => {
                let fields: TokenStream = fields
                    .named
                    .iter()
                    .map(|f| {
                        let i = f.ident.as_ref().unwrap();
                        quote!(#i,)
                    })
                    .collect();
                quote!({#fields})
            }
        }
    }
    fn find_meta(&self, attrs: &[Attribute]) -> Result<Option<Meta>> {
        let mut it = attrs
            .iter()
            .filter_map(|a| a.interpret_meta())
            .filter(|m| m.name() == self.trait_attr);

        let meta = it.next();
        if it.next().is_some() {
            Err(Error::new(meta.span(), "Too many formats given"))
        } else {
            Ok(meta)
        }
    }
    fn get_meta_fmt(&self, meta: &Meta) -> Result<TokenStream> {
        let list = match meta {
            Meta::List(list) => list,
            _ => return Err(Error::new(meta.span(), self.get_proper_syntax())),
        };

        let fmt = match &list.nested[0] {
            NestedMeta::Meta(Meta::NameValue(MetaNameValue {
                ident,
                lit: Lit::Str(s),
                ..
            })) if ident == "fmt" => s,
            _ => return Err(Error::new(list.nested[0].span(), self.get_proper_syntax())),
        };

        let args = list
            .nested
            .iter()
            .skip(1) // skip fmt = "..."
            .try_fold(TokenStream::new(), |args, arg| {
                let arg = match arg {
                    NestedMeta::Literal(Lit::Str(s)) => s,
                    NestedMeta::Meta(Meta::Word(i)) => {
                        return Ok(quote_spanned!(list.span()=> #args #i,));
                    }
                    _ => return Err(Error::new(arg.span(), self.get_proper_syntax())),
                };
                let arg: TokenStream = arg.parse().map_err(|e| Error::new(arg.span(), e))?;
                Ok(quote_spanned!(list.span()=> #args #arg,))
            })?;

        Ok(quote_spanned!(meta.span()=> write!(_derive_more_Display_formatter, #fmt, #args)))
    }
    fn infer_fmt(&self, fields: &Fields, name: &Ident) -> Result<TokenStream> {
        let fields = match fields {
            Fields::Unit => {
                return Ok(quote!(write!(
                    _derive_more_Display_formatter,
                    stringify!(#name)
                )));
            }
            Fields::Named(fields) => &fields.named,
            Fields::Unnamed(fields) => &fields.unnamed,
        };
        if fields.is_empty() {
            return Ok(quote!(write!(
                _derive_more_Display_formatter,
                stringify!(#name)
            )));
        } else if fields.len() > 1 {
            return Err(Error::new(
                fields.span(),
                "Can not automatically infer format for types with more than 1 field",
            ));
        }

        let trait_path = self.trait_path;
        if let Some(ident) = &fields.iter().next().as_ref().unwrap().ident {
            Ok(quote!(#trait_path::fmt(#ident, _derive_more_Display_formatter)))
        } else {
            Ok(quote!(#trait_path::fmt(_0, _derive_more_Display_formatter)))
        }
    }
    fn get_match_arms_and_bound_type_params(&self) -> Result<(TokenStream, HashSet<Type>)> {
        match &self.input.data {
            Data::Enum(e) => {
                if let Some(meta) = self.find_meta(&self.input.attrs)? {
                    let fmt = self.get_meta_fmt(&meta)?;

                    e.variants.iter().try_for_each(|v| {
                        if let Some(meta) = self.find_meta(&v.attrs)? {
                            Err(Error::new(
                                meta.span(),
                                "Can not have a format on the variant when the whole enum has one",
                            ))
                        } else {
                            Ok(())
                        }
                    })?;

                    Ok((
                        quote_spanned!(self.input.span()=> _ => #fmt,),
                        HashSet::new(),
                    ))
                } else {
                    e.variants.iter().try_fold(
                        (TokenStream::new(), HashSet::new()),
                        |(arms, mut all_bound_type_params), v| {
                            let matcher = self.get_matcher(&v.fields);
                            let name = &self.input.ident;
                            let v_name = &v.ident;
                            let fmt: TokenStream;
                            let mut bound_type_params = HashSet::new();

                            if let Some(meta) = self.find_meta(&v.attrs)? {
                                fmt = self.get_meta_fmt(&meta)?;
                                if !self.type_params.is_empty() {
                                    bound_type_params = self.find_used_type_params_in_meta(&v.fields, &meta);
                                }
                            } else {
                                fmt = self.infer_fmt(&v.fields, v_name)?;
                                // TODO: bound_type_params?
                            };
                            all_bound_type_params.extend(bound_type_params);

                            Ok((
                                quote_spanned!(self.input.span()=> #arms #name::#v_name #matcher => #fmt,),
                                all_bound_type_params,
                            ))
                        },
                    )
                }
            }
            Data::Struct(s) => {
                let matcher = self.get_matcher(&s.fields);
                let name = &self.input.ident;
                let fmt: TokenStream;
                let mut bound_type_params = HashSet::new();

                if let Some(meta) = self.find_meta(&self.input.attrs)? {
                    fmt = self.get_meta_fmt(&meta)?;
                    if !self.type_params.is_empty() {
                        bound_type_params = self.find_used_type_params_in_meta(&s.fields, &meta);
                    }
                } else {
                    fmt = self.infer_fmt(&s.fields, name)?;
                    // TODO: bound_type_params?
                }

                Ok((
                    quote_spanned!(self.input.span()=> #name #matcher => #fmt,),
                    bound_type_params,
                ))
            }
            Data::Union(_) => {
                let meta = self.find_meta(&self.input.attrs)?.ok_or_else(|| {
                    Error::new(
                        self.input.span(),
                        "Can not automatically infer format for unions",
                    )
                })?;
                let fmt = self.get_meta_fmt(&meta)?;

                Ok((
                    quote_spanned!(self.input.span()=> _ => #fmt,),
                    HashSet::new(),
                ))
            }
        }
    }
    fn find_used_type_params_in_meta(&self, fields: &Fields, meta: &Meta) -> HashSet<Type> {
        if let Fields::Unit = fields {
            return HashSet::new();
        }

        let list = match meta {
            Meta::List(list) => list,
            _ => return HashSet::new(),
        };
        let used_args: HashSet<Ident> = list
            .nested
            .iter()
            .skip(1) // skip fmt = "..."
            .filter_map(|arg| match arg {
                NestedMeta::Literal(Lit::Str(ref s)) => {
                    Some(Ident::new(&s.value(), Span::call_site()))
                }
                NestedMeta::Meta(Meta::Word(ref i)) => Some(i.clone()),
                _ => None,
            })
            .collect();
        // TODO: filter used_args by real use in the fmt string (trait dependent)
        if used_args.is_empty() {
            return HashSet::new();
        }

        // TODO: correctly work with non-trivial args usages like "_0.display()"
        let used_fields: HashSet<&syn::Field> = match fields {
            Fields::Unnamed(fields) => (0..fields.unnamed.len())
                .filter(|n| {
                    let i = Ident::new(&format!("_{}", n), Span::call_site());
                    used_args.contains(&i)
                })
                .map(|n| &fields.unnamed[n])
                .collect(),
            Fields::Named(fields) => fields
                .named
                .iter()
                .filter(|f| f.ident.is_some() && used_args.contains(f.ident.as_ref().unwrap()))
                .collect(),
            _ => unreachable!(),
        };
        used_fields
            .into_iter()
            .filter_map(|f| {
                if let Type::Path(ref ty) = f.ty {
                    if let Some(t) = match ty.path.segments.first() {
                        // TODO: test Punctuated variant
                        Some(Pair::Punctuated(ref t, _)) => Some(t),
                        Some(Pair::End(ref t)) => Some(t),
                        _ => None,
                    } {
                        if self.type_params.contains(&t.ident) {
                            return Some(f.ty.clone());
                        }
                    }
                }
                None
            })
            .collect()
    }
}
