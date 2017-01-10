#![feature(box_syntax, rustc_private, quote)]

#[macro_use]
extern crate rustc;
extern crate proc_macro;
extern crate rustc_plugin;
#[macro_use]
extern crate syntax;
extern crate syntax_ext;
#[macro_use]
extern crate quote;
extern crate syn;

use rustc_plugin::Registry;
use syntax::symbol::Symbol;
use syntax::ext::base::MultiDecorator;
use proc_macro::TokenStream;

mod from;
mod add_like;
mod mul_like;

const MULLIKE_OPS: &'static [&'static str] = &["Mul", "Div", "Rem", "Shr", "Shl"];

macro_rules! create_derive(
    ($mod_:ident, $trait_:ident, $fn_name: ident) => {
        #[proc_macro_derive($trait_)]
        pub fn $fn_name(input: TokenStream) -> TokenStream {
            let s = input.to_string();
            let ast = syn::parse_macro_input(&s).unwrap();
            $mod_::expand(&ast, stringify!($trait_)).parse().unwrap()
        }
    }
);

create_derive!(from, From, from_derive);
create_derive!(add_like, Add, add_derive);
create_derive!(add_like, Sub, sub_derive);
create_derive!(add_like, BitAnd, bit_and_derive);
create_derive!(add_like, BitOr, bit_or_derive);
create_derive!(add_like, BitXor, bit_xor_derive);
create_derive!(mul_like, Mul, mul_derive);



//pub fn plugin_registrar(reg: &mut Registry) {
//    reg.register_syntax_extension(Symbol::intern("derive_From"), MultiDecorator(box from::expand));
//    for op in ADDLIKE_OPS {
//        let expand = move |cx: &mut ExtCtxt, span: Span, _: &MetaItem, item: &Annotatable, push: &mut FnMut(Annotatable)| {
//            add_like::expand(cx, span, item, push, op)
//        };
//        reg.register_syntax_extension(Symbol::intern(&format!("derive_{}", op)), MultiDecorator(box expand));
//    }
//    for op in MULLIKE_OPS {
//        let expand = move |cx: &mut ExtCtxt, span: Span, _: &MetaItem, item: &Annotatable, push: &mut FnMut(Annotatable)| {
//            mul_like::expand(cx, span, item, push, op)
//        };
//        reg.register_syntax_extension(Symbol::intern(&format!("derive_{}", op)), MultiDecorator(box expand));
//    }
//}


use syntax::ast::*;
use syntax::codemap::Span;
use syntax::ext::base::{Annotatable, ExtCtxt};
