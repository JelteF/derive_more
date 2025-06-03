//! Implementations of [`cmp`]-related derive macros.
//!
//! [`cmp`]: std::cmp

pub(crate) mod eq;
pub(crate) mod partial_eq;

trait TypeExt {
    fn contains_type_structurally(&self, needle: &syn::Type) -> bool;
}

impl TypeExt for syn::Type {
    fn contains_type_structurally(&self, needle: &syn::Type) -> bool {
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
