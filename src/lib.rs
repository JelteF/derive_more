#![feature(plugin_registrar, box_syntax, rustc_private, custom_derive, quote, plugin, custom_attribute)]

#[macro_use]
extern crate rustc;
extern crate rustc_front;
extern crate rustc_plugin;
#[macro_use]
extern crate syntax;
extern crate syntax_ext;

use rustc_plugin::Registry;
use syntax::parse::token::intern;
use syntax::ext::base::MultiDecorator;


#[plugin_registrar]
pub fn plugin_registrar(reg: &mut Registry) {
    reg.register_syntax_extension(intern("derive_From"), MultiDecorator(box expand_derive_from));
}

use syntax::ast::{Item, VariantData, ItemKind, Expr, MetaItem, Mutability};
use syntax::codemap::Span;
use syntax::ext::base::{Annotatable, ExtCtxt};
use syntax::ext::build::AstBuilder;
use syntax::ptr::P;
use syntax_ext::deriving::generic::{Substructure, TraitDef, ty};
use syntax_ext::deriving::generic::{combine_substructure, FieldInfo, MethodDef};
use syntax_ext::deriving::generic::SubstructureFields::*;
use syntax_ext::deriving::generic::StaticFields::*;
use syntax_ext::deriving;
use syntax::ext::quote::rt::ToTokens;
use syntax::print::pprust::ty_to_string;

/// Provides the hook to expand `#[derive(From)]` into an implementation of `From`
fn expand_derive_from(cx: &mut ExtCtxt, span: Span, meta_item: &MetaItem,
                          item: &Annotatable, push: &mut FnMut(Annotatable)) {
    let old_type_opt = match *item {
        Annotatable::Item(ref x) => {
            match x.node {
                ItemKind::Struct(VariantData::Tuple(ref y, _), _) => {
                    if y.len() == 1 {
                        Some(ty_to_string(&*y[0].node.ty.clone()))
                    }
                    else {
                        None
                    }
                }
                _ => None,
            }
        },
        _ => None,
    };

    let old_type = match old_type_opt {
        Some(x) => x,
        None => {
            cx.span_bug(span, "only newtype structs can use `derive(From)`");
        }
    };

    let old_type = old_type.split("::").collect::<Vec<_>>();
    let mut old_type = ty::Path::new(old_type);
    old_type.global = false;

    let old_type = ty::Literal(old_type);

    let mut from_path = ty::Path::new(vec!["std", "convert", "From"]);

    from_path.params = vec!(box old_type.clone());

    // println!("{:#?}", item);


    let trait_def = TraitDef {
        span: span,
        attributes: vec![],
        path: from_path,
        additional_bounds: vec![],
        generics: ty::LifetimeBounds::empty(),
        is_unsafe: false,
        methods: vec![
            MethodDef {
                name: "from",
                generics: ty::LifetimeBounds::empty(),
                explicit_self: None,
                args: vec!(old_type),
                ret_ty: ty::Self_,
                attributes: vec![],
                is_unsafe: false,
                combine_substructure: combine_substructure(box from_substructure)
            }
        ],
        associated_types: vec![],
    };

    trait_def.expand(cx, meta_item, item, push)
}

// Mostly copied from syntax::ext::deriving::hash
/// Defines how the implementation for `from()` is to be generated
fn from_substructure(cx: &mut ExtCtxt, trait_span: Span, substr: &Substructure) -> P<Expr> {
    let state_expr = match (substr.nonself_args.len(), substr.nonself_args.get(0)) {
        (1, Some(o_f)) => o_f,
        _ => cx.span_bug(trait_span, "incorrect number of arguments in `derive(From)`")
    };

    // println!("{:#?}", substr.type_ident);
    // println!("{:#?}", substr.method_ident);
    // println!("{:#?}", substr.self_args);
    // println!("{:#?}", substr.nonself_args);

    let fields = match *substr.fields {
        StaticStruct(_, Unnamed(ref static_fields)) => static_fields,
        StaticStruct(_, Named(_)) => cx.span_bug(trait_span, "only newtype structs can use `derive(Hash)`"),
        // StaticEnum(_, _) => println!("Its a static enum"),
        _ => cx.span_bug(trait_span, "impossible substructure in `derive(From)`"),
    };

    if fields.len() == 1 {
        let self_path = cx.expr_path(cx.path_ident(trait_span, substr.type_ident));

        let expr = cx.expr_call(trait_span, self_path, vec!(state_expr.clone()));
        return expr;
    }
    else {
        cx.span_bug(trait_span, "only newtype structs can use `derive(From)`")
    }

    cx.expr_block(cx.block(trait_span, vec![], None))

}
