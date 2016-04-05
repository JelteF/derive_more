use syntax::ast::*;
use syntax_ext::deriving::generic::ty;
use syntax::codemap::{Span, Spanned};
use syntax::ext::base::{Annotatable, ExtCtxt};
use syntax::ext::build::AstBuilder;
use syntax::ext::quote::rt::ExtParseUtils;
use syntax::ptr::P;
use syntax::print::pprust::ty_to_string;

pub fn expand(cx: &mut ExtCtxt, span: Span, item: &Annotatable,
                          push: &mut FnMut(Annotatable), trait_name: &str) {
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

                ItemKind::Struct(VariantData::Struct(ref fields, _), _) => {
                    Some((x.ident, cx.ty_ident(span, x.ident), struct_content(cx, span, x, fields, method_name)))
                },

                ItemKind::Enum(ref definition, _) => {
                    let input_type = x.ident;
                    Some((x.ident, quote_ty!(cx, Result<$input_type, &'static str>), enum_content(cx, span, x, definition, method_name)))
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

fn tuple_content(cx: &mut ExtCtxt, span: Span, item: &P<Item>, fields: &Vec<StructField>, method_name: String) -> P<Expr> {
    let type_name = item.ident;
    let mut exprs: Vec<P<Expr>>= vec![];

    for i in 0..fields.len() {
        let i = &i.to_string();
        exprs.push(cx.parse_expr(format!("self.{}.{}(rhs.{})", i, method_name, i)));
    }

    cx.expr_call_ident(span, type_name, exprs)
}

fn struct_content(cx: &mut ExtCtxt, span: Span, item: &P<Item>, fields: &Vec<StructField>, method_name: String) -> P<Expr> {
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

fn enum_content(cx: &mut ExtCtxt, span: Span, item: &P<Item>, enum_def: &EnumDef, method_name: String) -> P<Expr> {
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
            VariantData::Unit(_) =>  {
                matches.push(quote_arm!(cx, ($type_path, $type_path) => Err("Cannot add unit types together"), ));
            }

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
