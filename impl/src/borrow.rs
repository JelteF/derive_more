//! Implementation of `Borrow`/`BorrowMut` derive macros.

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{
    parse::{Parse, ParseStream},
    parse_quote,
    spanned::Spanned as _,
};

use crate::utils::{
    add_extra_generic_type_param,
    attr::{self, ParseMultiple as _},
    Either, GenericsSearch, Spanning,
};

/// Expands a `Borrow`/`BorrowMut` derive macro.
pub(crate) fn expand(
    input: &syn::DeriveInput,
    trait_name: &'static str,
) -> syn::Result<TokenStream> {
    let trait_ident = format_ident!("{trait_name}");
    let is_mutable = trait_name == "BorrowMut";
    let attr_name =
        format_ident!("{}", if is_mutable { "borrow_mut" } else { "borrow" });
    let trait_info = ExpansionCtx {
        trait_ident: &trait_ident,
        attr_name: &attr_name,
        is_mutable,
    };

    let data = match &input.data {
        syn::Data::Struct(data) => Ok(data),
        syn::Data::Enum(e) => Err(syn::Error::new(
            e.enum_token.span(),
            format!("`{trait_ident}` cannot be derived for enums"),
        )),
        syn::Data::Union(u) => Err(syn::Error::new(
            u.union_token.span(),
            format!("`{trait_ident}` cannot be derived for unions"),
        )),
    }?;

    if data.fields.len() != 1 {
        return Err(syn::Error::new(
            if data.fields.is_empty() {
                data.struct_token.span
            } else {
                data.fields.span()
            },
            format!("`{trait_ident}` can only be derived for structs with exactly one field"),
        ));
    }

    let field = data.fields.iter().next().unwrap();
    let struct_attr = StructAttribute::parse_attrs(&input.attrs, trait_info.attr_name)?;
    let field_attr = FieldAttribute::parse_attrs(&field.attrs, trait_info.attr_name)?;

    if struct_attr.is_some() && field_attr.is_some() {
        return Err(syn::Error::new(
            field.span(),
            format!(
                "`#[{}(...)]` cannot be placed on both struct and its field",
                trait_info.attr_name,
            ),
        ));
    }

    let is_forwarded = if struct_attr.is_some() {
        true
    } else if let Some(attr) = field_attr {
        match attr.item {
            FieldAttribute::Direct(_) => false,
            FieldAttribute::Forward(_) => true,
            FieldAttribute::Skip(skip) => {
                return Err(syn::Error::new(
                    attr.span,
                    format!(
                        "`#[{}({})]` cannot be used when deriving `{}` \
                         because exactly one field must be borrowed",
                        trait_info.attr_name,
                        skip.name(),
                        trait_info.trait_ident,
                    ),
                ));
            }
        }
    } else {
        false
    };

    validate_field_type(field, &input.generics, is_forwarded, trait_info)?;

    Ok(expand_impl(
        &input.ident,
        &input.generics,
        field,
        is_forwarded,
        trait_info,
    ))
}

fn validate_field_type(
    field: &syn::Field,
    generics: &syn::Generics,
    is_forwarded: bool,
    trait_info: ExpansionCtx<'_>,
) -> syn::Result<()> {
    let field_ty = &field.ty;

    if is_assoc_type_involving_generic_param(field_ty, generics) {
        return Err(syn::Error::new(
            field_ty.span(),
            format!(
                "`{}` cannot be derived for an associated type projection involving generic \
                 parameters because the impl can overlap with `core`'s blanket `{}` \
                 implementation",
                trait_info.trait_ident,
                trait_info.trait_ident,
            ),
        ));
    }

    if is_forwarded && is_forwarded_overlap(field_ty, generics) {
        return Err(syn::Error::new(
            field_ty.span(),
            format!(
                "`#[{}(forward)]` cannot be used on a generic parameter field, an associated \
                 type projection involving generic parameters, or a forwarding pointer to one \
                 because the impl can overlap with `core`'s blanket `{}` implementation",
                trait_info.attr_name,
                trait_info.trait_ident,
            ),
        ));
    }

    Ok(())
}

fn is_assoc_type_involving_generic_param(
    ty: &syn::Type,
    generics: &syn::Generics,
) -> bool {
    let ty = match ty {
        syn::Type::Group(ty) => {
            return is_assoc_type_involving_generic_param(&ty.elem, generics)
        }
        syn::Type::Paren(ty) => {
            return is_assoc_type_involving_generic_param(&ty.elem, generics)
        }
        syn::Type::Path(ty) => ty,
        _ => return false,
    };

    if ty.qself.is_some() {
        return GenericsSearch::from(generics).any_in(&syn::Type::Path(ty.clone()));
    }

    ty.path.segments.len() > 1
        && generics
            .type_params()
            .any(|param| ty.path.segments[0].ident == param.ident)
}

fn is_bare_generic_param(ty: &syn::Type, generics: &syn::Generics) -> bool {
    let syn::Type::Path(ty) = ty else {
        return false;
    };

    ty.qself.is_none()
        && ty.path.segments.len() == 1
        && generics
            .type_params()
            .any(|param| ty.path.segments[0].ident == param.ident)
}

fn is_forwarded_overlap(ty: &syn::Type, generics: &syn::Generics) -> bool {
    let ty = peel_forwarding_pointer(ty);
    is_bare_generic_param(ty, generics)
        || is_assoc_type_involving_generic_param(ty, generics)
}

fn peel_forwarding_pointer(ty: &syn::Type) -> &syn::Type {
    match ty {
        syn::Type::Group(ty) => peel_forwarding_pointer(&ty.elem),
        syn::Type::Paren(ty) => peel_forwarding_pointer(&ty.elem),
        syn::Type::Reference(ty) => peel_forwarding_pointer(&ty.elem),
        syn::Type::Path(type_path) => {
            fundamental_path_inner_type(type_path).map_or(ty, peel_forwarding_pointer)
        }
        _ => ty,
    }
}

fn fundamental_path_inner_type(ty: &syn::TypePath) -> Option<&syn::Type> {
    let segment = fundamental_path_segment(ty)?;

    let syn::PathArguments::AngleBracketed(arguments) = &segment.arguments else {
        return None;
    };

    arguments.args.iter().find_map(|arg| {
        if let syn::GenericArgument::Type(ty) = arg {
            Some(ty)
        } else {
            None
        }
    })
}

fn fundamental_path_segment(ty: &syn::TypePath) -> Option<&syn::PathSegment> {
    if ty.qself.is_some() {
        return None;
    }

    let segments = ty.path.segments.iter().collect::<Vec<_>>();
    match segments.as_slice() {
        [segment] if segment.ident == "Box" || segment.ident == "Pin" => Some(segment),
        [std_or_alloc, boxed, segment]
            if (std_or_alloc.ident == "std" || std_or_alloc.ident == "alloc")
                && boxed.ident == "boxed"
                && segment.ident == "Box" =>
        {
            Some(segment)
        }
        [std_or_core, pin, segment]
            if (std_or_core.ident == "std" || std_or_core.ident == "core")
                && pin.ident == "pin"
                && segment.ident == "Pin" =>
        {
            Some(segment)
        }
        [derive_more, core, pin, segment]
            if derive_more.ident == "derive_more"
                && core.ident == "core"
                && pin.ident == "pin"
                && segment.ident == "Pin" =>
        {
            Some(segment)
        }
        _ => None,
    }
}

#[derive(Clone, Copy)]
struct ExpansionCtx<'a> {
    trait_ident: &'a syn::Ident,
    attr_name: &'a syn::Ident,
    is_mutable: bool,
}

fn expand_impl(
    ty_ident: &syn::Ident,
    generics: &syn::Generics,
    field: &syn::Field,
    is_forwarded: bool,
    trait_info: ExpansionCtx<'_>,
) -> TokenStream {
    let field_ty = &field.ty;
    let field_ident = field
        .ident
        .as_ref()
        .map_or_else(|| quote! { 0 }, |ident| quote! { #ident });
    let (_, ty_gens, _) = generics.split_for_impl();
    let borrow_ty = extra_borrow_type_param(generics, trait_info);
    let trait_ident = trait_info.trait_ident;
    let method_ident = trait_info.attr_name;
    let mutability = trait_info.is_mutable.then(|| quote! { mut });

    let trait_ty = if is_forwarded {
        quote! { #borrow_ty }
    } else {
        quote! { #field_ty }
    };
    let return_ty = if is_forwarded {
        quote! { #trait_ty }
    } else {
        borrowed_type_for_return(field_ty)
    };
    let trait_path = quote! {
        derive_more::core::borrow::#trait_ident<#trait_ty>
    };

    let generics = if is_forwarded {
        let mut generics = add_extra_generic_type_param(
            generics,
            quote! { #borrow_ty: ?derive_more::core::marker::Sized },
        );
        generics.make_where_clause().predicates.push(parse_quote! {
            #field_ty: #trait_path
        });
        generics
    } else {
        generics.clone()
    };
    let (impl_gens, _, where_clause) = generics.split_for_impl();

    let body = if is_forwarded {
        quote! {
            <#field_ty as #trait_path>::#method_ident(& #mutability self.#field_ident)
        }
    } else {
        quote! {
            & #mutability self.#field_ident
        }
    };

    quote! {
        #[allow(deprecated)] // omit warnings on deprecated fields/variants
        #[allow(unreachable_code)] // omit warnings for `!` and other unreachable types
        #[automatically_derived]
        impl #impl_gens #trait_path for #ty_ident #ty_gens #where_clause {
            #[inline]
            fn #method_ident(& #mutability self) -> & #mutability #return_ty {
                #body
            }
        }
    }
}

fn extra_borrow_type_param(
    generics: &syn::Generics,
    trait_info: ExpansionCtx<'_>,
) -> syn::Ident {
    let prefix = if trait_info.is_mutable {
        "__BorrowMutT"
    } else {
        "__BorrowT"
    };
    let mut ident = format_ident!("{prefix}");
    let mut index = 0;

    while generics.params.iter().any(|param| match param {
        syn::GenericParam::Type(param) => param.ident == ident,
        syn::GenericParam::Const(param) => param.ident == ident,
        syn::GenericParam::Lifetime(_) => false,
    }) {
        index += 1;
        ident = format_ident!("{prefix}{index}");
    }

    ident
}

fn borrowed_type_for_return(ty: &syn::Type) -> TokenStream {
    match ty {
        syn::Type::Group(ty) => borrowed_type_for_return(&ty.elem),
        syn::Type::Paren(ty) => borrowed_type_for_return(&ty.elem),
        syn::Type::TraitObject(ty) => {
            let mut ty = ty.clone();
            if !ty
                .bounds
                .iter()
                .any(|bound| matches!(bound, syn::TypeParamBound::Lifetime(_)))
            {
                ty.bounds.push(parse_quote! { 'static });
            }

            quote! { (#ty) }
        }
        _ => quote! { #ty },
    }
}

type StructAttribute = attr::Forward;

#[derive(Clone, Copy)]
enum FieldAttribute {
    Direct(attr::Empty),
    Forward(attr::Forward),
    Skip(attr::Skip),
}

type UntypedFieldAttribute = Either<attr::Empty, Either<attr::Forward, attr::Skip>>;

impl From<UntypedFieldAttribute> for FieldAttribute {
    fn from(value: UntypedFieldAttribute) -> Self {
        match value {
            UntypedFieldAttribute::Left(empty) => Self::Direct(empty),
            UntypedFieldAttribute::Right(Either::Left(forward)) => {
                Self::Forward(forward)
            }
            UntypedFieldAttribute::Right(Either::Right(skip)) => Self::Skip(skip),
        }
    }
}

impl From<FieldAttribute> for UntypedFieldAttribute {
    fn from(value: FieldAttribute) -> Self {
        match value {
            FieldAttribute::Direct(empty) => Self::Left(empty),
            FieldAttribute::Forward(forward) => Self::Right(Either::Left(forward)),
            FieldAttribute::Skip(skip) => Self::Right(Either::Right(skip)),
        }
    }
}

impl Parse for FieldAttribute {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        mod ident {
            use syn::custom_keyword;

            custom_keyword!(forward);
            custom_keyword!(skip);
            custom_keyword!(ignore);
        }

        let ahead = input.lookahead1();

        if ahead.peek(ident::forward) {
            input.parse::<attr::Forward>().map(Self::Forward)
        } else if ahead.peek(ident::skip) || ahead.peek(ident::ignore) {
            input.parse::<attr::Skip>().map(Self::Skip)
        } else {
            Err(ahead.error())
        }
    }
}

impl attr::ParseMultiple for FieldAttribute {
    fn parse_attr_with<P: attr::Parser>(
        attr: &syn::Attribute,
        parser: &P,
    ) -> syn::Result<Self> {
        if matches!(attr.meta, syn::Meta::Path(_)) {
            Ok(Self::Direct(attr::Empty))
        } else {
            attr.parse_args_with(|ps: ParseStream<'_>| parser.parse(ps))
        }
    }

    fn merge_attrs(
        prev: Spanning<Self>,
        new: Spanning<Self>,
        name: &syn::Ident,
    ) -> syn::Result<Spanning<Self>> {
        UntypedFieldAttribute::merge_attrs(
            prev.map(Into::into),
            new.map(Into::into),
            name,
        )
        .map(|attr| attr.map(Self::from))
    }
}
