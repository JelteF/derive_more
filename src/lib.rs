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

const INFIX_OPS: &'static [&'static str] = &["Add", "Sub", "Mul", "Div", "Rem", "BitAnd", "BitOr", "BitXor"];

#[plugin_registrar]
pub fn plugin_registrar(reg: &mut Registry) {
    reg.register_syntax_extension(intern("derive_From"), MultiDecorator(box expand_derive_from));
    for op in INFIX_OPS {
        let expand = move |cx: &mut ExtCtxt, span: Span, _: &MetaItem, item: &Annotatable, push: &mut FnMut(Annotatable)| {
            expand_derive_infix_op(cx, span, item, push, op)
        };
        reg.register_syntax_extension(intern(&format!("derive_{}", op)), MultiDecorator(box expand));
    }
}


use std::collections::HashMap;

use syntax::ast::*;
use syntax::codemap::{Span, Spanned};
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

fn expand_derive_infix_op(cx: &mut ExtCtxt, span: Span, item: &Annotatable,
                          push: &mut FnMut(Annotatable), trait_name: &str) {
    let trait_name = trait_name.to_string();
    let method_name = trait_name.to_lowercase();
    let method_ident = cx.ident_of(&method_name);
    // Get the that is wrapped by the newtype and do some checks
    let result = match *item {
        Annotatable::Item(ref x) => {
            match x.node {
                ItemKind::Struct(VariantData::Tuple(ref fields, _), _) => {
                    Some((x.ident, cx.ty_ident(span, x.ident), tuple_infix_op_content(cx, span, x, fields, method_name)))
                },

                ItemKind::Struct(VariantData::Struct(ref fields, _), _) => {
                    Some((x.ident, cx.ty_ident(span, x.ident), struct_infix_op_content(cx, span, x, fields, method_name)))
                },

                ItemKind::Enum(ref definition, _) => {
                    let input_type = x.ident;
                    Some((x.ident, quote_ty!(cx, Result<$input_type, &'static str>), enum_infix_op_content(cx, span, x, definition, method_name)))
                },

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

    let trait_path = cx.path_global(span, cx.std_path(&["ops", &trait_name]));

    let code = quote_item!(cx,
        impl $trait_path for $input_type {
            type Output = $output_type;
            fn $method_ident(self, rhs: $input_type) -> $output_type {
                $block
            }
        }
    ).unwrap();

    push(Annotatable::Item(code));

}

fn tuple_infix_op_content(cx: &mut ExtCtxt, span: Span, item: &P<Item>, fields: &Vec<StructField>, method_name: String) -> P<Expr> {
    let type_name = item.ident;
    let mut exprs: Vec<P<Expr>>= vec![];

    for i in 0..fields.len() {
        let i = &i.to_string();
        exprs.push(cx.parse_expr(format!("self.{}.{}(rhs.{})", i, method_name, i)));
    }

    cx.expr_call_ident(span, type_name, exprs)
}

fn struct_infix_op_content(cx: &mut ExtCtxt, span: Span, item: &P<Item>, fields: &Vec<StructField>, method_name: String) -> P<Expr> {
    let type_name = item.ident;
    let mut filled_fields = vec![];

    for field in fields {
        let (field_id, field_name) = match field.node.kind {
            NamedField(x, _) => (x, x.name.as_str()),
            _ => unreachable!(),


        };
        filled_fields.push(cx.field_imm(span, field_id,
                                        cx.parse_expr(format!("self.{}.{}(rhs.{})", field_name, method_name, field_name))));
    }

    cx.expr_struct_ident(span, type_name, filled_fields)
}

fn enum_infix_op_content(cx: &mut ExtCtxt, span: Span, item: &P<Item>, enum_def: &EnumDef, method_name: String) -> P<Expr> {
    let mut matches: Vec<Arm> = vec![];
    let enum_ident = item.ident;

    // Add paterns for the same enum types for self and rhs
    for variant in &enum_def.variants {
        let ident = variant.node.name;
        let type_path = quote_path!(cx, $enum_ident::$ident);

        match variant.node.data {
            VariantData::Tuple(ref fields, _)  => {
                // The patern that is outputted should look like this:
                // (TypePath(left_vars), TypePath(right_vars) => Ok(TypePath(exprs))
                let mut left_vars = vec![];
                let mut right_vars = vec![];
                let mut exprs = vec![];

                for i in 0..fields.len() {
                    left_vars.push(cx.pat_ident(span, cx.ident_of(&format!("__l_{}", i))));
                    right_vars.push(cx.pat_ident(span, cx.ident_of(&format!("__r_{}", i))));
                    exprs.push(cx.parse_expr(format!("__l_{}.{}(__r_{})", i, method_name, i)));
                }

                let left_patern = cx.pat_enum(span, type_path.clone(), left_vars);
                let right_patern = cx.pat_enum(span, type_path.clone(), right_vars);
                let new_tuple = cx.expr_call(span, cx.expr_path(type_path.clone()), exprs);
                matches.push(quote_arm!(cx, ($left_patern, $right_patern) => Ok($new_tuple),));
            },
            VariantData::Struct(ref fields, _) => {
                // The patern that is outputted should look like this:
                // (TypePath{left_vars}, TypePath{right_vars} => Ok(TypePath{filled_fields})
                let mut left_vars = vec![];
                let mut right_vars = vec![];
                let mut filled_fields = vec![];

                for field in fields {
                    let (field_id, field_name) = match field.node.kind {
                        NamedField(x, _) => (x, x.name.as_str()),
                        _ => unreachable!(),
                    };
                    filled_fields.push(cx.field_imm(
                            span, field_id,
                            cx.parse_expr(format!("__l_{}.{}(__r_{})", field_name, method_name, field_name))));
                    left_vars.push(fieldpat_str(cx, span, field_id, &format!("__l_{}", field_name)));
                    right_vars.push(fieldpat_str(cx, span, field_id, &format!("__r_{}", field_name)));
                }

                let left_patern = cx.pat_struct(span, type_path.clone(), left_vars);
                let right_patern = cx.pat_struct(span, type_path.clone(), right_vars);
                let new_tuple = cx.expr_struct(span, type_path.clone(), filled_fields);
                matches.push(quote_arm!(cx, ($left_patern, $right_patern) => Ok($new_tuple),));
            }
            VariantData::Unit(_) => { },

        }
    }

    // Other combinations should result in an error
    matches.push(quote_arm!(cx, _ => Err("Trying to add mismatched enum types"), ));
    quote_expr!(cx, match (self, rhs) { $matches })
}

fn fieldpat_str(cx: &mut ExtCtxt, span: Span, field_id: Ident, pat: &str) -> Spanned<FieldPat> {
    Spanned{
        span: span,
        node: FieldPat{
            ident: field_id,
            pat: cx.pat_ident(span, cx.ident_of(pat)),
            is_shorthand: false,
        },
    }
}
