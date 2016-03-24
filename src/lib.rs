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

use syntax::ast::{Ident, Ty, VariantData, ItemKind, MetaItem};
use syntax::codemap::Span;
use syntax::ext::base::{Annotatable, ExtCtxt};
use syntax::ext::build::AstBuilder;
use syntax::ptr::P;

/// Provides the hook to expand `#[derive(From)]` into an implementation of `From`
fn expand_derive_from(cx: &mut ExtCtxt, span: Span, _: &MetaItem,
                          item: &Annotatable, push: &mut FnMut(Annotatable)) {

    // Get the that is wrapped by the newtype and do some checks
    let failed = match *item {
        Annotatable::Item(ref x) => {
            match x.node {
                ItemKind::Struct(VariantData::Tuple(ref y, _), _) => {
                    if y.len() == 1 {
                        newtype_from(cx, x.ident, y[0].node.ty.clone(), push);
                        false
                    }
                    else {
                        true
                    }
                },
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
