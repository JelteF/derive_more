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

mod from;
mod add_like;
mod mul_like;

const ADDLIKE_OPS: &'static [&'static str] = &["Add", "Sub", "BitAnd", "BitOr", "BitXor"];
const MULLIKE_OPS: &'static [&'static str] = &["Mul", "Div", "Rem", "Shr", "Shl"];

#[plugin_registrar]
pub fn plugin_registrar(reg: &mut Registry) {
    reg.register_syntax_extension(intern("derive_From"), MultiDecorator(box from::expand));
    for op in ADDLIKE_OPS {
        let expand = move |cx: &mut ExtCtxt, span: Span, _: &MetaItem, item: &Annotatable, push: &mut FnMut(Annotatable)| {
            add_like::expand(cx, span, item, push, op)
        };
        reg.register_syntax_extension(intern(&format!("derive_{}", op)), MultiDecorator(box expand));
    }
    for op in MULLIKE_OPS {
        let expand = move |cx: &mut ExtCtxt, span: Span, _: &MetaItem, item: &Annotatable, push: &mut FnMut(Annotatable)| {
            mul_like::expand(cx, span, item, push, op)
        };
        reg.register_syntax_extension(intern(&format!("derive_{}", op)), MultiDecorator(box expand));
    }
}


use syntax::ast::*;
use syntax::codemap::Span;
use syntax::ext::base::{Annotatable, ExtCtxt};
