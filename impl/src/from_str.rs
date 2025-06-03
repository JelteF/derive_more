//! Implementation of a [`FromStr`] derive macro.

#[cfg(doc)]
use std::str::FromStr;
use std::{cmp, collections::HashMap, iter};

use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};
use syn::{parse::Parse, parse_quote, spanned::Spanned as _};

use crate::utils::{
    attr::{self, ParseMultiple},
    Either, GenericsSearch, Spanning,
};

mod keyword {
    use syn::custom_keyword;

    custom_keyword!(from_str);
}

/// Expands a [`FromStr`] derive macro.
pub fn expand(input: &syn::DeriveInput, _: &'static str) -> syn::Result<TokenStream> {
    match &input.data {
        syn::Data::Struct(data) => Ok(if data.fields.is_empty() {
            FlatExpansion::try_from(input)?.into_token_stream()
        } else {
            ForwardExpansion::try_from(input)?.into_token_stream()
        }),
        syn::Data::Enum(_) => Ok(FlatExpansion::try_from(input)?.into_token_stream()),
        syn::Data::Union(data) => Err(syn::Error::new(
            data.union_token.span(),
            "`FromStr` cannot be derived for unions",
        )),
    }
}

/// Expansion of a macro for generating a forwarding [`FromStr`] implementation of a struct.
struct ForwardExpansion<'i> {
    /// [`syn::Ident`] and [`syn::Generics`] of the struct.
    ///
    /// [`syn::Ident`]: struct@syn::Ident
    self_ty: (&'i syn::Ident, &'i syn::Generics),

    /// [`syn::Field`] representing the wrapped type to forward implementation on.
    inner: &'i syn::Field,
}

impl<'i> TryFrom<&'i syn::DeriveInput> for ForwardExpansion<'i> {
    type Error = syn::Error;

    fn try_from(input: &'i syn::DeriveInput) -> syn::Result<Self> {
        let syn::Data::Struct(data) = &input.data else {
            return Err(syn::Error::new(
                input.span(),
                "expected a struct for forward `FromStr` derive",
            ));
        };
        if let Some(attr) = input
            .attrs
            .iter()
            .find(|attr| attr.path().is_ident("from_str"))
        {
            return Err(syn::Error::new(
                attr.path().span(),
                "no attribute is allowed here",
            ));
        }

        // TODO: Unite these two conditions via `&&` once MSRV is bumped to 1.88 or above.
        if data.fields.len() != 1 {
            return Err(syn::Error::new(
                data.fields.span(),
                "only structs with single field can derive `FromStr`",
            ));
        }
        let Some(inner) = data.fields.iter().next() else {
            return Err(syn::Error::new(
                data.fields.span(),
                "only structs with single field can derive `FromStr`",
            ));
        };

        Ok(Self {
            self_ty: (&input.ident, &input.generics),
            inner,
        })
    }
}

impl ToTokens for ForwardExpansion<'_> {
    /// Expands a forwarding [`FromStr`] implementations for a struct.
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let inner_ty = &self.inner.ty;
        let ty = self.self_ty.0;

        let generics_search = GenericsSearch::from(self.self_ty.1);
        let mut generics = self.self_ty.1.clone();
        if generics_search.any_in(inner_ty) {
            generics.make_where_clause().predicates.push(parse_quote! {
                #inner_ty: derive_more::core::str::FromStr
            });
        }
        let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

        let constructor = self.inner.self_constructor([parse_quote! { v }]);

        quote! {
            #[automatically_derived]
            impl #impl_generics derive_more::core::str::FromStr for #ty #ty_generics #where_clause {
                type Err = <#inner_ty as derive_more::core::str::FromStr>::Err;

                #[inline]
                fn from_str(s: &str) -> derive_more::core::result::Result<Self, Self::Err> {
                    derive_more::core::str::FromStr::from_str(s).map(|v| #constructor)
                }
            }
        }.to_tokens(tokens);
    }
}

/// Expansion of a macro for generating a flat [`FromStr`] implementation of an enum or a struct.
struct FlatExpansion<'i> {
    /// [`syn::Ident`] and [`syn::Generics`] of the enum/struct.
    ///
    /// [`syn::Ident`]: struct@syn::Ident
    self_ty: (&'i syn::Ident, &'i syn::Generics),

    /// [`FlatMatcher`]s for every enum variant, or for the struct container.
    matches: Vec<FlatMatcher<'i>>,

    /// [`FlatExpansion::matches`] grouped by its similar representation for detecting whether their
    /// case-insensitivity should be disabled.
    similar_matches: HashMap<String, Vec<&'i syn::Ident>>,

    /// Optional [`attr::RenameAll`] indicating the case convertion to be applied to all the matched
    /// values (enum variants or struct itself).
    rename_all: Option<attr::RenameAll>,
}

impl<'i> TryFrom<&'i syn::DeriveInput> for FlatExpansion<'i> {
    type Error = syn::Error;

    fn try_from(input: &'i syn::DeriveInput) -> syn::Result<Self> {
        let attr_ident = &format_ident!("from_str");

        let mut matches = match &input.data {
            syn::Data::Struct(data) => {
                if !data.fields.is_empty() {
                    return Err(syn::Error::new(
                        data.fields.span(),
                        "only structs with no fields can derive `FromStr`",
                    ));
                }
                vec![FlatMatcher::from_unit_struct(input, data)?]
            }
            syn::Data::Enum(data) => data
                .variants
                .iter()
                .map(FlatMatcher::from_enum_variant)
                .collect::<syn::Result<_>>()?,
            syn::Data::Union(_) => {
                return Err(syn::Error::new(
                    input.span(),
                    "expected an enum or a struct for flat `FromStr` derive",
                ))
            }
        };

        let num_defaults = matches
            .iter()
            .filter(|matcher| matcher.settings.default.is_some())
            .count();
        if num_defaults > 1 {
            return Err(syn::Error::new(
                input.span(),
                "only one `#[try_from(default)]` attribute is allowed",
            ));
        }

        // push the default matcher to the end
        matches.sort_by(|lhs, rhs| {
            if lhs.settings.default.is_some() {
                cmp::Ordering::Greater
            } else if rhs.settings.default.is_some() {
                cmp::Ordering::Less
            } else {
                cmp::Ordering::Equal
            }
        });

        let rename_all = attr::RenameAll::parse_attrs(&input.attrs, attr_ident)?
            .map(Spanning::into_inner);

        let mut similar_matches = <HashMap<_, Vec<_>>>::new();
        if rename_all.is_none() {
            for matcher in &matches {
                if matcher.settings.default.is_some()
                    || matcher.settings.forward.is_some()
                    || matcher.settings.skip.is_some()
                {
                    continue;
                }

                let name = match &matcher.settings.rename {
                    Some(rename) => rename.matcher.value(),
                    None => matcher.ident.to_string(),
                };
                let lowercased = name.to_lowercase();

                if let Some(aliases) = &matcher.settings.aliases {
                    for alias in aliases.iter() {
                        let renamed_lowercase = alias.value().to_lowercase();
                        if renamed_lowercase != lowercased {
                            similar_matches
                                .entry(renamed_lowercase)
                                .or_default()
                                .push(matcher.ident);
                        }
                    }
                }

                if let Some(rename_all) = &matcher.settings.rename_all {
                    let renamed_lowercase = rename_all.convert_case(&name);
                    if renamed_lowercase != lowercased {
                        similar_matches
                            .entry(renamed_lowercase)
                            .or_default()
                            .push(matcher.ident);
                    }
                }

                similar_matches
                    .entry(lowercased)
                    .or_default()
                    .push(matcher.ident);
            }
        }

        let mut exact_matches = <HashMap<String, Vec<String>>>::new();
        for matcher in &matches {
            if matcher.settings.default.is_some()
                || matcher.settings.forward.is_some()
                || matcher.settings.skip.is_some()
            {
                continue;
            }

            let ident = matcher.ident.to_string();

            if let Some(aliases) = &matcher.settings.aliases {
                for alias in aliases.iter() {
                    let alias = alias.value();
                    exact_matches.entry(alias).or_default().push(ident.clone());
                }
            }

            if let Some(rename) = &matcher.settings.rename {
                exact_matches
                    .entry(rename.matcher.value())
                    .or_default()
                    .push(ident.clone());
                continue;
            }

            let name = matcher.ident.to_string();
            let match_renamer =
                matcher.settings.rename_all.as_ref().or(rename_all.as_ref());

            let exact = if let Some(renamer) = match_renamer {
                renamer.convert_case(&name)
            } else {
                let lowercased = name.to_lowercase();
                if similar_matches[&lowercased].len() > 1 {
                    name.clone()
                } else {
                    lowercased
                }
            };
            exact_matches.entry(exact).or_default().push(ident.clone());
        }

        if let Some((string, variants)) =
            exact_matches.into_iter().find(|(_, vs)| vs.len() > 1)
        {
            return Err(syn::Error::new(
                input.ident.span(),
                format!(
                    "`{}` variants cannot have the same \"{string}\" string representation",
                    variants.join("`, `"),
                ),
            ));
        }

        Ok(Self {
            self_ty: (&input.ident, &input.generics),
            matches,
            similar_matches,
            rename_all,
        })
    }
}

impl ToTokens for FlatExpansion<'_> {
    /// Expands a flat [`FromStr`] implementations for an enum.
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let ty = self.self_ty.0;
        let (impl_generics, ty_generics, where_clause) =
            self.self_ty.1.split_for_impl();
        let ty_name = ty.to_string();

        let forwarded = self
            .matches
            .iter()
            .filter(|matcher| matcher.settings.forward.is_some())
            .map(|matcher| {
                let constructor = matcher.data.self_constructor([parse_quote! {v}]);
                quote! {
                    if let Some(v) = derive_more::core::str::FromStr::from_str(s).map(|v| #constructor).ok() {
                        return Ok(v);
                    }
                }
            })
            .collect::<Vec<TokenStream>>();

        let match_arms = self
            .matches
            .iter()
            .filter(|matcher| {
                matcher.settings.skip.is_none() && matcher.settings.forward.is_none()
            })
            .map(|matcher| {
                let constructor = matcher.data.self_constructor_empty();
                let span = matcher.ident.span();

                if matcher.settings.default.is_some() {
                    return quote! {
                        _ => #constructor,
                    };
                }

                let mut aliases = matcher
                    .settings
                    .aliases
                    .as_ref()
                    .map(|a| a.iter())
                    .into_iter()
                    .flatten()
                    .map(|alias| quote! { | #alias })
                    .collect::<TokenStream>();
                if !aliases.is_empty() {
                    aliases = quote! { #aliases => #constructor, };
                }

                if let Some(attr::Rename { matcher: match_str }) =
                    matcher.settings.rename.as_ref()
                {
                    return quote! {
                        #aliases
                        #match_str => #constructor,
                    };
                }

                if let Some(renamer) = matcher
                    .settings
                    .rename_all
                    .as_ref()
                    .or(self.rename_all.as_ref())
                {
                    let match_str = syn::LitStr::new(
                        renamer
                            .convert_case(matcher.ident.to_string().as_str())
                            .as_str(),
                        span,
                    );
                    return quote! {
                        #aliases
                        #match_str => #constructor,
                    };
                }

                let name = matcher.ident.to_string();
                let lowercased = name.to_lowercase();
                if let Some(similar) = self.similar_matches.get(&lowercased) {
                    if similar.len() > 1 {
                        let match_str = syn::LitStr::new(name.as_str(), span);
                        return quote! {
                            #aliases
                            #match_str => #constructor,
                        };
                    }
                }

                let match_str = syn::LitStr::new(name.as_str(), span);

                quote! {
                    #aliases
                    _ if s.eq_ignore_ascii_case(#match_str) => #constructor,
                }
            })
            .collect::<Vec<TokenStream>>();

        let (error_type, fallback_constructor) = self
            .matches
            .iter()
            .rev() // default case was sorted to the end
            .find(|m| m.settings.default.is_some())
            .map(|m| {
                (quote! { core::convert::Infallible }, {
                    let constructor = m.data.self_constructor_empty();
                    quote! { #constructor }
                })
            })
            .unwrap_or_else(|| {
                (
                    quote! { derive_more::FromStrError },
                    quote! {
                        return derive_more::core::result::Result::Err(
                            derive_more::FromStrError::new(#ty_name),
                        )
                    },
                )
            });

        quote! {
            #[allow(unreachable_code)] // for empty enums
            #[automatically_derived]
            impl #impl_generics derive_more::core::str::FromStr for #ty #ty_generics #where_clause {
                type Err = #error_type;

                fn from_str(
                    s: &str,
                ) -> derive_more::core::result::Result<Self, derive_more::FromStrError> {
                    #(#forwarded)*
                    derive_more::core::result::Result::Ok(match s {
                        #( #match_arms )*
                        _ => #fallback_constructor,
                    })
                }
            }
        }.to_tokens(tokens);
    }
}

struct FlatMatcher<'i> {
    ident: &'i syn::Ident,
    settings: MatcherSettings,
    data: Either<&'i syn::DataStruct, &'i syn::Variant>,
}

impl<'i> FlatMatcher<'i> {
    fn from_enum_variant(variant: &'i syn::Variant) -> syn::Result<Self> {
        let attr_ident = &format_ident!("from_str");

        let settings =
            MatcherSettings::parse_attrs(variant.attrs.as_slice(), attr_ident)?
                .map(|spanning| spanning.item)
                .unwrap_or_default();

        if settings.default.is_some() && !variant.fields.is_empty() {
            return Err(syn::Error::new(
                variant.ident.span(),
                "`#[from_str(default)]` is not supported for non-unit variants",
            ));
        }

        if settings.forward.is_some() {
            if variant.fields.len() != 1 {
                return Err(syn::Error::new(
                    variant.ident.span(),
                    "`#[from_str(forward)]` is only supported for variants with exactly one field",
                ));
            }
        } else if !variant.fields.is_empty() {
            return Err(syn::Error::new(
                variant.fields.span(),
                "only enum variants with no fields can derive `FromStr` unless `#[from_str(forward)]` is specified",
            ));
        }

        Ok(Self {
            ident: &variant.ident,
            settings,
            data: Either::Right(variant),
        })
    }

    fn from_unit_struct(
        input: &'i syn::DeriveInput,
        data: &'i syn::DataStruct,
    ) -> syn::Result<Self> {
        let attr_ident = &format_ident!("from_str");

        let settings =
            MatcherSettings::parse_attrs(input.attrs.as_slice(), attr_ident)?
                .map(|spanning| spanning.item)
                .unwrap_or_default();

        if settings.default.is_some() {
            return Err(syn::Error::new(
                input.ident.span(),
                "#[from_str(default)] is not supported for unit structs",
            ));
        }

        if settings.forward.is_some() {
            return Err(syn::Error::new(
                input.ident.span(),
                "#[from_str(forward)] is not supported for unit structs",
            ));
        }

        if settings.skip.is_some() {
            return Err(syn::Error::new(
                input.ident.span(),
                "#[from_str(skip)] is not supported for unit structs",
            ));
        }

        Ok(Self {
            ident: &input.ident,
            settings,
            data: Either::Left(data),
        })
    }
}

#[derive(Default)]
struct MatcherSettings {
    aliases: Option<attr::Aliases>,
    default: Option<attr::Default>,
    forward: Option<attr::Forward>,
    rename: Option<attr::Rename>,
    rename_all: Option<attr::RenameAll>,
    skip: Option<attr::Skip>,
}

impl Parse for MatcherSettings {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        mod keyword {
            use syn::custom_keyword;

            custom_keyword!(aliases);
            custom_keyword!(default);
            custom_keyword!(forward);
            custom_keyword!(ignore);
            custom_keyword!(rename);
            custom_keyword!(rename_all);
            custom_keyword!(skip);
        }

        fn parse<T: Parse>(input: syn::parse::ParseStream) -> syn::Result<T> {
            T::parse(input)
        }

        let default = Self::default();

        let ahead = input.lookahead1();

        if ahead.peek(keyword::aliases) {
            Ok(Self {
                aliases: Some(parse(input)?),
                ..default
            })
        } else if ahead.peek(keyword::default) {
            Ok(Self {
                default: Some(parse(input)?),
                ..default
            })
        } else if ahead.peek(keyword::forward) {
            Ok(Self {
                forward: Some(parse(input)?),
                ..default
            })
        } else if ahead.peek(keyword::ignore) || ahead.peek(keyword::skip) {
            Ok(Self {
                skip: Some(parse(input)?),
                ..default
            })
        } else if ahead.peek(keyword::rename) {
            Ok(Self {
                rename: Some(parse(input)?),
                ..default
            })
        } else if ahead.peek(keyword::rename_all) {
            Ok(Self {
                rename_all: Some(parse(input)?),
                ..default
            })
        } else {
            Err(ahead.error())
        }
    }
}

impl ParseMultiple for MatcherSettings {
    fn merge_attrs(
        prev: Spanning<Self>,
        new: Spanning<Self>,
        name: &syn::Ident,
    ) -> syn::Result<Spanning<Self>> {
        let Spanning {
            span: prev_span,
            item: prev,
        } = prev;
        let Spanning {
            span: new_span,
            item: new,
        } = new;

        let aliases = ParseMultiple::merge_opt_attrs(
            Spanning::new(prev.aliases, prev_span).transpose(),
            Spanning::new(new.aliases, new_span).transpose(),
            name,
        )?;
        let default = ParseMultiple::merge_opt_attrs(
            Spanning::new(prev.default, prev_span).transpose(),
            Spanning::new(new.default, new_span).transpose(),
            name,
        )?;
        let forward = ParseMultiple::merge_opt_attrs(
            Spanning::new(prev.forward, prev_span).transpose(),
            Spanning::new(new.forward, new_span).transpose(),
            name,
        )?;
        let rename = ParseMultiple::merge_opt_attrs(
            Spanning::new(prev.rename, prev_span).transpose(),
            Spanning::new(new.rename, new_span).transpose(),
            name,
        )?;
        let rename_all = ParseMultiple::merge_opt_attrs(
            Spanning::new(prev.rename_all, prev_span).transpose(),
            Spanning::new(new.rename_all, new_span).transpose(),
            name,
        )?;
        let skip = ParseMultiple::merge_opt_attrs(
            Spanning::new(prev.skip, prev_span).transpose(),
            Spanning::new(new.skip, new_span).transpose(),
            name,
        )?;

        if aliases.is_some() {
            if default.is_some() {
                return Err(syn::Error::new(
                    new_span,
                    format!(
                        r#"cannot use `#[{name}(alias = "...")]` and `#[{name}(default = "...")]` together"#
                    ),
                ));
            }

            if forward.is_some() {
                return Err(syn::Error::new(
                    new_span,
                    format!(
                        r#"cannot use `#[{name}(alias = "...")]` and `#[{name}(forward = "...")]` together"#
                    ),
                ));
            }
        }

        if default.is_some() {
            if forward.is_some() {
                return Err(syn::Error::new(
                    new_span,
                    format!(
                        r#"cannot use `#[{name}(forward = "...")]` and `#[{name}(default = "...")]` together"#
                    ),
                ));
            }

            if rename.is_some() {
                return Err(syn::Error::new(
                    new_span,
                    format!(
                        r#"cannot use `#[{name}(rename = "...")]` and `#[{name}(default = "...")]` together"#
                    ),
                ));
            }

            if rename_all.is_some() {
                return Err(syn::Error::new(
                    new_span,
                    format!(
                        r#"cannot use `#[{name}(rename_all = "...")]` and `#[{name}(forward = "...")]` together"#
                    ),
                ));
            }

            if skip.is_some() {
                return Err(syn::Error::new(
                    new_span,
                    format!(
                        r#"cannot use `#[{name}(skip)]` and `#[{name}(forward = "...")]` together"#
                    ),
                ));
            }
        }

        if forward.is_some() {
            if rename.is_some() {
                return Err(syn::Error::new(
                    new_span,
                    format!(
                        r#"cannot use `#[{name}(rename = "...")]` and `#[{name}(forward = "...")]` together"#
                    ),
                ));
            }

            if rename_all.is_some() {
                return Err(syn::Error::new(
                    new_span,
                    format!(
                        r#"cannot use `#[{name}(rename_all = "...")]` and `#[{name}(forward = "...")]` together"#
                    ),
                ));
            }

            if skip.is_some() {
                return Err(syn::Error::new(
                    new_span,
                    format!(
                        r#"cannot use `#[{name}(skip)]` and `#[{name}(forward = "...")]` together"#
                    ),
                ));
            }
        }

        if rename.is_some() {
            if rename_all.is_some() {
                return Err(syn::Error::new(
                    new_span,
                    format!(
                        r#"cannot use `#[{name}(rename = "...")]` and `#[{name}(rename_all = "...")]` together"#
                    ),
                ));
            }

            if skip.is_some() {
                return Err(syn::Error::new(
                    new_span,
                    format!(
                        r#"cannot use `#[{name}(rename = "...")]` and `#[{name}(skip)]` together"#
                    ),
                ));
            }
        }

        if rename_all.is_some() && skip.is_some() {
            return Err(syn::Error::new(
                new_span,
                format!(
                    r#"cannot use `#[{name}(rename_all = "...")]` and `#[{name}(skip)]` together"#
                ),
            ));
        }

        Ok(Spanning::new(
            Self {
                aliases: aliases.map(|a| a.item),
                default: default.map(|d| d.item),
                forward: forward.map(|f| f.item),
                rename: rename.map(|r| r.item),
                rename_all: rename_all.map(|r| r.item),
                skip: skip.map(|s| s.item),
            },
            prev_span.join(new_span).unwrap_or(prev_span),
        ))
    }
}

/// Extension of [`syn::Fields`] used by this expansion.
trait FieldsExt {
    /// Generates a `name`d constructor with the provided `values` assigned to these
    /// [`syn::Fields`].
    ///
    /// # Panics
    ///
    /// If number of provided `values` doesn't match number of these [`syn::Fields`].
    fn constructor(
        &self,
        name: &syn::Path,
        values: impl IntoIterator<Item = syn::Ident>,
    ) -> TokenStream;

    /// Generates a `Self` type constructor with the provided `values` assigned to these
    /// [`syn::Fields`].
    ///
    /// # Panics
    ///
    /// If number of provided `values` doesn't match number of these [`syn::Fields`].
    fn self_constructor(
        &self,
        values: impl IntoIterator<Item = syn::Ident>,
    ) -> TokenStream {
        self.constructor(&self.self_ty(), values)
    }

    /// Generates a `Self` type constructor with no fields.
    ///
    /// # Panics
    ///
    /// If these [`syn::Fields`] are not [empty].
    ///
    /// [empty]: syn::Fields::is_empty
    fn self_constructor_empty(&self) -> TokenStream {
        self.self_constructor(iter::empty())
    }

    /// Returns a [`syn::Path`] representing a `Self` type of these [`syn::Fields`].
    fn self_ty(&self) -> syn::Path {
        parse_quote! { Self }
    }
}

impl FieldsExt for syn::Fields {
    fn constructor(
        &self,
        name: &syn::Path,
        values: impl IntoIterator<Item = syn::Ident>,
    ) -> TokenStream {
        let values = values.into_iter();
        let fields = match self {
            Self::Named(fields) => {
                let initializers = fields.named.iter().zip(values).map(|(f, value)| {
                    let ident = &f.ident;
                    quote! { #ident: #value }
                });
                Some(quote! { { #( #initializers, )*} })
            }
            Self::Unnamed(_) => Some(quote! { ( #( #values, )* ) }),
            Self::Unit => None,
        };
        quote! { #name #fields }
    }
}

impl FieldsExt for syn::Field {
    fn constructor(
        &self,
        name: &syn::Path,
        values: impl IntoIterator<Item = syn::Ident>,
    ) -> TokenStream {
        let mut values = values.into_iter();
        let value = values.next().expect("expected a single value");
        if values.next().is_some() {
            panic!("expected a single value");
        }

        if let Some(ident) = &self.ident {
            quote! { #name { #ident: #value } }
        } else {
            quote! { #name(#value) }
        }
    }
}

impl FieldsExt for syn::Variant {
    fn constructor(
        &self,
        name: &syn::Path,
        values: impl IntoIterator<Item = syn::Ident>,
    ) -> TokenStream {
        self.fields.constructor(name, values)
    }

    fn self_ty(&self) -> syn::Path {
        let variant = &self.ident;

        parse_quote! { Self::#variant }
    }
}

impl FieldsExt for syn::DataStruct {
    fn constructor(
        &self,
        name: &syn::Path,
        values: impl IntoIterator<Item = syn::Ident>,
    ) -> TokenStream {
        self.fields.constructor(name, values)
    }
}

impl<L: FieldsExt, R: FieldsExt> FieldsExt for Either<&L, &R> {
    fn constructor(
        &self,
        name: &syn::Path,
        values: impl IntoIterator<Item = syn::Ident>,
    ) -> TokenStream {
        match self {
            Self::Left(l) => l.constructor(name, values),
            Self::Right(r) => r.constructor(name, values),
        }
    }

    fn self_ty(&self) -> syn::Path {
        match self {
            Self::Left(l) => l.self_ty(),
            Self::Right(r) => r.self_ty(),
        }
    }
}
