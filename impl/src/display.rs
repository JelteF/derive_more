use std::{fmt::Display, iter, mem, str::FromStr as _};

use proc_macro2::{Ident, Span, TokenStream};
use quote::{format_ident, quote, quote_spanned, ToTokens};
use syn::{
    ext::IdentExt as _,
    parse::{discouraged::Speculative as _, Parse, ParseStream, Parser},
    parse_quote,
    punctuated::Punctuated,
    spanned::Spanned as _,
    Error, Result,
};

use crate::{
    parsing,
    utils::{self, HashMap, HashSet},
};

// /// Allowed [`syn::MetaNameValue`] arguments of `#[display]` attribute.
// const ALLOWED_ATTRIBUTE_ARGUMENTS: &[&str] = &["fmt", "bound"];
//
// /// Provides the hook to expand `#[derive(Display)]` into an implementation of `From`
// pub fn expand(input: &syn::DeriveInput, trait_name: &str) -> Result<TokenStream> {
//     let trait_name = trait_name.trim_end_matches("Custom");
//     let trait_ident = syn::Ident::new(trait_name, Span::call_site());
//     let trait_path = &quote!(::core::fmt::#trait_ident);
//     let trait_attr = trait_name_to_attribute_name(trait_name);
//     let type_params = input
//         .generics
//         .type_params()
//         .map(|t| t.ident.clone())
//         .collect();
//
//     let ParseResult {
//         arms,
//         bounds,
//         requires_helper,
//     } = State {
//         trait_path,
//         trait_attr,
//         input,
//         type_params,
//     }
//     .get_match_arms_and_extra_bounds()?;
//
//     let generics = if !bounds.is_empty() {
//         let bounds: Vec<_> = bounds
//             .into_iter()
//             .map(|(ty, trait_names)| {
//                 let bounds: Vec<_> = trait_names
//                     .into_iter()
//                     .map(|bound| quote!(#bound))
//                     .collect();
//                 quote!(#ty: #(#bounds)+*)
//             })
//             .collect();
//         let where_clause = quote_spanned!(input.span()=> where #(#bounds),*);
//         utils::add_extra_where_clauses(&input.generics, where_clause)
//     } else {
//         input.generics.clone()
//     };
//     let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
//     let name = &input.ident;
//
//     let helper_struct = if requires_helper {
//         display_as_helper_struct()
//     } else {
//         TokenStream::new()
//     };
//
//     Ok(quote! {
//         #[automatically_derived]
//         impl #impl_generics #trait_path for #name #ty_generics #where_clause {
//             #[inline]
//             fn fmt(&self, _derive_more_display_formatter: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
//                 #helper_struct
//
//                 match *self {
//                     #arms
//                 }
//             }
//         }
//     })
// }

fn trait_name_to_attribute_name(trait_name: &str) -> &'static str {
    match trait_name {
        "Display" => "display",
        "Binary" => "binary",
        "Octal" => "octal",
        "LowerHex" => "lower_hex",
        "UpperHex" => "upper_hex",
        "LowerExp" => "lower_exp",
        "UpperExp" => "upper_exp",
        "Pointer" => "pointer",
        "Debug" => "debug",
        _ => unimplemented!(),
    }
}

// fn attribute_name_to_trait_name(attribute_name: &str) -> &'static str {
//     match attribute_name {
//         "display" => "Display",
//         "binary" => "Binary",
//         "octal" => "Octal",
//         "lower_hex" => "LowerHex",
//         "upper_hex" => "UpperHex",
//         "lower_exp" => "LowerExp",
//         "upper_exp" => "UpperExp",
//         "pointer" => "Pointer",
//         _ => unreachable!(),
//     }
// }
//
// fn trait_name_to_trait_bound(trait_name: &str) -> syn::TraitBound {
//     let path_segments_iterator = vec!["core", "fmt", trait_name]
//         .into_iter()
//         .map(|segment| syn::PathSegment::from(Ident::new(segment, Span::call_site())));
//
//     syn::TraitBound {
//         lifetimes: None,
//         modifier: syn::TraitBoundModifier::None,
//         paren_token: None,
//         path: syn::Path {
//             leading_colon: Some(syn::Token![::](Span::call_site())),
//             segments: path_segments_iterator.collect(),
//         },
//     }
// }
//
// /// Create a helper struct that is required by some `Display` impls.
// ///
// /// The struct is necessary in cases where `Display` is derived for an enum
// /// with an outer `#[display(fmt = "...")]` attribute and if that outer
// /// format-string contains a single placeholder. In that case, we have to
// /// format twice:
// ///
// /// - we need to format each variant according to its own, optional
// ///   format-string,
// /// - we then need to insert this formatted variant into the outer
// ///   format-string.
// ///
// /// This helper struct solves this as follows:
// /// - formatting the whole object inserts the helper struct into the outer
// ///   format string,
// /// - upon being formatted, the helper struct calls an inner closure to produce
// ///   its formatted result,
// /// - the closure in turn uses the inner, optional format-string to produce its
// ///   result. If there is no inner format-string, it falls back to plain
// ///   `$trait::fmt()`.
// fn display_as_helper_struct() -> TokenStream {
//     quote! {
//         struct _derive_more_DisplayAs<F>(F)
//         where
//             F: ::core::ops::Fn(&mut ::core::fmt::Formatter) -> ::core::fmt::Result;
//
//         const _derive_more_DisplayAs_impl: () = {
//             impl<F> ::core::fmt::Display for _derive_more_DisplayAs<F>
//             where
//                 F: ::core::ops::Fn(&mut ::core::fmt::Formatter) -> ::core::fmt::Result
//             {
//                 fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
//                     (self.0)(f)
//                 }
//             }
//         };
//     }
// }
//
// /// Result type of `State::get_match_arms_and_extra_bounds()`.
// #[derive(Default)]
// struct ParseResult {
//     /// The match arms destructuring `self`.
//     arms: TokenStream,
//     /// Any trait bounds that may be required.
//     bounds: HashMap<syn::Type, HashSet<syn::TraitBound>>,
//     /// `true` if the Display impl requires the `DisplayAs` helper struct.
//     requires_helper: bool,
// }
//
// struct State<'a, 'b> {
//     trait_path: &'b TokenStream,
//     trait_attr: &'static str,
//     input: &'a syn::DeriveInput,
//     type_params: HashSet<Ident>,
// }
//
// impl<'a, 'b> State<'a, 'b> {
//     fn get_proper_fmt_syntax(&self) -> impl Display {
//         format!(
//             r#"Proper syntax: #[{}(fmt = "My format", arg1, arg2)]"#,
//             self.trait_attr
//         )
//     }
//     fn get_proper_bound_syntax(&self) -> impl Display {
//         format!(
//             "Proper syntax: #[{}(bound = \"T, U: Trait1 + Trait2, V: Trait3\")]",
//             self.trait_attr
//         )
//     }
//
//     fn get_matcher(&self, fields: &syn::Fields) -> TokenStream {
//         match fields {
//             syn::Fields::Unit => TokenStream::new(),
//             syn::Fields::Unnamed(fields) => {
//                 let fields: TokenStream = (0..fields.unnamed.len())
//                     .map(|n| {
//                         let i = Ident::new(&format!("_{}", n), Span::call_site());
//                         quote!(ref #i,)
//                     })
//                     .collect();
//                 quote!((#fields))
//             }
//             syn::Fields::Named(fields) => {
//                 let fields: TokenStream = fields
//                     .named
//                     .iter()
//                     .map(|f| {
//                         let i = f.ident.as_ref().unwrap();
//                         quote!(ref #i,)
//                     })
//                     .collect();
//                 quote!({#fields})
//             }
//         }
//     }
//     fn find_meta(
//         &self,
//         attrs: &[syn::Attribute],
//         meta_key: &str,
//     ) -> Result<Option<MetaExpr>> {
//         let mut metas = Vec::new();
//         for attr in attrs
//             .iter()
//             .filter(|attr| attr.path.is_ident(self.trait_attr))
//         {
//             let meta = MetaExpr::from_attr(attr)?;
//             let meta_list = match &meta {
//                 MetaExpr::List(meta) => meta,
//                 _ => continue,
//             };
//
//             let meta_nv = match meta_list.nested.first() {
//                 Some(NestedMetaExpr::Meta(MetaExpr::NameValue(meta_nv))) => {
//                     if ALLOWED_ATTRIBUTE_ARGUMENTS
//                         .iter()
//                         .any(|attr| meta_nv.path.is_ident(attr))
//                     {
//                         meta_nv
//                     } else {
//                         return Err(Error::new(
//                             meta_nv.path.span(),
//                             format!(
//                                 "Unknown `{}` attribute argument. \
//                                  Allowed arguments are: {}",
//                                 meta_nv.path.to_token_stream(),
//                                 ALLOWED_ATTRIBUTE_ARGUMENTS
//                                     .iter()
//                                     .fold(None, |acc, key| acc.map_or_else(
//                                         || Some(key.to_string()),
//                                         |acc| Some(format!("{}, {}", acc, key))
//                                     ))
//                                     .unwrap_or_default(),
//                             ),
//                         ));
//                     }
//                 }
//                 _ => {
//                     // If the given attribute is not MetaNameValue, it most likely implies that the
//                     // user is writing an incorrect format. For example:
//                     // - `#[display()]`
//                     // - `#[display("foo")]`
//                     // - `#[display(foo)]`
//                     return Err(Error::new(
//                         meta.span(),
//                         format!(
//                             r#"The format for this attribute cannot be parsed. Correct format: `#[{}({} = "...")]`"#,
//                             self.trait_attr, meta_key
//                         ),
//                     ));
//                 }
//             };
//
//             if meta_nv.path.is_ident(meta_key) {
//                 metas.push(meta);
//             }
//         }
//
//         let mut iter = metas.into_iter();
//         let meta = iter.next();
//         if iter.next().is_none() {
//             Ok(meta)
//         } else {
//             Err(Error::new(meta.span(), "Too many attributes specified"))
//         }
//     }
//     fn parse_meta_bounds(
//         &self,
//         bounds: &syn::LitStr,
//     ) -> Result<HashMap<syn::Type, HashSet<syn::TraitBound>>> {
//         let span = bounds.span();
//
//         let input = bounds.value();
//         let tokens = TokenStream::from_str(&input)?;
//         let parser = Punctuated::<syn::GenericParam, syn::Token![,]>::parse_terminated;
//
//         let generic_params = parser
//             .parse2(tokens)
//             .map_err(|error| Error::new(span, error.to_string()))?;
//
//         if generic_params.is_empty() {
//             return Err(Error::new(span, "No bounds specified"));
//         }
//
//         let mut bounds = HashMap::default();
//
//         for generic_param in generic_params {
//             let type_param = match generic_param {
//                 syn::GenericParam::Type(type_param) => type_param,
//                 _ => return Err(Error::new(span, "Only trait bounds allowed")),
//             };
//
//             if !self.type_params.contains(&type_param.ident) {
//                 return Err(Error::new(
//                     span,
//                     "Unknown generic type argument specified",
//                 ));
//             } else if !type_param.attrs.is_empty() {
//                 return Err(Error::new(span, "Attributes aren't allowed"));
//             } else if type_param.eq_token.is_some() || type_param.default.is_some() {
//                 return Err(Error::new(span, "Default type parameters aren't allowed"));
//             }
//
//             let ident = type_param.ident.to_string();
//
//             let ty = syn::Type::Path(syn::TypePath {
//                 qself: None,
//                 path: type_param.ident.into(),
//             });
//             let bounds = bounds.entry(ty).or_insert_with(HashSet::default);
//
//             for bound in type_param.bounds {
//                 let bound = match bound {
//                     syn::TypeParamBound::Trait(bound) => bound,
//                     _ => return Err(Error::new(span, "Only trait bounds allowed")),
//                 };
//
//                 if bound.lifetimes.is_some() {
//                     return Err(Error::new(
//                         span,
//                         "Higher-rank trait bounds aren't allowed",
//                     ));
//                 }
//
//                 bounds.insert(bound);
//             }
//
//             if bounds.is_empty() {
//                 return Err(Error::new(
//                     span,
//                     format!("No bounds specified for type parameter {}", ident),
//                 ));
//             }
//         }
//
//         Ok(bounds)
//     }
//     fn parse_meta_fmt(
//         &self,
//         meta: &MetaExpr,
//         outer_enum: bool,
//     ) -> Result<(TokenStream, bool)> {
//         let list = match meta {
//             MetaExpr::List(list) => list,
//             _ => {
//                 return Err(Error::new(meta.span(), self.get_proper_fmt_syntax()));
//             }
//         };
//
//         match &list.nested[0] {
//             NestedMetaExpr::Meta(MetaExpr::NameValue(MetaNameValueExpr {
//                 path,
//                 expr:
//                     syn::Expr::Lit(syn::ExprLit {
//                         lit: syn::Lit::Str(fmt),
//                         ..
//                     }),
//                 ..
//             })) => match path {
//                 op if op.segments.first().expect("path shouldn't be empty").ident
//                     == "fmt" =>
//                 {
//                     let expected_affix_usage = "outer `enum` `fmt` is an affix spec that expects no args and at most 1 placeholder for inner variant display";
//                     let placeholders = Placeholder::parse_fmt_string(&fmt.value());
//                     if outer_enum {
//                         if list.nested.iter().skip(1).count() != 0 {
//                             return Err(Error::new(
//                                 list.nested[1].span(),
//                                 expected_affix_usage,
//                             ));
//                         }
//                         if placeholders.len() > 1
//                             || placeholders
//                                 .first()
//                                 .map(|p| p.arg != Parameter::Positional(0))
//                                 .unwrap_or_default()
//                         {
//                             return Err(Error::new(
//                                 list.nested[1].span(),
//                                 expected_affix_usage,
//                             ));
//                         }
//                         if placeholders.len() == 1 {
//                             return Ok((quote_spanned!(fmt.span()=> #fmt), true));
//                         }
//                     }
//                     let args = list
//                         .nested
//                         .iter()
//                         .skip(1) // skip fmt = "..."
//                         .try_fold(TokenStream::new(), |args, arg| match arg {
//                             NestedMetaExpr::Expr(expr) => {
//                                 Ok(quote_spanned!(list.span()=> #args #expr,))
//                             }
//                             NestedMetaExpr::Meta(MetaExpr::Path(i)) => {
//                                 Ok(quote_spanned!(list.span()=> #args #i,))
//                             }
//                             _ => Err(Error::new(
//                                 arg.span(),
//                                 self.get_proper_fmt_syntax(),
//                             )),
//                         })?;
//
//                     let interpolated_args = placeholders
//                         .into_iter()
//                         .flat_map(|p| {
//                             let map_argument = |arg| match arg {
//                                 Parameter::Named(i) => Some(i),
//                                 Parameter::Positional(_) => None,
//                             };
//                             map_argument(p.arg)
//                                 .into_iter()
//                                 .chain(p.width.and_then(map_argument))
//                                 .chain(p.precision.and_then(map_argument))
//                         })
//                         .collect::<HashSet<_>>()
//                         .into_iter()
//                         .map(|ident| {
//                             let ident = syn::Ident::new(&ident, fmt.span());
//                             quote! { #ident = #ident, }
//                         })
//                         .collect::<TokenStream>();
//
//                     Ok((
//                         quote_spanned!(meta.span()=> write!(_derive_more_display_formatter, #fmt, #args #interpolated_args)),
//                         false,
//                     ))
//                 }
//                 _ => Err(Error::new(
//                     list.nested[0].span(),
//                     self.get_proper_fmt_syntax(),
//                 )),
//             },
//             _ => Err(Error::new(
//                 list.nested[0].span(),
//                 self.get_proper_fmt_syntax(),
//             )),
//         }
//     }
//     fn infer_fmt(&self, fields: &syn::Fields, name: &Ident) -> Result<TokenStream> {
//         let fields = match fields {
//             syn::Fields::Unit => {
//                 return Ok(quote!(
//                     _derive_more_display_formatter.write_str(stringify!(#name))
//                 ))
//             }
//             syn::Fields::Named(fields) => &fields.named,
//             syn::Fields::Unnamed(fields) => &fields.unnamed,
//         };
//         if fields.is_empty() {
//             return Ok(quote!(
//                 _derive_more_display_formatter.write_str(stringify!(#name))
//             ));
//         } else if fields.len() > 1 {
//             return Err(Error::new(
//                 fields.span(),
//                 "Cannot automatically infer format for types with more than 1 field",
//             ));
//         }
//
//         let trait_path = self.trait_path;
//         if let Some(ident) = &fields.iter().next().as_ref().unwrap().ident {
//             Ok(quote!(#trait_path::fmt(#ident, _derive_more_display_formatter)))
//         } else {
//             Ok(quote!(#trait_path::fmt(_0, _derive_more_display_formatter)))
//         }
//     }
//     fn get_match_arms_and_extra_bounds(&self) -> Result<ParseResult> {
//         let result: Result<_> = match &self.input.data {
//             syn::Data::Enum(e) => {
//                 match self
//                     .find_meta(&self.input.attrs, "fmt")
//                     .and_then(|m| m.map(|m| self.parse_meta_fmt(&m, true)).transpose())?
//                 {
//                     // #[display(fmt = "no placeholder")] on whole enum.
//                     Some((fmt, false)) => {
//                         e.variants.iter().try_for_each(|v| {
//                             if let Some(meta) = self.find_meta(&v.attrs, "fmt")? {
//                                 Err(Error::new(
//                                     meta.span(),
//                                     "`fmt` cannot be used on variant when the whole enum has a format string without a placeholder, maybe you want to add a placeholder?",
//                                 ))
//                             } else {
//                                 Ok(())
//                             }
//                         })?;
//
//                         Ok(ParseResult {
//                             arms: quote_spanned!(self.input.span()=> _ => #fmt,),
//                             bounds: HashMap::default(),
//                             requires_helper: false,
//                         })
//                     }
//                     // #[display(fmt = "one placeholder: {}")] on whole enum.
//                     Some((outer_fmt, true)) => {
//                         let fmt: Result<TokenStream> = e.variants.iter().try_fold(TokenStream::new(), |arms, v| {
//                             let matcher = self.get_matcher(&v.fields);
//                             let fmt = if let Some(meta) = self.find_meta(&v.attrs, "fmt")? {
//                                 self.parse_meta_fmt(&meta, false)?.0
//                             } else {
//                                 self.infer_fmt(&v.fields, &v.ident)?
//                             };
//                             let v_name = &v.ident;
//                             Ok(quote_spanned!(fmt.span()=> #arms Self::#v_name #matcher => write!(
//                                 _derive_more_display_formatter,
//                                 #outer_fmt,
//                                 _derive_more_DisplayAs(|_derive_more_display_formatter| #fmt)
//                             ),))
//                         });
//                         let fmt = fmt?;
//                         Ok(ParseResult {
//                             arms: quote_spanned!(self.input.span()=> #fmt),
//                             bounds: HashMap::default(),
//                             requires_helper: true,
//                         })
//                     }
//                     // No format attribute on whole enum.
//                     None => e.variants.iter().try_fold(ParseResult::default(), |result, v| {
//                         let ParseResult{ arms, mut bounds, requires_helper } = result;
//                         let matcher = self.get_matcher(&v.fields);
//                         let v_name = &v.ident;
//                         let fmt: TokenStream;
//                         let these_bounds: HashMap<_, _>;
//
//                         if let Some(meta) = self.find_meta(&v.attrs, "fmt")? {
//                             fmt = self.parse_meta_fmt(&meta, false)?.0;
//                             these_bounds = self.get_used_type_params_bounds(&v.fields, &meta);
//                         } else {
//                             fmt = self.infer_fmt(&v.fields, v_name)?;
//                             these_bounds = self.infer_type_params_bounds(&v.fields);
//                         };
//                         these_bounds.into_iter().for_each(|(ty, trait_names)| {
//                             bounds.entry(ty).or_default().extend(trait_names)
//                         });
//                         let arms = quote_spanned!(self.input.span()=> #arms Self::#v_name #matcher => #fmt,);
//
//                         Ok(ParseResult{ arms, bounds, requires_helper })
//                     }),
//                 }
//             }
//             syn::Data::Struct(s) => {
//                 let matcher = self.get_matcher(&s.fields);
//                 let name = &self.input.ident;
//                 let fmt: TokenStream;
//                 let bounds: HashMap<_, _>;
//
//                 if let Some(meta) = self.find_meta(&self.input.attrs, "fmt")? {
//                     fmt = self.parse_meta_fmt(&meta, false)?.0;
//                     bounds = self.get_used_type_params_bounds(&s.fields, &meta);
//                 } else {
//                     fmt = self.infer_fmt(&s.fields, name)?;
//                     bounds = self.infer_type_params_bounds(&s.fields);
//                 }
//
//                 Ok(ParseResult {
//                     arms: quote_spanned!(self.input.span()=> #name #matcher => #fmt,),
//                     bounds,
//                     requires_helper: false,
//                 })
//             }
//             syn::Data::Union(_) => {
//                 let meta =
//                     self.find_meta(&self.input.attrs, "fmt")?.ok_or_else(|| {
//                         Error::new(
//                             self.input.span(),
//                             "Cannot automatically infer format for unions",
//                         )
//                     })?;
//                 let fmt = self.parse_meta_fmt(&meta, false)?.0;
//
//                 Ok(ParseResult {
//                     arms: quote_spanned!(self.input.span()=> _ => #fmt,),
//                     bounds: HashMap::default(),
//                     requires_helper: false,
//                 })
//             }
//         };
//
//         let mut result = result?;
//
//         let meta = match self.find_meta(&self.input.attrs, "bound")? {
//             Some(meta) => meta,
//             _ => return Ok(result),
//         };
//
//         let span = meta.span();
//
//         let meta = match meta {
//             MetaExpr::List(meta) => meta.nested,
//             _ => return Err(Error::new(span, self.get_proper_bound_syntax())),
//         };
//
//         if meta.len() != 1 {
//             return Err(Error::new(span, self.get_proper_bound_syntax()));
//         }
//
//         let meta = match &meta[0] {
//             NestedMetaExpr::Meta(MetaExpr::NameValue(meta)) => meta,
//             _ => return Err(Error::new(span, self.get_proper_bound_syntax())),
//         };
//
//         let extra_bounds = match &meta.expr {
//             syn::Expr::Lit(syn::ExprLit {
//                 lit: syn::Lit::Str(extra_bounds),
//                 ..
//             }) => extra_bounds,
//             _ => return Err(Error::new(span, self.get_proper_bound_syntax())),
//         };
//
//         let extra_bounds = self.parse_meta_bounds(extra_bounds)?;
//
//         extra_bounds.into_iter().for_each(|(ty, trait_names)| {
//             result.bounds.entry(ty).or_default().extend(trait_names)
//         });
//
//         Ok(result)
//     }
//     fn get_used_type_params_bounds(
//         &self,
//         fields: &syn::Fields,
//         meta: &MetaExpr,
//     ) -> HashMap<syn::Type, HashSet<syn::TraitBound>> {
//         if self.type_params.is_empty() {
//             return HashMap::default();
//         }
//
//         let fields_type_params: HashMap<syn::Path, _> = fields
//             .iter()
//             .enumerate()
//             .filter_map(|(i, field)| {
//                 utils::get_if_type_parameter_used_in_type(&self.type_params, &field.ty)
//                     .map(|ty| {
//                         (
//                             field
//                                 .ident
//                                 .clone()
//                                 .unwrap_or_else(|| {
//                                     Ident::new(&format!("_{}", i), Span::call_site())
//                                 })
//                                 .into(),
//                             ty,
//                         )
//                     })
//             })
//             .collect();
//         if fields_type_params.is_empty() {
//             return HashMap::default();
//         }
//
//         let list = match meta {
//             MetaExpr::List(list) => list,
//             // This one has been checked already in get_meta_fmt() method.
//             _ => unreachable!(),
//         };
//         let fmt_args: HashMap<_, _> = list
//             .nested
//             .iter()
//             .skip(1) // skip fmt = "..."
//             .enumerate()
//             .filter_map(|(i, arg)| match arg {
//                 NestedMetaExpr::Expr(syn::Expr::Path(syn::ExprPath {
//                     path, ..
//                 })) => Some((i, path.clone())), // TODO: what about Lit?
//                 NestedMetaExpr::Meta(MetaExpr::Path(ref id)) => Some((i, id.clone())),
//                 _ => None,
//                 // This one has been checked already in get_meta_fmt() method.
//                 _ => unreachable!(),
//             })
//             .collect();
//         let (fmt_string, fmt_span) = match &list.nested[0] {
//             NestedMetaExpr::Meta(MetaExpr::NameValue(MetaNameValueExpr {
//                 path,
//                 expr:
//                     syn::Expr::Lit(syn::ExprLit {
//                         lit: syn::Lit::Str(s),
//                         ..
//                     }),
//                 ..
//             })) if path
//                 .segments
//                 .first()
//                 .expect("path shouldn't be empty")
//                 .ident
//                 == "fmt" =>
//             {
//                 (s.value(), s.span())
//             }
//             // This one has been checked already in get_meta_fmt() method.
//             _ => unreachable!(),
//         };
//
//         Placeholder::parse_fmt_string(&fmt_string).into_iter().fold(
//             HashMap::default(),
//             |mut bounds, pl| {
//                 let arg = match pl.arg {
//                     Parameter::Positional(i) => fmt_args.get(&i).cloned(),
//                     Parameter::Named(i) => Some(syn::Ident::new(&i, fmt_span).into()),
//                 };
//                 if let Some(arg) = &arg {
//                     if fields_type_params.contains_key(arg) {
//                         bounds
//                             .entry(fields_type_params[arg].clone())
//                             .or_insert_with(HashSet::default)
//                             .insert(trait_name_to_trait_bound(pl.trait_name));
//                     }
//                 }
//                 bounds
//             },
//         )
//     }
//     fn infer_type_params_bounds(
//         &self,
//         fields: &syn::Fields,
//     ) -> HashMap<syn::Type, HashSet<syn::TraitBound>> {
//         if self.type_params.is_empty() {
//             return HashMap::default();
//         }
//         if let syn::Fields::Unit = fields {
//             return HashMap::default();
//         }
//         // infer_fmt() uses only first field.
//         fields
//             .iter()
//             .take(1)
//             .filter_map(|field| {
//                 utils::get_if_type_parameter_used_in_type(&self.type_params, &field.ty)
//                     .map(|ty| {
//                         (
//                             ty,
//                             [trait_name_to_trait_bound(attribute_name_to_trait_name(
//                                 self.trait_attr,
//                             ))]
//                             .iter()
//                             .cloned()
//                             .collect(),
//                         )
//                     })
//             })
//             .collect()
//     }
// }

// ---

/// Provides the hook to expand `#[derive(Display)]` into an implementation of `From`
pub fn expand(input: &syn::DeriveInput, trait_name: &str) -> Result<TokenStream> {
    let attrs = Attributes::parse_attrs(&input.attrs, trait_name)?;
    let trait_ident = format_ident!("{trait_name}");

    // TODO: top-level attribute on enum or union.
    let ctx = (&attrs, &trait_ident, trait_name);
    let (bounds, fmt) = match &input.data {
        syn::Data::Struct(s) => expand_struct(s, ctx),
        syn::Data::Enum(e) => expand_enum(e, ctx),
        syn::Data::Union(u) => expand_union(u, ctx),
    }?;

    let ident = &input.ident;
    let (impl_gens, ty_gens, where_clause) = input.generics.split_for_impl();
    let mut where_clause = where_clause.cloned().unwrap_or_else(|| parse_quote!(where));
    where_clause.predicates.extend(bounds);

    let res = quote! {
        #[automatically_derived]
        impl #impl_gens core::fmt::#trait_ident for #ident #ty_gens
             #where_clause
        {
            fn fmt(
                &self, __derive_more_f: &mut core::fmt::Formatter<'_>
            ) -> core::fmt::Result {
                #fmt
            }
        }
    };

    Ok(res)
}

fn expand_struct(
    s: &syn::DataStruct,
    (attrs, trait_ident, trait_name): (&Attributes, &Ident, &str),
) -> Result<(Vec<syn::WherePredicate>, TokenStream)> {
    let trait_ident = format_ident!("{trait_name}");

    let s = StructOrEnumVariant {
        attrs,
        fields: &s.fields,
        trait_ident: &trait_ident,
    };
    let bounds = s.generate_bounds();
    let fmt = s.generate_fmt();

    let vars = s.fields.iter().enumerate().map(|(i, f)| {
        let var = f.ident.clone().unwrap_or_else(|| format_ident!("_{i}"));
        let member = f
            .ident
            .clone()
            .map_or_else(|| syn::Member::Unnamed(i.into()), syn::Member::Named);
        quote! { let #var = &self.#member; }
    });

    let fmt = quote! {
        #( #vars )*
        #fmt
    };

    Ok((bounds, fmt))
}

fn expand_enum(
    e: &syn::DataEnum,
    (attrs, trait_ident, trait_name): (&Attributes, &Ident, &str),
) -> Result<(Vec<syn::WherePredicate>, TokenStream)> {
    let trait_ident = format_ident!("{trait_name}");

    let (bounds, fmt) = e.variants.iter().try_fold(
        (Vec::new(), TokenStream::new()),
        |(mut bounds, mut fmt), variant| {
            let attrs = Attributes::parse_attrs(&variant.attrs, trait_name)?;

            let v = StructOrEnumVariant {
                attrs: &attrs,
                fields: &variant.fields,
                trait_ident: &trait_ident,
            };
            let fmt_inner = v.generate_fmt();
            bounds.extend(v.generate_bounds());

            let ident = &variant.ident;
            let fields_idents =
                variant.fields.iter().enumerate().map(|(i, f)| {
                    f.ident.clone().unwrap_or_else(|| format_ident!("_{i}"))
                });
            let matcher = match variant.fields {
                syn::Fields::Named(_) => {
                    quote! { Self::#ident { #( #fields_idents ),* } }
                }
                syn::Fields::Unnamed(_) => {
                    quote! { Self::#ident ( #( #fields_idents ),* ) }
                }
                syn::Fields::Unit => todo!(),
            };

            fmt.extend([quote! { #matcher => { #fmt_inner }, }]);

            Result::Ok((bounds, fmt))
        },
    )?;

    Ok((bounds, quote! { match self { #fmt } }))
}

fn expand_union(
    u: &syn::DataUnion,
    (attrs, trait_ident, trait_name): (&Attributes, &Ident, &str),
) -> Result<(Vec<syn::WherePredicate>, TokenStream)> {
    let fmt_lit = attrs.display_literal.as_ref().ok_or_else(|| {
        Error::new(
            u.fields.span(),
            format!(
                "Unions must have `#[{}(\"...\", ...)]` attribute",
                trait_name_to_attribute_name(trait_name),
            ),
        )
    })?;
    let fmt_args = &attrs.display_args;

    let fmt = quote! { ::core::write!(__derive_more_f, #fmt_lit, #( #fmt_args ),*) };

    Ok((attrs.bounds.clone().into_iter().collect(), fmt))
}

#[derive(Debug)]
struct Attributes {
    display_literal: Option<syn::LitStr>,
    display_args: Vec<FmtArgument>,
    bounds: Punctuated<syn::WherePredicate, syn::token::Comma>,
}

impl Attributes {
    fn parse_attrs(
        attrs: impl AsRef<[syn::Attribute]>,
        trait_name: &str,
    ) -> Result<Self> {
        let (display_literal, display_args, bounds) = attrs
            .as_ref()
            .iter()
            .filter(|attr| attr.path.is_ident(trait_name_to_attribute_name(trait_name)))
            .try_fold(
                (None, Vec::new(), Punctuated::new()),
                |(lit, args, mut bounds), attr| {
                    let attribute =
                        Parser::parse2(Attribute::parse, attr.tokens.clone())?;
                    Result::Ok(match attribute {
                        Attribute::Bounds(more) => {
                            bounds.extend(more);
                            (lit, args, bounds)
                        }
                        Attribute::Display {
                            display_literal,
                            display_arguments,
                        } => (
                            lit.map_or(Result::Ok(Some(display_literal)), |_| {
                                todo!("dup")
                            })?,
                            args.into_iter().chain(display_arguments).collect(),
                            bounds,
                        ),
                    })
                },
            )?;

        Ok(Self {
            display_literal,
            display_args,
            bounds,
        })
    }
}

#[derive(Debug)]
enum Attribute {
    Display {
        display_literal: syn::LitStr,
        display_arguments: Vec<FmtArgument>,
    },
    Bounds(Punctuated<syn::WherePredicate, syn::token::Comma>),
}

#[derive(Debug)]
struct FmtArgument {
    alias: Option<Ident>,
    expr: IdentOrTokenStream,
}

impl FmtArgument {
    fn is_ident(&self, ident: impl PartialEq<Ident>) -> Option<&Ident> {
        match (&self.alias, &self.expr) {
            (Some(alias), IdentOrTokenStream::Ident(i)) if ident == *alias => Some(i),
            (None, IdentOrTokenStream::Ident(i)) if ident == *i => Some(i),
            _ => None,
        }
    }
}

impl ToTokens for FmtArgument {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        if let Some(alias) = &self.alias {
            alias.to_tokens(tokens);
            syn::token::Eq::default().to_tokens(tokens);
        }
        self.expr.to_tokens(tokens);
    }
}

#[derive(Debug)]
enum IdentOrTokenStream {
    Ident(Ident),
    TokenStream(TokenStream),
}

impl IdentOrTokenStream {
    fn ident(&self) -> Option<&Ident> {
        match self {
            IdentOrTokenStream::Ident(i) => Some(i),
            IdentOrTokenStream::TokenStream(_) => None,
        }
    }
}

impl Default for IdentOrTokenStream {
    fn default() -> Self {
        Self::TokenStream(TokenStream::new())
    }
}

impl ToTokens for IdentOrTokenStream {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::Ident(ident) => ident.to_tokens(tokens),
            Self::TokenStream(ts) => ts.to_tokens(tokens),
        }
    }
}

impl IdentOrTokenStream {
    fn new() -> Self {
        Self::default()
    }

    fn extend_tt(&mut self, stream: proc_macro2::TokenTree) -> &mut Self {
        let this = mem::take(self);
        *self = Self::TokenStream(match this {
            Self::Ident(ident) => {
                let mut ident = ident.into_token_stream();
                ident.extend([stream]);
                ident
            }
            Self::TokenStream(mut old) => {
                old.extend([stream]);
                old
            }
        });
        self
    }

    fn extend_ts(&mut self, stream: proc_macro2::TokenStream) -> &mut Self {
        let this = mem::take(self);
        *self = Self::TokenStream(match this {
            Self::Ident(ident) => {
                let mut ident = ident.into_token_stream();
                ident.extend([stream]);
                ident
            }
            Self::TokenStream(mut old) => {
                old.extend([stream]);
                old
            }
        });
        self
    }

    fn push_ident(mut self, ident: proc_macro2::Ident) -> Self {
        self.extend_tt(ident.into());
        self
    }
}

impl Parse for Attribute {
    fn parse(input: ParseStream) -> Result<Self> {
        use proc_macro2::{Delimiter, TokenTree};
        use syn::buffer::Cursor;

        const DELIMS: [Delimiter; 4] = {
            // Exhaustive match.
            match Delimiter::Parenthesis {
                Delimiter::Parenthesis => {}
                Delimiter::Brace => {}
                Delimiter::Bracket => {}
                Delimiter::None => {}
            }

            [
                Delimiter::Parenthesis,
                Delimiter::Brace,
                Delimiter::Bracket,
                Delimiter::None,
            ]
        };

        const PAIRED_PUNCTS: [(char, char); 2] = [('<', '>'), ('|', '|')];

        fn paired_punct(p: char) -> Option<char> {
            PAIRED_PUNCTS
                .iter()
                .find_map(|(l, r)| (*l == p).then_some(*r))
        }

        fn parse_until_paired_punct(
            punct: char,
            mut cursor: Cursor<'_>,
        ) -> Option<(TokenStream, Cursor<'_>)> {
            let mut stream = TokenStream::new();

            while let Some((tt, c)) = cursor.token_tree() {
                match tt {
                    TokenTree::Punct(p) if p.as_char() == punct => {
                        stream.extend([TokenTree::Punct(p)]);
                        return Some((stream, c));
                    }
                    TokenTree::Punct(p) if paired_punct(p.as_char()).is_some() => {
                        let (more, c) = paired_punct(p.as_char())
                            .and_then(|p| parse_until_paired_punct(p, c))?;
                        stream.extend([TokenTree::Punct(p)]);
                        stream.extend([more]);
                        cursor = c;
                    }
                    tt => stream.extend([tt]),
                }
            }

            None
        }

        let content;
        syn::parenthesized!(content in input);

        if content.peek(syn::LitStr) {
            let display_literal = content.parse()?;

            if content.peek(syn::token::Comma) {
                let _ = content.parse::<syn::token::Comma>()?;
            }

            let display_arguments = content.step(|cursor| {
                let mut arguments = Vec::new();

                let mut rest = *cursor;
                while !rest.eof() {
                    let mut expr = None;
                    let mut alias = None;

                    if let Some((ident, c)) = rest.ident() {
                        if let Some((_eq, c)) =
                            rest.punct().filter(|(p, _)| p.as_char() == '=')
                        {
                            // arg.get_or_insert_with(IdentOrTokenStream::new)
                            //     .extend_tt(ident.clone().into())
                            //     .extend_tt(eq.into());
                            alias = Some(ident);
                            rest = c;
                        }
                    }

                    if let Some((gr @ TokenTree::Group(_), c)) = rest.token_tree() {
                        expr.get_or_insert_with(IdentOrTokenStream::new)
                            .extend_tt(gr);
                        rest = c;
                    }

                    if let Some((ident, c)) = rest.ident() {
                        if c.eof()
                            || c.punct().filter(|(p, _)| p.as_char() == ',').is_some()
                        {
                            expr = Some(match expr.take() {
                                None => IdentOrTokenStream::Ident(ident),
                                Some(s) => s.push_ident(ident),
                            });
                            rest = c;
                        }
                    }

                    while let Some((tt, c)) = rest.token_tree() {
                        rest = c;

                        match tt {
                            TokenTree::Punct(p) if p.as_char() == ',' => {
                                break;
                            }
                            TokenTree::Punct(p)
                                if paired_punct(p.as_char()).is_some() =>
                            {
                                let (more, c) = paired_punct(p.as_char())
                                    .and_then(|p| parse_until_paired_punct(p, c))
                                    .ok_or_else::<syn::Error, _>(|| todo!("err"))?;
                                rest = c;
                                expr.get_or_insert_with(IdentOrTokenStream::new)
                                    .extend_ts(more);
                            }
                            tt => {
                                expr.get_or_insert_with(IdentOrTokenStream::new)
                                    .extend_tt(tt);
                            }
                        }
                    }

                    if let Some(expr) = expr {
                        arguments.push(FmtArgument { alias, expr });
                    }
                }

                Ok((arguments, rest))
            })?;

            return Ok(Attribute::Display {
                display_literal,
                display_arguments,
            });
        }

        let _ = content.parse::<syn::Path>().and_then(|p| {
            p.is_ident("bound")
                .then_some(Ok(p))
                .unwrap_or_else(|| todo!("error message"))
        })?;

        let inner;
        syn::parenthesized!(inner in content);

        inner
            .parse_terminated(syn::WherePredicate::parse)
            .map(Attribute::Bounds)
    }
}

// TODO: check if attr.display_literal.is_some() ||
//                fields.iter().next().is_some()
#[derive(Debug)]
struct StructOrEnumVariant<'a> {
    attrs: &'a Attributes,
    fields: &'a syn::Fields,
    trait_ident: &'a Ident,
}

impl<'a> StructOrEnumVariant<'a> {
    fn generate_fmt(&self) -> TokenStream {
        if let Some(lit) = &self.attrs.display_literal {
            let args = &self.attrs.display_args;
            quote! { ::core::write!(__derive_more_f, #lit, #( #args ),*) }
        } else if self.fields.iter().count() == 1 {
            let field = self
                .fields
                .iter()
                .next()
                .unwrap_or_else(|| unreachable!("count() == 1"));
            let ident = field.ident.clone().unwrap_or_else(|| format_ident!("_0"));
            let trait_ident = self.trait_ident;

            quote! { ::core::fmt::#trait_ident::fmt(#ident, __derive_more_f) }
        } else {
            todo!("err")
        }
    }

    fn generate_bounds(&self) -> Vec<syn::WherePredicate> {
        let Some(display_literal) = &self.attrs.display_literal else {
            return self.fields.iter().next().map(|f| {
                let ty = &f.ty;
                vec![parse_quote! { #ty: ::core::fmt::Display }]
            })
            .unwrap_or_default();
        };

        let placeholders = Placeholder::parse_fmt_string(&display_literal.value());

        // We ignore unknown fields, as compiler will produce better error
        // messages.
        placeholders
            .into_iter()
            .filter_map(|placeholder| {
                let name = match placeholder.arg {
                    Parameter::Named(name) => self
                        .attrs
                        .display_args
                        .iter()
                        .find_map(|a| (a.alias.as_ref()? == &name).then_some(&a.expr))
                        .map_or(Some(name), |expr| {
                            expr.ident().map(ToString::to_string)
                        })?,
                    Parameter::Positional(i) => self
                        .attrs
                        .display_args
                        .iter()
                        .nth(i)
                        .and_then(|a| a.expr.ident().filter(|_| a.alias.is_none()))?
                        .to_string(),
                };

                let unnamed = name.strip_prefix("_").and_then(|s| s.parse().ok());
                let ty = match (&self.fields, unnamed) {
                    (syn::Fields::Unnamed(f), Some(i)) => {
                        f.unnamed.iter().nth(i).map(|f| &f.ty)
                    }
                    (syn::Fields::Named(f), None) => f.named.iter().find_map(|f| {
                        f.ident.as_ref().filter(|s| **s == name).map(|_| &f.ty)
                    }),
                    _ => None,
                }?;

                let tr = format_ident!("{}", placeholder.trait_name);
                Some(parse_quote! { #ty: ::core::fmt::#tr })
            })
            .chain(self.attrs.bounds.clone())
            .collect()
    }
}

// ---

// enum MetaExpr {
//     Path(syn::Path),
//     List(MetaListExpr),
//     NameValue(MetaNameValueExpr),
// }
//
// impl MetaExpr {
//     fn from_attr(attr: &syn::Attribute) -> Result<Self> {
//         use syn::punctuated::Pair;
//
//         fn clone_ident_segment(segment: &syn::PathSegment) -> syn::PathSegment {
//             syn::PathSegment {
//                 ident: segment.ident.clone(),
//                 arguments: syn::PathArguments::None,
//             }
//         }
//
//         let path = syn::Path {
//             leading_colon: attr
//                 .path
//                 .leading_colon
//                 .as_ref()
//                 .map(|colon| syn::Token![::](colon.spans)),
//             segments: attr
//                 .path
//                 .segments
//                 .pairs()
//                 .map(|pair| match pair {
//                     Pair::Punctuated(seg, punct) => Pair::Punctuated(
//                         clone_ident_segment(seg),
//                         syn::Token![::](punct.spans),
//                     ),
//                     Pair::End(seg) => Pair::End(clone_ident_segment(seg)),
//                 })
//                 .collect(),
//         };
//
//         let parser = |input: ParseStream| parse_meta_after_path(path, input);
//         Parser::parse2(parser, attr.tokens.clone())
//     }
// }
//
// impl Parse for MetaExpr {
//     fn parse(input: ParseStream) -> Result<Self> {
//         let path = input.call(parse_meta_path)?;
//
//         if input.peek(syn::token::Paren) {
//             parse_meta_list_after_path(path, input).map(MetaExpr::List)
//         } else if input.peek(syn::Token![=]) {
//             parse_meta_name_value_after_path(path, input).map(MetaExpr::NameValue)
//         } else {
//             Ok(MetaExpr::Path(path))
//         }
//     }
// }
//
// impl ToTokens for MetaExpr {
//     fn to_tokens(&self, tokens: &mut TokenStream) {
//         match self {
//             Self::Path(v) => v.to_tokens(tokens),
//             Self::List(v) => v.to_tokens(tokens),
//             Self::NameValue(v) => v.to_tokens(tokens),
//         }
//     }
// }
//
// struct MetaListExpr {
//     path: syn::Path,
//     paren_token: syn::token::Paren,
//     nested: Punctuated<NestedMetaExpr, syn::token::Comma>,
// }
//
// impl Parse for MetaListExpr {
//     fn parse(input: ParseStream) -> Result<Self> {
//         let path = input.call(parse_meta_path)?;
//         parse_meta_list_after_path(path, input)
//     }
// }
//
// impl ToTokens for MetaListExpr {
//     fn to_tokens(&self, tokens: &mut TokenStream) {
//         self.path.to_tokens(tokens);
//         self.paren_token
//             .surround(tokens, |tokens| self.nested.to_tokens(tokens))
//     }
// }
//
// enum NestedMetaExpr {
//     Meta(MetaExpr),
//     Expr(syn::Expr),
// }
//
// impl Parse for NestedMetaExpr {
//     fn parse(input: ParseStream) -> Result<Self> {
//         let fork = input.fork();
//
//         if let Ok(path) = fork.call(parse_meta_path) {
//             if fork.is_empty() || fork.peek(syn::Token![,]) {
//                 input.advance_to(&fork);
//             }
//         }
//
//         if input.peek(Ident::peek_any)
//             || input.peek(syn::Token![::]) && input.peek3(Ident::peek_any)
//         {
//             input.parse().map(NestedMetaExpr::Meta)
//         } else {
//             input.parse().map(NestedMetaExpr::Expr)
//         }
//     }
// }
//
// impl ToTokens for NestedMetaExpr {
//     fn to_tokens(&self, tokens: &mut TokenStream) {
//         match self {
//             Self::Meta(v) => v.to_tokens(tokens),
//             Self::Expr(v) => v.to_tokens(tokens),
//         }
//     }
// }
//
// struct MetaNameValueExpr {
//     path: syn::Path,
//     eq_token: syn::Token![=],
//     expr: syn::Expr,
// }
//
// impl Parse for MetaNameValueExpr {
//     fn parse(input: ParseStream) -> Result<Self> {
//         let path = input.call(parse_meta_path)?;
//         parse_meta_name_value_after_path(path, input)
//     }
// }
//
// impl ToTokens for MetaNameValueExpr {
//     fn to_tokens(&self, tokens: &mut TokenStream) {
//         self.path.to_tokens(tokens);
//         self.eq_token.to_tokens(tokens);
//         self.expr.to_tokens(tokens);
//     }
// }
//
// fn parse_meta_after_path(path: syn::Path, input: ParseStream) -> Result<MetaExpr> {
//     if input.peek(syn::token::Paren) {
//         parse_meta_list_after_path(path, input).map(MetaExpr::List)
//     } else if input.peek(syn::Token![=]) {
//         parse_meta_name_value_after_path(path, input).map(MetaExpr::NameValue)
//     } else {
//         Ok(MetaExpr::Path(path))
//     }
// }
//
// fn parse_meta_list_after_path(
//     path: syn::Path,
//     input: ParseStream,
// ) -> Result<MetaListExpr> {
//     let content;
//
//     syn::__private::parse_parens;
//
//     Ok(MetaListExpr {
//         path,
//         paren_token: syn::parenthesized!(content in input),
//         nested: content.parse_terminated(NestedMetaExpr::parse)?,
//     })
// }
//
// fn parse_meta_name_value_after_path(
//     path: syn::Path,
//     input: ParseStream,
// ) -> Result<MetaNameValueExpr> {
//     Ok(MetaNameValueExpr {
//         path,
//         eq_token: input.parse()?,
//         expr: input.parse()?,
//     })
// }
//
// /// Like [`syn::Path::parse_mod_style()`] but accepts keywords in the path.
// fn parse_meta_path(input: ParseStream) -> Result<syn::Path> {
//     Ok(syn::Path {
//         leading_colon: input.parse()?,
//         segments: {
//             let mut segments = Punctuated::new();
//             while input.peek(Ident::peek_any) {
//                 let ident = Ident::parse_any(input)?;
//                 segments.push_value(syn::PathSegment::from(ident));
//                 if !input.peek(syn::Token![::]) {
//                     break;
//                 }
//                 let punct = input.parse()?;
//                 segments.push_punct(punct);
//             }
//             if segments.is_empty() {
//                 return Err(input.error("expected path"));
//             } else if segments.trailing_punct() {
//                 return Err(input.error("expected path segment"));
//             }
//             segments
//         },
//     })
// }
//
// --

/// [Parameter][1] used in [`Placeholder`].
///
/// [1]: https://doc.rust-lang.org/stable/std/fmt/index.html#formatting-parameters
#[derive(Debug, Eq, PartialEq)]
enum Parameter {
    /// [Positional parameter][1].
    ///
    /// [1]: https://doc.rust-lang.org/stable/std/fmt/index.html#positional-parameters
    Positional(usize),

    /// [Named parameter][1].
    ///
    /// [1]: https://doc.rust-lang.org/stable/std/fmt/index.html#named-parameters
    Named(String),
}

impl<'a> From<parsing::Argument<'a>> for Parameter {
    fn from(arg: parsing::Argument<'a>) -> Self {
        match arg {
            parsing::Argument::Integer(i) => Parameter::Positional(i),
            parsing::Argument::Identifier(i) => Parameter::Named(i.to_owned()),
        }
    }
}

/// Representation of formatting placeholder.
#[derive(Debug, PartialEq, Eq)]
struct Placeholder {
    /// Formatting argument (either named or positional) to be used by this placeholder.
    arg: Parameter,

    /// [Width parameter][1], if present.
    ///
    /// [1]: https://doc.rust-lang.org/stable/std/fmt/index.html#width
    width: Option<Parameter>,

    /// [Precision parameter][1], if present.
    ///
    /// [1]: https://doc.rust-lang.org/stable/std/fmt/index.html#precision
    precision: Option<Parameter>,

    /// Name of [`std::fmt`] trait to be used for rendering this placeholder.
    trait_name: &'static str,
}

impl Placeholder {
    /// Parses [`Placeholder`]s from a given formatting string.
    fn parse_fmt_string(s: &str) -> Vec<Self> {
        let mut n = 0;
        parsing::format_string(s)
            .into_iter()
            .flat_map(|f| f.formats)
            .map(|format| {
                let (maybe_arg, ty) = (
                    format.arg,
                    format.spec.map(|s| s.ty).unwrap_or(parsing::Type::Display),
                );
                let position = maybe_arg.map(Into::into).unwrap_or_else(|| {
                    // Assign "the next argument".
                    // https://doc.rust-lang.org/stable/std/fmt/index.html#positional-parameters
                    n += 1;
                    Parameter::Positional(n - 1)
                });

                Self {
                    arg: position,
                    width: format.spec.and_then(|s| match s.width {
                        Some(parsing::Count::Parameter(arg)) => Some(arg.into()),
                        _ => None,
                    }),
                    precision: format.spec.and_then(|s| match s.precision {
                        Some(parsing::Precision::Count(parsing::Count::Parameter(
                            arg,
                        ))) => Some(arg.into()),
                        _ => None,
                    }),
                    trait_name: ty.trait_name(),
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod placeholder_parse_fmt_string_spec {
    use super::*;

    #[test]
    fn indicates_position_and_trait_name_for_each_fmt_placeholder() {
        let fmt_string = "{},{:?},{{}},{{{1:0$}}}-{2:.1$x}{par:#?}{:width$}";
        assert_eq!(
            Placeholder::parse_fmt_string(&fmt_string),
            vec![
                Placeholder {
                    arg: Parameter::Positional(0),
                    width: None,
                    precision: None,
                    trait_name: "Display",
                },
                Placeholder {
                    arg: Parameter::Positional(1),
                    width: None,
                    precision: None,
                    trait_name: "Debug",
                },
                Placeholder {
                    arg: Parameter::Positional(1),
                    width: Some(Parameter::Positional(0)),
                    precision: None,
                    trait_name: "Display",
                },
                Placeholder {
                    arg: Parameter::Positional(2),
                    width: None,
                    precision: Some(Parameter::Positional(1)),
                    trait_name: "LowerHex",
                },
                Placeholder {
                    arg: Parameter::Named("par".to_owned()),
                    width: None,
                    precision: None,
                    trait_name: "Debug",
                },
                Placeholder {
                    arg: Parameter::Positional(2),
                    width: Some(Parameter::Named("width".to_owned())),
                    precision: None,
                    trait_name: "Display",
                },
            ],
        );
    }
}
