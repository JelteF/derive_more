use syntax::ast::*;
use syntax_ext::deriving;
use syntax_ext::deriving::generic::*;
use syntax::codemap::{Span, Spanned};
use syntax::ext::base::{Annotatable, ExtCtxt};
use syntax::ext::build::AstBuilder;
use syntax::ext::quote::rt::{ExtParseUtils, ToTokens};
use syntax::ptr::P;
use syntax::print::pprust::ty_to_string;

use syntax::parse::token;

pub fn expand(cx: &mut ExtCtxt, span: Span, mitem: &MetaItem, item: &Annotatable, push: &mut FnMut(Annotatable),
          trait_name: &str) {
    let trait_name = trait_name.to_string();
    let method_name = &trait_name.to_lowercase();
    // Get the that is wrapped by the newtype and do some checks
    let result = match *item {
        Annotatable::Item(ref x) => {
            match x.node {
                ItemKind::Struct(VariantData::Tuple(ref fields, _), _) => {
                    Some((ty::Ty::Self_, tuple_content(cx, span, x, fields, method_name)))
                },

                ItemKind::Struct(VariantData::Struct(ref fields, _), _) => {
                    Some((ty::Ty::Self_, struct_content(cx, span, x, fields, method_name)))
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

    let (output_type, block) = match result {
        Some(x) => x,
        _ => {
            cx.span_fatal(span, &format!("only structs can use `derive({})`", trait_name))
        },
    };

    let typaram_name = "T";

    let arg = ty::Literal(ty::Path::new_local(typaram_name));

    let trait_path_vec = vec!["std", "ops", &trait_name];

    let path = ty::Path::new_(trait_path_vec.clone(), None,
                              vec!(box arg.clone()), true);

    let mut ty_bounds = vec![];
    for t in vec!["i32"] {
        let path_params = vec![box ty::Literal(ty::Path::new_local(t))];
        // TODO: No possibility to specify Output=type

        let path = ty::Path::new_(trait_path_vec.clone(), None, path_params,
                                  true);
        ty_bounds.push(path);
    }

    let hash_trait_def = TraitDef {
        span: span,
        attributes: Vec::new(),
        path: path,
        additional_bounds: vec![arg.clone()],
        generics: ty::LifetimeBounds {
            lifetimes: vec![],
            bounds: vec![(typaram_name, ty_bounds)],
        },
        is_unsafe: false,
        associated_types: vec![(cx.ident_of("Output"), output_type)],

        methods: vec![
            MethodDef {
                name: method_name,
                generics: ty::LifetimeBounds::empty(),
                explicit_self: Some(None),
                args: vec![arg],
                ret_ty: ty::Ty::Self_,
                attributes: vec![],
                is_unsafe: false,
                combine_substructure: combine_substructure(Box::new(move |_, _, _| {
                    block.clone()
                }))
            }
        ],

    };

    hash_trait_def.expand(cx, mitem, item, push);
}

fn tuple_content(cx: &mut ExtCtxt, span: Span, item: &P<Item>, fields: &Vec<StructField>, method_name: &str) -> P<Expr> {
    let type_name = item.ident;
    let mut exprs: Vec<P<Expr>>= vec![];

    for i in 0..fields.len() {
        let i = &i.to_string();
        exprs.push(cx.parse_expr(format!("__arg_0.{}(self.{})", method_name, i)));
    }

    cx.expr_call_ident(span, type_name, exprs)
}

fn struct_content(cx: &mut ExtCtxt, span: Span, item: &P<Item>, fields: &Vec<StructField>, method_name: &str) -> P<Expr> {
    let type_name = item.ident;
    let mut filled_fields = vec![];

    for field in fields {
        let (field_id, field_name) = match field.node.kind {
            NamedField(x, _) => (x, x.name.as_str()),
            _ => unreachable!(),


        };
        filled_fields.push(cx.field_imm(span, field_id,
                                        cx.parse_expr(format!("__arg_0.{}(rhs.{})", method_name, field_name))));
    }

    cx.expr_struct_ident(span, type_name, filled_fields)
}

