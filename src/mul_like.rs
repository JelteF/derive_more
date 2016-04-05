use syntax::ast::*;
use syntax_ext::deriving::generic::ty;
use syntax::codemap::{Span, Spanned};
use syntax::ext::base::{Annotatable, ExtCtxt};
use syntax::ext::build::AstBuilder;
use syntax::ext::quote::rt::ExtParseUtils;
use syntax::ptr::P;
use syntax::print::pprust::ty_to_string;

pub fn expand(cx: &mut ExtCtxt, span: Span, item: &Annotatable, push: &mut FnMut(Annotatable),
          trait_name: &str) {
    let trait_name = trait_name.to_string();
    let method_name = trait_name.to_lowercase();
    let method_ident = cx.ident_of(&method_name);
    // Get the that is wrapped by the newtype and do some checks
    let result = match *item {
        Annotatable::Item(ref x) => {
            match x.node {
                ItemKind::Struct(VariantData::Tuple(ref fields, _), _) => {
                    Some((x.ident, cx.ty_ident(span, x.ident), tuple_content(cx, span, x, fields, method_name)))
                },

                //ItemKind::Struct(VariantData::Struct(ref fields, _), _) => {
                //    Some((x.ident, cx.ty_ident(span, x.ident), struct_content(cx, span, x, fields, method_name)))
                //},

                //ItemKind::Enum(ref definition, _) => {
                //    let input_type = x.ident;
                //    Some((x.ident, quote_ty!(cx, Result<$input_type, &'static str>), enum_content(cx, span, x, definition, method_name)))
                //},

                _ => None,
            }
        },
        _ => None,
    };

    let (input_type, output_type, block) = match result {
        Some(x) => x,
        _ => {
            cx.span_fatal(span, &format!("only structs can use `derive({})`", trait_name))
        },
    };


    let trait_path = cx.path_global(span, cx.std_path(&["ops", &trait_name, "<T>"]));

    let code = quote_item!(cx,
        impl<T: ::std::ops::Mul<i32, Output=i32>> $trait_path for $input_type {
            type Output = $output_type;
            fn $method_ident(self, rhs: T) -> $output_type {
                $block
            }
        }
    ).unwrap();

    push(Annotatable::Item(code));

}

fn tuple_content(cx: &mut ExtCtxt, span: Span, item: &P<Item>, fields: &Vec<StructField>, method_name: String) -> P<Expr> {
    let type_name = item.ident;
    let mut exprs: Vec<P<Expr>>= vec![];

    for i in 0..fields.len() {
        let i = &i.to_string();
        exprs.push(cx.parse_expr(format!("rhs.{}(self.{})", method_name, i)));
    }

    cx.expr_call_ident(span, type_name, exprs)
}



