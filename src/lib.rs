#![feature(plugin_registrar, box_syntax, rustc_private, custom_derive, quote)]

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
    reg.register_syntax_extension(intern("derive_Add"), MultiDecorator(box expand_derive_add));
}

use std::collections::HashMap;

use syntax::ast::*;
use syntax::codemap::Span;
use syntax::ext::base::{Annotatable, ExtCtxt};
use syntax::ext::build::AstBuilder;
use syntax::ext::quote::rt::ExtParseUtils;
use syntax::ptr::P;
use syntax::print::pprust::ty_to_string;

/// Provides the hook to expand `#[derive(From)]` into an implementation of `From`
fn expand_derive_from(cx: &mut ExtCtxt, span: Span, _: &MetaItem,
                          item: &Annotatable, push: &mut FnMut(Annotatable)) {

    // Get the that is wrapped by the newtype and do some checks
    let failed = match *item {
        Annotatable::Item(ref x) => {
            match x.node {
                ItemKind::Struct(VariantData::Tuple(ref structs, _), _) => {
                    if structs.len() == 1 {
                        newtype_from(cx, x.ident, structs[0].node.ty.clone(), push);
                        false
                    }
                    else {
                        true
                    }
                },
                ItemKind::Enum(ref definition, _) => {
                    enum_from(cx, x.ident, definition, push);
                    false

                }
                _ => true,
            }
        },
        _ => true,
    };

    if failed {
        cx.span_bug(span, "only newtype structs can use `derive(From)`");
    }
}

fn newtype_from(cx: &mut ExtCtxt, ident: Ident, old_type: P<Ty>,
                push: &mut FnMut(Annotatable)) {
    let code = quote_item!(cx,
        impl ::std::convert::From<$old_type> for $ident {
            fn from(a: $old_type) -> $ident {
                $ident(a)
            }
        }
    ).unwrap();

    push(Annotatable::Item(code));
}

fn enum_from(cx: &mut ExtCtxt, enum_ident: Ident, definition: &EnumDef,
             push: &mut FnMut(Annotatable)) {
    let mut types = vec![];
    let mut idents = vec![];
    let mut type_counts = HashMap::new();

    for variant in &definition.variants {
        match variant.node.data {
            VariantData::Tuple(ref structs, _) => {
                if structs.len() == 1 {
                    let ty = structs[0].node.ty.clone();
                    idents.push(variant.node.name);
                    types.push(ty.clone());
                    let counter = type_counts.entry(ty_to_string(&*ty)).or_insert(0);
                    *counter += 1;
                }
            }
            _ => (),
        }
    }

    for (ident, old_type) in idents.iter().zip(types) {
        if *type_counts.get(&ty_to_string(&*old_type)).unwrap() != 1 {
            // If more than one newtype is present don't add automatic From, since it is
            // ambiguous.
            continue
        }

        let code = quote_item!(cx,
            impl ::std::convert::From<$old_type> for $enum_ident {
                fn from(a: $old_type) -> $enum_ident {
                    $enum_ident::$ident(a)
                }
            }
        ).unwrap();

        push(Annotatable::Item(code));
    }
}

fn expand_derive_add(cx: &mut ExtCtxt, span: Span, _: &MetaItem,
                     item: &Annotatable, push: &mut FnMut(Annotatable)) {

    // Get the that is wrapped by the newtype and do some checks
    let result = match *item {
        Annotatable::Item(ref x) => {
            match x.node {
                ItemKind::Struct(VariantData::Tuple(ref fields, _), _) => {
                    Some((x.ident, newtype_add_content(cx, span, x, fields)))
                },
                ItemKind::Struct(VariantData::Struct(ref fields, _), _) => {
                    Some((x.ident, struct_add_content(cx, span, x, fields)))
                }
                _ => None,
            }
        },
        _ => None,
    };

    let (type_name, block) = match result {
        Some(x) => x,
        _ => {
            cx.span_fatal(span, "only structs can use `derive(Add)`")
        },
    };

    let code = quote_item!(cx,
        impl ::std::ops::Add for $type_name {
            type Output = $type_name;
            fn add(self, rhs: $type_name) -> $type_name {
                $block
            }
        }
    ).unwrap();

    push(Annotatable::Item(code));

}

fn newtype_add_content(cx: &mut ExtCtxt, span: Span, item: &P<Item>, fields: &Vec<StructField>) -> P<Expr> {
    let type_name = item.ident;
    let mut exprs: Vec<P<Expr>>= vec![];

    for i in 0..fields.len() {
        let i = &i.to_string();
        let self_ = cx.parse_expr("self.".to_string() + i);
        let rhs = cx.parse_expr("rhs.".to_string() + i);
        exprs.push(quote_expr!(cx, $self_ + $rhs))
    }

    cx.expr_call_ident(span, type_name, exprs)
}

fn struct_add_content(cx: &mut ExtCtxt, span: Span, item: &P<Item>, fields: &Vec<StructField>) -> P<Expr> {
    let type_name = item.ident;
    let mut filled_fields = vec![];

    for field in fields {
        let attr = match field.node.kind {
            NamedField(x, _) => x,
            _ => unreachable!(),

        };
        filled_fields.push(cx.field_imm(span, attr,
                                        quote_expr!(cx, self.$attr + rhs.$attr)))
    }

    cx.expr_struct_ident(span, type_name, filled_fields)
}
