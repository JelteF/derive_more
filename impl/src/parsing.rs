#![allow(dead_code)]

use std::iter;

use proc_macro2::{Delimiter, Spacing, TokenStream, TokenTree};
use quote::ToTokens;
use syn::parse::{Parse, ParseStream};
use syn::{
    buffer::Cursor, punctuated::Punctuated, spanned::Spanned as _, token, Error, Result,
};

impl Parse for Type {
    fn parse(input: ParseStream) -> Result<Self> {
        input.step(|c| {
            let outer = *c;

            if let Some((mut cursor, paren_span, next_item)) =
                outer.group(Delimiter::Parenthesis)
            {
                let mut items = Punctuated::new();
                while !cursor.eof() {
                    let (stream, c) = Self::parse_one(cursor).ok_or_else(|| {
                        Error::new(cursor.span(), "failed to parse type")
                    })?;
                    items.push_value(stream);
                    cursor = c;
                    if let Some((p, c)) = punct(',')(cursor) {
                        items.push_punct(token::Comma(p.span()));
                        cursor = c;
                    }
                }
                if items.len() == 1 && !items.trailing_punct() {
                    // TODO: docs
                    let stream = outer
                        .token_tree()
                        .unwrap_or_else(|| unreachable!())
                        .0
                        .into_token_stream();
                    Ok((Type::Other(stream), next_item))
                } else {
                    Ok((
                        Type::Tuple {
                            paren: token::Paren(paren_span),
                            items,
                        },
                        next_item,
                    ))
                }
            } else {
                Self::parse_one(outer)
                    .map(|(s, c)| (Self::Other(s), c))
                    .ok_or_else(|| Error::new(outer.span(), "failed to parse type"))
            }
        })
    }
}

#[derive(Debug)]
pub enum Type {
    Tuple {
        paren: token::Paren,
        items: Punctuated<TokenStream, token::Comma>,
    },
    Other(TokenStream),
}

impl ToTokens for Type {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Type::Tuple { paren, items } => {
                paren.surround(tokens, |tokens| items.to_tokens(tokens))
            }
            Type::Other(other) => other.to_tokens(tokens),
        }
    }
}

impl Type {
    pub fn parse_one(c: Cursor<'_>) -> Option<(TokenStream, Cursor<'_>)> {
        take_until1(
            alt([&mut balanced_pair(punct('<'), punct('>')), &mut token_tree]),
            punct(','),
        )(c)
    }
}

#[derive(Debug)]
pub struct Expr(pub TokenStream);

impl Parse for Expr {
    fn parse(input: ParseStream) -> Result<Self> {
        input.step(|c| {
            take_until1(
                alt([
                    &mut seq([&mut colon2, &mut qself]),
                    &mut seq([&mut qself, &mut colon2]),
                    &mut balanced_pair(punct('|'), punct('|')),
                    &mut token_tree,
                ]),
                punct(','),
            )(*c)
            .map(|(stream, cursor)| (Self(stream), cursor))
            .ok_or_else(|| Error::new(c.span(), "failed to parse expression"))
        })
    }
}

impl ToTokens for Expr {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.0.to_tokens(tokens);
    }
}

pub fn type_hack(c: Cursor<'_>) -> Option<(TokenStream, Cursor<'_>)> {
    take_until1(
        alt([&mut balanced_pair(punct('<'), punct('>')), &mut token_tree]),
        punct(','),
    )(c)
}

pub fn r#type(c: Cursor<'_>) -> Option<(TokenStream, Cursor<'_>)> {
    alt([
        &mut type_array_or_slice,
        &mut type_path,
        &mut type_ptr,
        &mut type_reference,
        &mut type_trait_object,
        &mut type_tuple,
    ])(c)
}

pub fn type_tuple(c: Cursor<'_>) -> Option<(TokenStream, Cursor<'_>)> {
    c.token_tree().and_then(|(tt, c)| {
        matches!(&tt, TokenTree::Group(g) if g.delimiter() == Delimiter::Parenthesis)
            .then(|| (tt.into_token_stream(), c))
    })
}

pub fn type_trait_object(c: Cursor<'_>) -> Option<(TokenStream, Cursor<'_>)> {
    seq([&mut ident("dyn"), &mut r#type])(c)
}

pub fn type_reference(c: Cursor<'_>) -> Option<(TokenStream, Cursor<'_>)> {
    seq([
        &mut punct('&'),
        &mut opt(lifetime),
        &mut opt(ident("mut")),
        &mut r#type,
    ])(c)
}

pub fn lifetime(c: Cursor<'_>) -> Option<(TokenStream, Cursor<'_>)> {
    seq([
        &mut punct_with_spacing('\'', Spacing::Joint),
        &mut any_ident,
    ])(c)
}

pub fn type_ptr(c: Cursor<'_>) -> Option<(TokenStream, Cursor<'_>)> {
    seq([
        &mut punct('*'),
        &mut opt(alt([&mut ident("const"), &mut ident("mut")])),
        &mut r#type,
    ])(c)
}

/// Tries to parse [`syn::TypePath`].
pub fn type_path(c: Cursor<'_>) -> Option<(TokenStream, Cursor<'_>)> {
    seq([&mut opt(qself), &mut path])(c)
}

/// Tries to parse [`syn::QSelf`].
pub fn qself(c: Cursor<'_>) -> Option<(TokenStream, Cursor<'_>)> {
    balanced_pair(punct('<'), punct('>'))(c)
}

/// Tries to parse [`syn::Path`].
pub fn path(c: Cursor<'_>) -> Option<(TokenStream, Cursor<'_>)> {
    seq([&mut opt(colon2), &mut punctuated1(path_segment, colon2)])(c)
}

/// Tries to parse [`syn::PathSegment`].
pub fn path_segment(c: Cursor<'_>) -> Option<(TokenStream, Cursor<'_>)> {
    seq([&mut any_ident, &mut path_arguments])(c)
}

/// Tries to parse [`syn::PathArguments`].
pub fn path_arguments(c: Cursor<'_>) -> Option<(TokenStream, Cursor<'_>)> {
    opt(alt([
        &mut group(Delimiter::Parenthesis),
        &mut balanced_pair(punct('<'), punct('>')),
    ]))(c)
}

pub fn type_array_or_slice(c: Cursor<'_>) -> Option<(TokenStream, Cursor<'_>)> {
    c.token_tree().and_then(|(tt, c)| {
        matches!(&tt, TokenTree::Group(g) if g.delimiter() == Delimiter::Bracket)
            .then(|| (tt.into_token_stream(), c))
    })
}

/// Tries to parse [`syn::token::Colon2`].
pub fn colon2(c: Cursor<'_>) -> Option<(TokenStream, Cursor<'_>)> {
    seq([
        &mut punct_with_spacing(':', Spacing::Joint),
        &mut punct(':'),
    ])(c)
}

pub fn ident(
    n: &'static str,
) -> impl FnMut(Cursor<'_>) -> Option<(TokenStream, Cursor<'_>)> {
    move |c| {
        let (ident, c) = c.ident().filter(|(i, _)| i == n)?;
        Some((ident.into_token_stream(), c))
    }
}

pub fn any_ident(c: Cursor<'_>) -> Option<(TokenStream, Cursor<'_>)> {
    let (ident, c) = c.ident()?;
    Some((ident.into_token_stream(), c))
}

pub fn punct_with_spacing(
    p: char,
    spacing: Spacing,
) -> impl FnMut(Cursor<'_>) -> Option<(TokenStream, Cursor<'_>)> {
    move |c| {
        c.punct().and_then(|(punct, c)| {
            (punct.as_char() == p && punct.spacing() == spacing)
                .then(|| (punct.into_token_stream(), c))
        })
    }
}

pub fn punct(p: char) -> impl FnMut(Cursor<'_>) -> Option<(TokenStream, Cursor<'_>)> {
    move |c| {
        c.punct().and_then(|(punct, c)| {
            (punct.as_char() == p).then(|| (punct.into_token_stream(), c))
        })
    }
}

pub fn any_group() -> impl FnMut(Cursor<'_>) -> Option<(TokenStream, Cursor<'_>)> {
    move |c| {
        c.token_tree().and_then(|(tt, c)| {
            matches!(&tt, TokenTree::Group(_)).then(|| (tt.into_token_stream(), c))
        })
    }
}

pub fn group(
    d: Delimiter,
) -> impl FnMut(Cursor<'_>) -> Option<(TokenStream, Cursor<'_>)> {
    move |c| {
        c.token_tree().and_then(|(tt, c)| {
            matches!(&tt, TokenTree::Group(g) if g.delimiter() == d)
                .then(|| (tt.into_token_stream(), c))
        })
    }
}

pub fn token_tree(c: Cursor<'_>) -> Option<(TokenStream, Cursor<'_>)> {
    c.token_tree().map(|(tt, c)| (tt.into_token_stream(), c))
}

pub fn eof(c: Cursor<'_>) -> Option<(TokenStream, Cursor<'_>)> {
    c.eof().then(|| (TokenStream::new(), c))
}

/// Parses until balanced amount of `open` and `close` [`TokenTree::Punct`] or
/// eof.
///
/// [`Cursor`] should be pointing **right after** the first `open`ing.
pub fn balanced_pair(
    mut open: impl FnMut(Cursor<'_>) -> Option<(TokenStream, Cursor<'_>)>,
    mut close: impl FnMut(Cursor<'_>) -> Option<(TokenStream, Cursor<'_>)>,
) -> impl FnMut(Cursor<'_>) -> Option<(TokenStream, Cursor<'_>)> {
    move |c| {
        let (mut out, mut c) = open(c)?;
        let mut count = 1;

        while count != 0 {
            let (stream, cursor) = if let Some(closing) = close(c) {
                count -= 1;
                closing
            } else if let Some(opening) = open(c) {
                count += 1;
                opening
            } else {
                let (tt, c) = c.token_tree()?;
                (tt.into_token_stream(), c)
            };
            out.extend(stream);
            c = cursor;
        }

        Some((out, c))
    }
}

pub fn seq<const N: usize>(
    mut parsers: [&mut dyn FnMut(Cursor<'_>) -> Option<(TokenStream, Cursor<'_>)>; N],
) -> impl FnMut(Cursor<'_>) -> Option<(TokenStream, Cursor<'_>)> + '_ {
    move |c| {
        parsers
            .iter_mut()
            .fold(Some((TokenStream::new(), c)), |out, parser| {
                let (mut out, mut c) = out?;
                let (stream, cursor) = parser(c)?;
                out.extend(stream);
                c = cursor;
                Some((out, c))
            })
    }
}

pub fn alt<const N: usize>(
    mut parsers: [&mut dyn FnMut(Cursor<'_>) -> Option<(TokenStream, Cursor<'_>)>; N],
) -> impl FnMut(Cursor<'_>) -> Option<(TokenStream, Cursor<'_>)> + '_ {
    move |c| {
        parsers
            .iter_mut()
            .find_map(|parser| parser(c).map(|(s, c)| (s, c)))
    }
}

pub fn punctuated1<P, S>(
    mut parser: P,
    mut sep: S,
) -> impl FnMut(Cursor<'_>) -> Option<(TokenStream, Cursor<'_>)>
where
    P: FnMut(Cursor<'_>) -> Option<(TokenStream, Cursor<'_>)>,
    S: FnMut(Cursor<'_>) -> Option<(TokenStream, Cursor<'_>)>,
{
    move |c| {
        let (mut out, mut c) = parser(c)?;
        while let Some((stream, cursor)) = sep(c) {
            out.extend(stream);
            c = cursor;
            if let Some((stream, cursor)) = parser(c) {
                out.extend(stream);
                c = cursor;
            } else {
                break;
            }
        }
        Some((out, c))
    }
}

pub fn take_until1<P, U>(
    mut parser: P,
    mut until: U,
) -> impl FnMut(Cursor<'_>) -> Option<(TokenStream, Cursor<'_>)>
where
    P: FnMut(Cursor<'_>) -> Option<(TokenStream, Cursor<'_>)>,
    U: FnMut(Cursor<'_>) -> Option<(TokenStream, Cursor<'_>)>,
{
    move |mut cursor| {
        let mut out = TokenStream::new();
        let mut parsed = false;

        loop {
            if cursor.eof() || until(cursor).is_some() {
                return parsed.then_some((out, cursor));
            }

            let (stream, c) = parser(cursor)?;
            out.extend(stream);
            cursor = c;
            parsed = true;
        }
    }
}

pub fn take_while1<F>(
    mut f: F,
) -> impl FnMut(Cursor<'_>) -> Option<(TokenStream, Cursor<'_>)>
where
    F: FnMut(Cursor<'_>) -> Option<(TokenStream, Cursor<'_>)>,
{
    move |mut c| {
        let mut out = TokenStream::new();

        iter::from_fn(|| {
            let (stream, cursor) = f(c)?;
            out.extend(stream);
            c = cursor;
            Some(())
        })
        .last()
        .map(|_| (out, c))
    }
}

pub fn opt<F>(mut f: F) -> impl FnMut(Cursor<'_>) -> Option<(TokenStream, Cursor<'_>)>
where
    F: FnMut(Cursor<'_>) -> Option<(TokenStream, Cursor<'_>)>,
{
    move |c| Some(f(c).unwrap_or_else(|| (TokenStream::new(), c)))
}

#[cfg(test)]
mod spec {
    use std::{fmt::Debug, str::FromStr};

    use itertools::Itertools as _;
    use proc_macro2::TokenStream;
    use quote::ToTokens;
    use syn::{
        parse::{Parse, Parser as _},
        punctuated::Punctuated,
        token::Comma,
    };

    use super::{Expr, Type};

    fn assert<'a, T: Debug + Parse + ToTokens>(
        input: &'a str,
        parsed: impl AsRef<[&'a str]>,
    ) {
        let parsed = parsed.as_ref();
        let punctuated = Punctuated::<T, Comma>::parse_terminated
            .parse2(TokenStream::from_str(input).unwrap())
            .unwrap();

        assert_eq!(
            parsed.len(),
            punctuated.len(),
            "Wrong length\n\
             Expected: {parsed:?}\n\
             Found: {punctuated:?}",
        );

        punctuated
            .iter()
            .map(|ty| ty.to_token_stream().to_string())
            .zip(parsed)
            .enumerate()
            .for_each(|(i, (found, expected))| {
                assert_eq!(
                    *expected, &found,
                    "Mismatch at index {i}\n\
                     Expected: {parsed:?}\n\
                     Found: {punctuated:?}",
                );
            });
    }

    mod tuple {
        use super::*;

        #[test]
        fn group_not_tuple() {
            let group_not_tuple = "(Type)";
            match syn::parse_str::<Type>(group_not_tuple).unwrap() {
                Type::Other(group) => {
                    assert_eq!(group.to_string(), group_not_tuple);
                }
                tuple => panic!("Expected `Type::Other(_)`, found: {tuple:?}"),
            }
        }

        #[test]
        fn cases() {
            let cases = [
                "[Type ; 3]",
                "fn (usize) -> bool",
                "for <'a > fn (&'a usize) -> bool",
                "(Type)",
                "path :: to :: Type",
                "path :: to :: Generic < Type >",
                "< Type as Trait >:: Assoc",
                "< Type as Trait >:: Assoc < GAT >",
                "* const ()",
                "* mut ()",
                "& i32",
                "&'static str",
                "& [str]",
                "dyn Trait",
                "dyn Trait + Send",
                "()",
                "(Type ,)",
                "(Type , Type)",
                "(Type , Type ,)",
            ];

            assert::<Type>("", []);
            for i in 1..4 {
                for permutations in cases.into_iter().permutations(i) {
                    let mut input = permutations.join(",");
                    assert::<Type>(&input, &permutations);
                    input.push(',');
                    assert::<Type>(&input, &permutations);
                }
            }
        }
    }

    mod expr {
        use super::*;

        #[test]
        fn cases() {
            let cases = [
                "ident",
                "[a , b , c , d]",
                "counter += 1",
                "async { fut . await }",
                "a < b",
                "a > b",
                "{ let x = (a , b) ; }",
                "invoke (a , b)",
                "foo as f64",
                "| a , b | a + b",
                "obj . k",
                "for pat in expr { break pat ; }",
                "if expr { true } else { false }",
                "vector [2]",
                "1",
                "\"foo\"",
                "loop { break i ; }",
                "format ! (\"{}\" , q)",
                "match n { Some (n) => { } , None => { } }",
                "x . foo ::< T > (a , b)",
                "x . foo ::< T < [T < T >; if a < b { 1 } else { 2 }] >, { a < b } > (a , b)",
                "(a + b)",
                "i32 :: MAX",
                "1 .. 2",
                "& a",
                "[0u8 ; N]",
                "(a , b , c , d)",
                "< Ty as Trait > :: T",
                "< Ty < Ty < T >, { a < b } > as Trait < T > > :: T",
            ];

            assert::<Expr>("", []);
            for i in 1..4 {
                for permutations in cases.into_iter().permutations(i) {
                    let mut input = permutations.clone().join(",");
                    assert::<Expr>(&input, &permutations);
                    input.push(',');
                    assert::<Expr>(&input, &permutations);
                }
            }
        }
    }
}
