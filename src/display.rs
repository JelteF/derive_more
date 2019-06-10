use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
};

use proc_macro2::{Ident, Span, TokenStream};
use regex::Regex;
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

    let (arms, bounds) = State {
        trait_path,
        trait_attr,
        input,
        type_params,
    }
    .get_match_arms_and_extra_bounds()?;

    let generics = if !bounds.is_empty() {
        let bounds: Vec<_> = bounds
            .into_iter()
            .map(|(ty, trait_names)| {
                let bounds: Vec<_> = trait_names
                    .into_iter()
                    .map(|trait_name| {
                        let import_root = get_import_root();
                        let trait_ident = Ident::new(trait_name, Span::call_site());
                        quote!(#import_root::fmt::#trait_ident)
                    })
                    .collect();
                quote!(#ty: #(#bounds)+*)
            })
            .collect();
        let where_clause = quote_spanned!(input.span()=> where #(#bounds),*);
        add_extra_where_clauses(&input.generics, where_clause)
    } else {
        input.generics.clone()
    };
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
            .filter_map(Attribute::interpret_meta)
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
            Fields::Unit => return Ok(quote!(stringify!(#name))),
            Fields::Named(fields) => &fields.named,
            Fields::Unnamed(fields) => &fields.unnamed,
        };
        if fields.is_empty() {
            return Ok(quote!(stringify!(#name)));
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
    fn get_match_arms_and_extra_bounds(
        &self,
    ) -> Result<(TokenStream, HashMap<Type, HashSet<&'static str>>)> {
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
                        HashMap::new(),
                    ))
                } else {
                    e.variants.iter().try_fold(
                        (TokenStream::new(), HashMap::new()),
                        |(arms, mut all_bounds), v| {
                            let matcher = self.get_matcher(&v.fields);
                            let name = &self.input.ident;
                            let v_name = &v.ident;
                            let fmt: TokenStream;
                            let bounds: HashMap<_, _>;

                            if let Some(meta) = self.find_meta(&v.attrs)? {
                                fmt = self.get_meta_fmt(&meta)?;
                                bounds = self.get_used_type_params_bounds(&v.fields, &meta);
                            } else {
                                fmt = self.infer_fmt(&v.fields, v_name)?;
                                bounds = self.infer_type_params_bounds(&v.fields);
                            };
                            all_bounds = bounds.into_iter()
                                .fold(all_bounds, |mut bounds, (ty, trait_names)| {
                                    bounds.entry(ty).or_insert_with(HashSet::new).extend(trait_names);
                                    bounds
                                });

                            Ok((
                                quote_spanned!(self.input.span()=> #arms #name::#v_name #matcher => #fmt,),
                                all_bounds,
                            ))
                        },
                    )
                }
            }
            Data::Struct(s) => {
                let matcher = self.get_matcher(&s.fields);
                let name = &self.input.ident;
                let fmt: TokenStream;
                let bounds: HashMap<_, _>;

                if let Some(meta) = self.find_meta(&self.input.attrs)? {
                    fmt = self.get_meta_fmt(&meta)?;
                    bounds = self.get_used_type_params_bounds(&s.fields, &meta);
                } else {
                    fmt = self.infer_fmt(&s.fields, name)?;
                    bounds = self.infer_type_params_bounds(&s.fields);
                }

                Ok((
                    quote_spanned!(self.input.span()=> #name #matcher => #fmt,),
                    bounds,
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
                    HashMap::new(),
                ))
            }
        }
    }
    fn get_used_type_params_bounds(
        &self,
        fields: &Fields,
        meta: &Meta,
    ) -> HashMap<Type, HashSet<&'static str>> {
        if self.type_params.is_empty() {
            return HashMap::new();
        }

        let fields_type_params: HashMap<_, _> = fields
            .iter()
            .enumerate()
            .filter_map(|(i, field)| {
                if !self.has_type_param_in(field) {
                    return None;
                }
                let ident = field
                    .ident
                    .clone()
                    .unwrap_or_else(|| Ident::new(&format!("_{}", i), Span::call_site()));
                Some((ident, field.ty.clone()))
            })
            .collect();
        if fields_type_params.is_empty() {
            return HashMap::new();
        }

        let list = match meta {
            Meta::List(list) => list,
            // This one has been checked already in get_meta_fmt() method.
            _ => unreachable!(),
        };
        let fmt_args: HashMap<_, _> = list
            .nested
            .iter()
            .skip(1) // skip fmt = "..."
            .enumerate()
            .filter_map(|(i, arg)| match arg {
                NestedMeta::Literal(Lit::Str(ref s)) => {
                    syn::parse_str(&s.value()).ok().map(|id| (i, id))
                }
                NestedMeta::Meta(Meta::Word(ref id)) => Some((i, id.clone())),
                // This one has been checked already in get_meta_fmt() method.
                _ => unreachable!(),
            })
            .collect();
        if fmt_args.is_empty() {
            return HashMap::new();
        }
        let fmt_string = match &list.nested[0] {
            NestedMeta::Meta(Meta::NameValue(MetaNameValue {
                ident,
                lit: Lit::Str(s),
                ..
            })) if ident == "fmt" => s.value(),
            // This one has been checked already in get_meta_fmt() method.
            _ => unreachable!(),
        };

        Placeholder::parse_fmt_string(&fmt_string).into_iter().fold(
            HashMap::new(),
            |mut bounds, pl| {
                if let Some(arg) = fmt_args.get(&pl.position) {
                    if fields_type_params.contains_key(arg) {
                        bounds
                            .entry(fields_type_params[arg].clone())
                            .or_insert_with(HashSet::new)
                            .insert(pl.trait_name);
                    }
                }
                bounds
            },
        )
    }
    fn infer_type_params_bounds(&self, fields: &Fields) -> HashMap<Type, HashSet<&'static str>> {
        if self.type_params.is_empty() {
            return HashMap::new();
        }
        if let Fields::Unit = fields {
            return HashMap::new();
        }
        // infer_fmt() uses only first field.
        fields
            .iter()
            .take(1)
            .filter_map(|field| {
                if !self.has_type_param_in(field) {
                    return None;
                }
                Some((
                    field.ty.clone(),
                    [match self.trait_attr {
                        "display" => "Display",
                        "binary" => "Binary",
                        "octal" => "Octal",
                        "lower_hex" => "LowerHex",
                        "upper_hex" => "UpperHex",
                        "lower_exp" => "LowerExp",
                        "upper_exp" => "UpperExp",
                        "pointer" => "Pointer",
                        _ => unreachable!(),
                    }]
                    .iter()
                    .cloned()
                    .collect(),
                ))
            })
            .collect()
    }
    fn has_type_param_in(&self, field: &syn::Field) -> bool {
        if let Type::Path(ref ty) = field.ty {
            return match ty.path.segments.first() {
                Some(Pair::Punctuated(ref t, _)) => self.type_params.contains(&t.ident),
                Some(Pair::End(ref t)) => self.type_params.contains(&t.ident),
                _ => false,
            };
        }
        false
    }
}

lazy_static! {
    /// Regular expression for parsing formatting placeholders from a string.
    ///
    /// Reproduces `maybe-format` expression of [formatting syntax][1].
    ///
    /// [1]: https://doc.rust-lang.org/stable/std/fmt/index.html#syntax
    static ref MAYBE_PLACEHOLDER: Regex = Regex::new(
        r"(\{\{|}}|(?P<placeholder>\{[^{}]*}))",
    ).unwrap();

    /// Regular expression for parsing inner type of formatting placeholder.
    ///
    /// Reproduces `format` expression of [formatting syntax][1], but is simplified
    /// in the following way (as we need to parse `type` only):
    /// - `argument` is replaced just with `\d+` (instead of [`identifier`][2]);
    /// - `character` is allowed to be any symbol.
    ///
    /// [1]: https://doc.rust-lang.org/stable/std/fmt/index.html#syntax
    /// [2]: https://doc.rust-lang.org/reference/identifiers.html#identifiers
    static ref PLACEHOLDER_FORMAT: Regex = Regex::new(
        r"^\{(?P<arg>\d+)?(:(.?[<^>])?[+-]?#?0?(\w+\$|\d+)?(\.(\w+\$|\d+|\*))?(?P<type>([oxXpbeE?]|[xX]\?)?)?)?}$",
    ).unwrap();
}

/// Representation of formatting placeholder.
#[derive(Debug, PartialEq)]
struct Placeholder {
    /// Position of formatting argument to be used for this placeholder.
    position: usize,
    /// Name of [`std::fmt`] trait to be used for rendering this placeholder.
    trait_name: &'static str,
}

impl Placeholder {
    /// Parses [`Placeholder`]s from a given formatting string.
    fn parse_fmt_string(s: &str) -> Vec<Placeholder> {
        let mut n = 0;
        MAYBE_PLACEHOLDER
            .captures_iter(s)
            .filter_map(|cap| cap.name("placeholder"))
            .map(|m| {
                let captured = PLACEHOLDER_FORMAT.captures(m.as_str()).unwrap();
                let position = captured
                    .name("arg")
                    .map(|s| s.as_str().parse().unwrap())
                    .unwrap_or_else(|| {
                        // Assign "the next argument".
                        // https://doc.rust-lang.org/stable/std/fmt/index.html#positional-parameters
                        n += 1;
                        n - 1
                    });
                let typ = captured
                    .name("type")
                    .map(|s| s.as_str())
                    .unwrap_or_default();
                let trait_name = match typ {
                    "" => "Display",
                    "?" | "x?" | "X?" => "Debug",
                    "o" => "Octal",
                    "x" => "LowerHex",
                    "X" => "UpperHex",
                    "p" => "Pointer",
                    "b" => "Binary",
                    "e" => "LowerExp",
                    "E" => "UpperExp",
                    _ => unreachable!(),
                };
                Placeholder {
                    position,
                    trait_name,
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod regex_maybe_placeholder_spec {
    use super::*;

    #[test]
    fn parses_placeholders_and_omits_escaped() {
        let fmt_string = "{}, {:?}, {{}}, {{{1:0$}}}";
        let placeholders: Vec<_> = MAYBE_PLACEHOLDER
            .captures_iter(&fmt_string)
            .filter_map(|cap| cap.name("placeholder"))
            .map(|m| m.as_str())
            .collect();
        assert_eq!(placeholders, vec!["{}", "{:?}", "{1:0$}"]);
    }
}

#[cfg(test)]
mod regex_placeholder_format_spec {
    use super::*;

    #[test]
    fn detects_type() {
        for (p, expected) in vec![
            ("{}", ""),
            ("{:?}", "?"),
            ("{:x?}", "x?"),
            ("{:X?}", "X?"),
            ("{:o}", "o"),
            ("{:x}", "x"),
            ("{:X}", "X"),
            ("{:p}", "p"),
            ("{:b}", "b"),
            ("{:e}", "e"),
            ("{:E}", "E"),
            ("{:.*}", ""),
            ("{8}", ""),
            ("{:04}", ""),
            ("{1:0$}", ""),
            ("{:width$}", ""),
            ("{9:>8.*}", ""),
            ("{2:.1$x}", "x"),
        ] {
            let typ = PLACEHOLDER_FORMAT
                .captures(p)
                .unwrap()
                .name("type")
                .map(|s| s.as_str())
                .unwrap_or_default();
            assert_eq!(typ, expected);
        }
    }

    #[test]
    fn detects_arg() {
        for (p, expected) in vec![
            ("{}", ""),
            ("{0:?}", "0"),
            ("{12:x?}", "12"),
            ("{3:X?}", "3"),
            ("{5:o}", "5"),
            ("{6:x}", "6"),
            ("{:X}", ""),
            ("{8}", "8"),
            ("{:04}", ""),
            ("{1:0$}", "1"),
            ("{:width$}", ""),
            ("{9:>8.*}", "9"),
            ("{2:.1$x}", "2"),
        ] {
            let arg = PLACEHOLDER_FORMAT
                .captures(p)
                .unwrap()
                .name("arg")
                .map(|s| s.as_str())
                .unwrap_or_default();
            assert_eq!(arg, expected);
        }
    }
}

#[cfg(test)]
mod placeholder_parse_fmt_string_spec {
    use super::*;

    #[test]
    fn indicates_position_and_trait_name_for_each_fmt_placeholder() {
        let fmt_string = "{},{:?},{{}},{{{1:0$}}}-{2:.1$x}{0:#?}{:width$}";
        assert_eq!(
            Placeholder::parse_fmt_string(&fmt_string),
            vec![
                Placeholder {
                    position: 0,
                    trait_name: "Display",
                },
                Placeholder {
                    position: 1,
                    trait_name: "Debug",
                },
                Placeholder {
                    position: 1,
                    trait_name: "Display",
                },
                Placeholder {
                    position: 2,
                    trait_name: "LowerHex",
                },
                Placeholder {
                    position: 0,
                    trait_name: "Debug",
                },
                Placeholder {
                    position: 2,
                    trait_name: "Display",
                },
            ],
        )
    }
}
