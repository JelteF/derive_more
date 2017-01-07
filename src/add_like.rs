use quote::Tokens;
use syn::{Body, Field, Ident, Variant, VariantData, MacroInput, Ty};

pub fn expand(input: &MacroInput) -> Tokens {
    let trait_name = "Add";
    let method_name = trait_name.to_lowercase();
    let method_ident = Ident::from(method_name);
    let result = match input.body {
        Body::Struct(VariantData::Tuple(ref fields)) => {
            Some((input.ident, tuple_content(cx, span, x, fields, method_name)))
        },
        _ => None,
    };
    quote!()
}

fn tuple_content(item: Item, fields: &Vec<StructField>, method_name: String) -> Tokens {
    let type_name = item.ident;
    let mut exprs: = vec![];

    for i in 0..fields.len() {
        let i = &i.to_string();
        exprs.push(cx.parse_expr(format!("self.{}.{}(rhs.{})", i, method_name, i)));
    }

    cx.expr_call_ident(span, type_name, exprs)
}

