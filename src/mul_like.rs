use syntax::ast::*;
use syntax::codemap::Span;
use syntax::ext::base::{Annotatable, ExtCtxt};
use syntax::ext::build::AstBuilder;
use syntax::ext::quote::rt::{ExtParseUtils, ToTokens};
use syntax::ptr::P;

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

                ItemKind::Struct(VariantData::Struct(ref fields, _), _) => {
                    Some((x.ident, cx.ty_ident(span, x.ident), struct_content(cx, span, x, fields, method_name)))
                },

                //ItemKind::Enum(ref definition, _) => {
                //    let input_type = x.ident;
                //    Some((x.ident, quote_ty!(cx, Result<$input_type, &'static str>), enum_content(cx, span, x, definition, method_name)))
                //},

                _ => None,
            }
        },
        _ => None,
    };

    let (input_type, output_type, (block, tys)) = match result {
        Some(x) => x,
        _ => {
            cx.span_fatal(span, &format!("only structs can use `derive({})`", trait_name))
        },
    };


    let t = quote_ty!(cx, T);
    let trait_path = cx.path_all(span, true,
                                 cx.std_path(&["ops", &trait_name]),
                                 vec![],
                                 vec![t.clone()],
                                 vec![],
                                 );

    let mut bounds: Vec<_> = tys.iter().map(|ty| {
        let binding = typebinding_str(cx, span, "Output", ty.clone());

        cx.typarambound(cx.path_all(span, true,
                                    cx.std_path(&["ops", &trait_name]),
                                    vec![],
                                    vec![ty.clone()],
                                    vec![binding],
                                    ))
    }).collect();

    if bounds.len() > 1 {
        bounds.push(cx.typarambound(cx.path_global(span, cx.std_path(&["marker", "Copy"]))));

    }

    let where_ = whereclause(span, t, P::from_vec(bounds));

    let code = quote_item!(cx,
        impl<T> $trait_path for $input_type $where_ {
            type Output = $output_type;
            fn $method_ident(self, rhs: T) -> $output_type {
                $block
            }
        }
    );

    push(Annotatable::Item(code.unwrap()));

}

fn tuple_content(cx: &mut ExtCtxt, span: Span, item: &P<Item>, fields: &Vec<StructField>, method_name: String) -> (P<Expr>, Vec<P<Ty>>) {
    let type_name = item.ident;
    let mut exprs = vec![];
    let mut tys = vec![];

    for (i, f) in fields.iter().enumerate() {
        let i = &i.to_string();
        exprs.push(cx.parse_expr(format!("rhs.{}(self.{})", method_name, i)));
        tys.push(f.node.ty.clone());
    }

    (cx.expr_call_ident(span, type_name, exprs), tys)
}

fn struct_content(cx: &mut ExtCtxt, span: Span, item: &P<Item>, fields: &Vec<StructField>, method_name: String) -> (P<Expr>, Vec<P<Ty>>) {
    let type_name = item.ident;
    let mut filled_fields = vec![];
    let mut tys = vec![];

    for f in fields {
        let (field_id, field_name) = match f.ident {
            Some(x) => (x, x.name.as_str()),
            _ => unreachable!(),
        };
        filled_fields.push(cx.field_imm(span, field_id,
                                        cx.parse_expr(format!("rhs.{}(self.{})", method_name, field_name))));
        tys.push(f.node.ty.clone())
    }

    (cx.expr_struct_ident(span, type_name, filled_fields), tys)
}

fn typebinding_str(cx: &mut ExtCtxt, span: Span, name: &str, ty: P<Ty>) -> TypeBinding {
    TypeBinding {
        id: DUMMY_NODE_ID,
        ident: cx.ident_of(name),
        ty: ty,
        span: span,
    }
}

fn whereclause(span: Span, ty: P<Ty>, bounds: TyParamBounds) -> WhereClause {
    WhereClause {
        id: DUMMY_NODE_ID,
        predicates: vec![WherePredicate::BoundPredicate(WhereBoundPredicate {
            span: span,
            bound_lifetimes: vec![],
            bounded_ty: ty,
            bounds: bounds,
        })]
    }
}
