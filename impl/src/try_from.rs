//! Implementation of a [`TryFrom`] derive macro.

use proc_macro2::{Literal, TokenStream};
use quote::{format_ident, quote, ToTokens};
use syn::{
    parse::{Parse, ParseStream},
    spanned::Spanned as _,
    Token,
};

use crate::utils::{
    attr::{self, ParseMultiple},
    Spanning,
};

/// Expands a [`TryFrom`] derive macro.
pub fn expand(input: &syn::DeriveInput, _: &'static str) -> syn::Result<TokenStream> {
    Ok(Expansion::expand(input)?.into_token_stream())
}

/// Optional Repr detail for conversion to a repr.
///
/// Parsed with a single attr, provides a default int repr and if parsed with multiple attrs,
/// requires the `repr` explicitly.
struct ReprArgument {
    /// Repr type to convert to.
    repr: attr::ReprInt,
    /// Repr argument given to the attribute.
    conversion: attr::ReprConversion,
}

impl Parse for ReprArgument {
    /// Assuming default `#[repr]`, this parses the arguments of `#[try_from]` accepting `repr` only.
    fn parse(input: ParseStream) -> syn::Result<Self> {
        match input.parse()? {
            attr::ReprConversion::Types(_) => Err(syn::Error::new(
                input.span(),
                "`#[{name}(repr(...))]` attribute is not supported yet",
            )),
            conversion => Ok(Self {
                conversion,
                repr: Default::default(),
            }),
        }
    }
}

impl ParseMultiple for ReprArgument {
    fn parse_attrs(
        attrs: impl AsRef<[syn::Attribute]>,
        name: &syn::Ident,
    ) -> syn::Result<Option<Spanning<Self>>> {
        let Some(mut conv) = Self::parse_attrs_with(&attrs, name, &())? else {
            return Ok(None);
        };

        // if a repr is given explicitly replace the values
        if let Some(repr) = attr::ReprInt::parse_attrs(&attrs, &format_ident!("repr"))?
        {
            conv.repr = repr.item;
            conv.span = conv.span.join(repr.span).unwrap_or(conv.span);
        }

        Ok(Some(conv))
    }

    /// Use each field's `merge_attrs` for more specific error messages.
    fn merge_attrs(
        mut prev: Spanning<Self>,
        new: Spanning<Self>,
        name: &syn::Ident,
    ) -> syn::Result<Spanning<Self>> {
        let prev_span = prev.span();
        let new_span = new.span();

        prev.item.conversion = attr::ReprConversion::merge_attrs(
            Spanning::new(prev.item.conversion, prev_span),
            Spanning::new(new.item.conversion, new_span),
            name,
        )?
        .item;

        prev.item.repr = attr::ReprInt::merge_attrs(
            Spanning::new(prev.item.repr, prev_span),
            Spanning::new(new.item.repr, new_span),
            name,
        )?
        .item;

        prev.span = prev.span.join(new.span).unwrap_or(prev.span);

        Ok(prev)
    }
}

/// Inputs given to the macro when converting to a non-repr.
struct TypeArgument {
    /// What will be the result of the conversion (`T` in `TryFrom<T>`).
    from_type: syn::Type,
    /// The type of the error returned (`TryFrom::<T>::Error`).
    err_type: syn::Type,
    /// The expression which creates the error. If not given, [`Self::err_type`] is used.
    err_constructor: Option<syn::Expr>,
}

impl TypeArgument {
    /// Return a valid error constructor ready to be wrapped in an `Err`.
    fn err_constructor(&self) -> TokenStream {
        self.err_constructor
            .as_ref()
            .map(ToTokens::to_token_stream)
            .unwrap_or_else(|| self.err_type.to_token_stream())
    }
}

impl Parse for TypeArgument {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let from_type = input.parse()?;

        let err_type = if input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
            input.parse()?
        } else {
            // "()"
            syn::Type::Tuple(syn::TypeTuple {
                elems: Default::default(),
                paren_token: Default::default(),
            })
        };

        let err_constructor = if input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
            Some(input.parse()?)
        } else {
            None
        };

        Ok(Self {
            from_type,
            err_type,
            err_constructor,
        })
    }
}

struct Targets {
    /// Implement `TryFrom<T>` when a `#[repr(T)]` is given and input is `repr`.
    ///
    /// Can be only one for each enum.
    repr: Option<ReprArgument>,
    /// Implement `TryFrom<T>` when a `T` is directly given via the input for every call.
    ///
    /// This is validated to be uniquely determinable.  See `Self::are_fields_unqiue`.
    types: Vec<TypeArgument>,
}

impl Parse for Targets {
    /// Parse a single `#[try_from]` either repr or type.
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.cursor().ident().is_some_and(|(i, _)| i == "repr") {
            return Ok(Self {
                repr: Some(input.parse()?),
                types: Default::default(),
            });
        }

        Ok(Self {
            types: vec![input.parse()?],
            repr: Default::default(),
        })
    }
}

impl ParseMultiple for Targets {
    // Try to parse normally (parse only try_from using the default parser) and add the related
    // information from other attrs if relevatent.
    fn parse_attrs(
        attrs: impl AsRef<[syn::Attribute]>,
        name: &syn::Ident,
    ) -> syn::Result<Option<Spanning<Self>>> {
        let mut candidate = Self::parse_attrs_with(&attrs, name, &())?;

        if let Some(ref mut target) = candidate {
            if let Some(ref mut repr_arg) = target.repr {
                // try to add the repr type to the repr arg if possible since parse ignores it.
                // This is basically a shared behavior with the default parser of the `ReprArgument`
                // type
                if let Some(repr) =
                    attr::ReprInt::parse_attrs(&attrs, &format_ident!("repr"))?
                {
                    repr_arg.repr = repr.item;
                    target.span = target.span.join(repr.span).unwrap_or(target.span);
                }
            }
        }

        Ok(candidate)
    }

    fn merge_attrs(
        mut prev: Spanning<Self>,
        mut new: Spanning<Self>,
        name: &syn::Ident,
    ) -> syn::Result<Spanning<Self>> {
        let prev_span = prev.span();
        let new_span = new.span();

        prev.item.repr = match (prev.item.repr, new.item.repr) {
            (Some(p), Some(n)) => Some(
                ReprArgument::merge_attrs(
                    Spanning::new(p, prev_span),
                    Spanning::new(n, new_span),
                    name,
                )?
                .item,
            ),
            (Some(v), None) | (None, Some(v)) => Some(v),
            (None, None) => None,
        };

        prev.item.types.append(&mut new.item.types);

        Ok(prev)
    }
}

/// Expansion of a macro for generating [`TryFrom`] implementation of an enum.
struct Expansion {
    /// The `TryFrom<T>` implementations.
    targets: Option<Targets>,

    /// [`syn::Ident`] of the enum.
    ///
    /// [`syn::Ident`]: struct@syn::Ident
    ident: syn::Ident,

    /// [`syn::Generics`] of the enum.
    generics: syn::Generics,

    /// [`syn::Variant`]s of the enum.
    variants: Vec<syn::Variant>,
}

impl Expansion {
    pub fn expand(input: &syn::DeriveInput) -> syn::Result<TokenStream> {
        match &input.data {
            syn::Data::Struct(data) => Err(syn::Error::new(
                data.struct_token.span(),
                "`TryFrom` cannot be derived for structs",
            )),
            syn::Data::Enum(data) => {
                let targets =
                    Targets::parse_attrs(&input.attrs, &format_ident!("try_from"))?
                        .map(Spanning::into_inner);

                let variants = data.variants.clone().into_iter().collect::<Vec<_>>();

                // When types are requested, the enum cannot have duplicate field types
                if let Some(Targets { types, .. }) = &targets {
                    if !types.is_empty() {
                        // TODO collect errors and return for all units
                        for var in variants.iter() {
                            if matches!(var.fields, syn::Fields::Unit) {
                                return Err(
                                    syn::Error::new(
                                        var.span(),
                                        "empty variant not supported when using non-repr `try_from`"
                                    )
                                );
                            }
                        }

                        Self::are_fields_unique(&variants)?;
                    }
                }

                Ok(Expansion {
                    targets,
                    ident: input.ident.clone(),
                    generics: input.generics.clone(),
                    variants,
                }
                .into_token_stream())
            }
            syn::Data::Union(data) => Err(syn::Error::new(
                data.union_token.span(),
                "`TryFrom` cannot be derived for unions",
            )),
        }
    }

    /// Try to convert every field of this variant in preserved order and return `Ok`.
    ///
    /// Generates a `if let Ok(v) = V::try_from(field1) { if ... { ... { return Ok(v); } }}`.
    ///
    /// Assumes the fields are already unique in any other case, generates error-prone code.
    fn try_from_variant(
        &self,
        syn::Variant {
            ident: var_ident,
            fields,
            ..
        }: &syn::Variant,
    ) -> TokenStream {
        let enum_ident = &self.ident;

        let ok = quote! { derive_more::core::result::Result::Ok };

        let bindings = (0..fields.len()).map(|i| format_ident!("__binding_{i}"));

        let bindings_types = bindings.clone().zip(match fields {
            syn::Fields::Named(syn::FieldsNamed { named: fields, .. })
            | syn::Fields::Unnamed(syn::FieldsUnnamed {
                unnamed: fields, ..
            }) => fields.into_iter().map(|i| &i.ty),
            syn::Fields::Unit => unreachable!("units are already filtered out"),
        });

        let nested_result = match fields {
            syn::Fields::Unnamed(_) => quote! {
                return #ok(#enum_ident::#var_ident(#(#bindings,)*));
            },
            syn::Fields::Named(syn::FieldsNamed { named, .. }) => {
                let names = named.into_iter().map(|i| i.ident.as_ref().unwrap());
                quote! { return #ok(#enum_ident::#var_ident { #(#names: #bindings,)* }); }
            }
            syn::Fields::Unit => unreachable!("units are already filtered out"),
        };

        bindings_types
            .rev()
            .fold(nested_result, |tokens, (binding, ty)| {
                quote! {
                    if let #ok(#binding) =
                        #ty::try_from(value) {
                        #tokens
                    }
                }
            })
    }

    /// Generate the `impl TryFrom<T> for Ident where C: Criterion` for a given type.
    fn generate_tokens(
        &self,
        from_type: &TokenStream,
        err_type: &TokenStream,
        body: &TokenStream,
    ) -> TokenStream {
        let (impl_generics, ty_generics, where_clause) = self.generics.split_for_impl();
        let ident = &self.ident;

        quote! {
            #[automatically_derived]
            impl #impl_generics derive_more::core::convert::TryFrom<#from_type #ty_generics>
             for #ident #where_clause {
                type Error = #err_type;

                #[allow(non_upper_case_globals)]
                #[inline]
                fn try_from(value: #from_type) ->
                     derive_more::core::result::Result<Self, #err_type> {
                    #body
                }
            }
        }
    }

    /// If the try_from is done using a repr, expand the code.
    fn repr_to_tokens(&self, tokens: &mut TokenStream) {
        let Some(Targets {
            repr: Some(arg), ..
        }) = &self.targets
        else {
            return;
        };
        let repr_ty = arg.repr.ty();

        let mut last_discriminant = quote! { 0 };
        let mut inc = 0usize;
        let (consts, (discriminants, variants)): (
            Vec<syn::Ident>,
            (Vec<TokenStream>, Vec<TokenStream>),
        ) = self
            .variants
            .iter()
            .filter_map(
                |syn::Variant {
                     ident,
                     fields,
                     discriminant,
                     ..
                 }| {
                    if let Some(d) = discriminant {
                        last_discriminant = d.1.to_token_stream();
                        inc = 0;
                    }
                    let ret = {
                        let inc = Literal::usize_unsuffixed(inc);
                        fields.is_empty().then_some((
                            format_ident!("__DISCRIMINANT_{ident}"),
                            (
                                quote! { #last_discriminant + #inc },
                                quote! { #ident #fields },
                            ),
                        ))
                    };
                    inc += 1;
                    ret
                },
            )
            .unzip();

        let error = quote! { derive_more::TryFromReprError<#repr_ty> };
        let ident = &self.ident;
        let body = quote! {
            #( const #consts: #repr_ty = #discriminants; )*
            match value {
                #(#consts => derive_more::core::result::Result::Ok(#ident::#variants),)*
                _ => derive_more::core::result::Result::Err(
                    derive_more::TryFromReprError::new(value)
                ),
            }
        };

        self.generate_tokens(&repr_ty.to_token_stream(), &error, &body)
            .to_tokens(tokens);
    }

    /// If the try_from is done using a type other than reprs, expand the code.
    fn type_to_tokens(&self, tokens: &mut TokenStream) {
        let Some(Targets { types, .. }) = &self.targets else {
            return;
        };
        if types.is_empty() {
            return;
        }

        // since the function is a trait/generic function a single body will do for every type in
        // targets.
        let body =
            self.variants
                .iter()
                .fold(TokenStream::default(), |mut tokens, i| {
                    tokens.extend(self.try_from_variant(i));
                    tokens
                });

        types
            .iter()
            .fold(TokenStream::default(), |mut tokens, args| {
                let default_return = args.err_constructor();
                tokens.extend(self.generate_tokens(
                    &args.from_type.to_token_stream(),
                    &args.err_type.to_token_stream(),
                    &quote! {
                        #body
                        derive_more::core::result::Result::Err(#default_return)
                    },
                ));
                tokens
            })
            .to_tokens(tokens);
    }

    /// Return variant types in the order of declaration if all are unique else throws syn error.
    ///
    /// Checks for being unique in the order of declaration means the function checks if:
    /// - order is unique meaning `E::T1(u32, u16)` is not the same as `E::T2(u16, u32)`.
    /// - unit (empty variants) is, at most, provided only once (see [`syn::Fields::Unit`]).
    /// - the type of fields in each variant (regardless of its type) is the same or not.  In other
    ///   words, of [`syn::Fields`] types are checked, meaning `E::T1(u32, u16)` is the same as
    ///   `E::T2 { t1: u32, t2: u16 }`.
    //
    // "The complexity" factor is not taken into account since rarely enums have more than a few
    // variants. So the most crude way to implement it yet, but it is preferred for simplicity and not
    // using no_std.
    //
    // This could use a sorted approach (for example by doing ty.to_token_stream().to_string() for all),
    // that is not used again for the same reason
    fn are_fields_unique(variants: &[syn::Variant]) -> syn::Result<()> {
        let mut unique_types = Vec::new();

        for variant in variants.iter() {
            // since the comparison function is string based and the ident field changes the
            // representation, this conversion to all "unnamed" is required.
            let types = match &variant.fields {
                syn::Fields::Named(syn::FieldsNamed { named: fields, .. })
                | syn::Fields::Unnamed(syn::FieldsUnnamed {
                    unnamed: fields, ..
                }) => fields
                    .into_iter()
                    .map(|i| {
                        let mut i = i.to_owned();
                        i.ident = None;
                        i.colon_token = None;
                        i.to_token_stream().to_string() + ", "
                    })
                    .collect(),
                syn::Fields::Unit => String::new(),
            };

            match unique_types.iter().position(|i| i == &types) {
                Some(dup_i) => {
                    return Err(syn::Error::new(
                        variant.fields.span(),
                        format!(
                            "`{}` types collection is used for more than one variant (`{}`, `{}`), \
                             non-repr `try_from` cannot be implemented (try changing the order of \
                             fields as a workaround)",
                            &types[..types.len().saturating_sub(2)], // remove trailing ", "
                            variants[dup_i].ident,
                            variant.ident,
                        ),
                    ));
                }
                None => unique_types.push(types),
            };
        }

        Ok(())
    }
}

impl ToTokens for Expansion {
    /// Expands [`TryFrom`] implementations for a struct.
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.repr_to_tokens(tokens);
        self.type_to_tokens(tokens);
    }
}
