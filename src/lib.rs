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

use std::collections::HashMap;

use syntax::ast::{Ident, Ty, VariantData, ItemKind, MetaItem, EnumDef};
use syntax::codemap::Span;
use syntax::ext::base::{Annotatable, ExtCtxt};
use syntax::ext::build::AstBuilder;
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
