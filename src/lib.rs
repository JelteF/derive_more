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

use syntax::ast::{VariantData, ItemKind, MetaItem};
use syntax::codemap::Span;
use syntax::ext::base::{Annotatable, ExtCtxt};
use syntax::ext::build::AstBuilder;

/// Provides the hook to expand `#[derive(From)]` into an implementation of `From`
fn expand_derive_from(cx: &mut ExtCtxt, span: Span, _: &MetaItem,
                          item: &Annotatable, push: &mut FnMut(Annotatable)) {

    // Get the that is wrapped by the newtype and do some checks
    let old_type_opt = match *item {
        Annotatable::Item(ref x) => {
            match x.node {
                ItemKind::Struct(VariantData::Tuple(ref y, _), _) => {
                    if y.len() == 1 {
                        Some((x.ident, y[0].node.ty.clone()))
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

    let (ident, old_type) = match old_type_opt {
        Some(x) => x,
        None => {
            cx.span_bug(span, "only newtype structs can use `derive(From)`");
        }
    };

    let code = quote_item!(cx,
        impl ::std::convert::From<$old_type> for $ident {
            fn from(a: $old_type) -> $ident {
                $ident(a)
            }
        }
    ).unwrap();

    push(Annotatable::Item(code));
}
