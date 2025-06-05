//! Implementations of [`cmp`]-related derive macros.
//!
//! [`cmp`]: std::cmp

pub(crate) mod eq;
pub(crate) mod partial_eq;

trait TypeExt {
    /// Checks whether the provided [`syn::Type`] is contained within this [`syn::Type`]
    /// structurally (part of the actual structure).
    ///
    /// # False positives
    ///
    /// This check naturally gives a false positives when a type parameter is not used directly in
    /// a field, but its associative type does (e.g. `struct Foo<T: Some>(T::Assoc);`). This is
    /// because the structure of the type cannot be scanned by its name only.
    fn contains_type_structurally(&self, needle: &syn::Type) -> bool;
}

impl TypeExt for syn::Type {
    fn contains_type_structurally(&self, needle: &Self) -> bool {
        if self == needle {
            return true;
        }
        match self {
            syn::Type::Array(syn::TypeArray { elem, .. })
            | syn::Type::Group(syn::TypeGroup { elem, .. })
            | syn::Type::Paren(syn::TypeParen { elem, .. })
            | syn::Type::Ptr(syn::TypePtr { elem, .. })
            | syn::Type::Reference(syn::TypeReference { elem, .. })
            | syn::Type::Slice(syn::TypeSlice { elem, .. }) => {
                elem.contains_type_structurally(needle)
            }
            syn::Type::Tuple(syn::TypeTuple { elems, .. }) => {
                elems.iter().any(|elem| elem.contains_type_structurally(needle))
            }
            syn::Type::Path(syn::TypePath { path, .. }) => path
                .segments
                .iter()
                .filter_map(|seg| {
                    if let syn::PathArguments::AngleBracketed(args) = &seg.arguments {
                        Some(&args.args)
                    } else {
                        None
                    }
                })
                .flatten()
                .any(|generic_arg| {
                    matches!(
                        generic_arg,
                        syn::GenericArgument::Type(ty) if ty.contains_type_structurally(needle),
                    )
                }),
            syn::Type::BareFn(_)
            | syn::Type::ImplTrait(_)
            | syn::Type::Infer(_)
            | syn::Type::Macro(_)
            | syn::Type::Never(_)
            | syn::Type::TraitObject(_)
            | syn::Type::Verbatim(_) => false,
            _ => unimplemented!(
                "syntax is not supported by `derive_more`, please report a bug"
            ),
        }
    }
}

#[cfg(test)]
mod type_ext_spec {
    use quote::ToTokens as _;
    use syn::parse_quote;

    use super::TypeExt as _;

    #[test]
    fn contains() {
        for (input, container) in [
            (parse_quote! { Self }, parse_quote! { Self }),
            (parse_quote! { Self }, parse_quote! { (Self) }),
            (parse_quote! { Self }, parse_quote! { (Self,) }),
            (parse_quote! { Self }, parse_quote! { (Self, Foo) }),
            (
                parse_quote! { Self },
                parse_quote! { (Foo, Bar, Baz, Self) },
            ),
            (parse_quote! { Self }, parse_quote! { [Self] }),
            (parse_quote! { Self }, parse_quote! { [Self; N] }),
            (parse_quote! { Self }, parse_quote! { *const Self }),
            (parse_quote! { Self }, parse_quote! { *mut Self }),
            (parse_quote! { Self }, parse_quote! { &'a Self }),
            (parse_quote! { Self }, parse_quote! { &'a mut Self }),
            (parse_quote! { Self }, parse_quote! { Box<Self> }),
            (parse_quote! { Self }, parse_quote! { PhantomData<Self> }),
            (parse_quote! { Self }, parse_quote! { Arc<Mutex<Self>> }),
            (
                parse_quote! { Self },
                parse_quote! { [*const (&'a [Arc<Mutex<Self>>],); 0] },
            ),
        ] {
            let container: syn::Type = container; // for type inference only
            assert!(
                container.contains_type_structurally(&input),
                "cannot find type `{}` in type `{}`",
                input.into_token_stream(),
                container.into_token_stream(),
            );
        }
    }

    #[test]
    fn not_contains() {
        for (input, container) in [
            (parse_quote! { Self }, parse_quote! { Foo }),
            (parse_quote! { Self }, parse_quote! { (Foo) }),
            (parse_quote! { Self }, parse_quote! { (Foo,) }),
            (parse_quote! { Self }, parse_quote! { (Foo, Bar, Baz) }),
            (parse_quote! { Self }, parse_quote! { [Foo] }),
            (parse_quote! { Self }, parse_quote! { [Foo; N] }),
            (parse_quote! { Self }, parse_quote! { *const Foo }),
            (parse_quote! { Self }, parse_quote! { *mut Foo }),
            (parse_quote! { Self }, parse_quote! { &'a Foo }),
            (parse_quote! { Self }, parse_quote! { &'a mut Foo }),
            (parse_quote! { Self }, parse_quote! { Box<Foo> }),
            (parse_quote! { Self }, parse_quote! { PhantomData<Foo> }),
            (parse_quote! { Self }, parse_quote! { Arc<Mutex<Foo>> }),
            (
                parse_quote! { Self },
                parse_quote! { [*const (&'a [Arc<Mutex<Foo>>],); 0] },
            ),
            (parse_quote! { Self }, parse_quote! { fn(Self) -> Foo }),
            (parse_quote! { Self }, parse_quote! { fn(Foo) -> Self }),
            (parse_quote! { Self }, parse_quote! { impl Foo<Self> }),
            (
                parse_quote! { Self },
                parse_quote! { impl Foo<Type = Self> },
            ),
            (parse_quote! { Self }, parse_quote! { dyn Foo<Self> }),
            (
                parse_quote! { Self },
                parse_quote! { dyn Sync + Foo<Type = Self> },
            ),
        ] {
            let container: syn::Type = container; // for type inference only
            assert!(
                !container.contains_type_structurally(&input),
                "found type `{}` in type `{}`",
                input.into_token_stream(),
                container.into_token_stream(),
            );
        }
    }
}
