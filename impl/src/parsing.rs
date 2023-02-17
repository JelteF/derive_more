#![allow(dead_code)]

use std::iter;

use proc_macro2::{Delimiter, Spacing, TokenStream, TokenTree};
use quote::ToTokens as _;
use syn::buffer::Cursor;

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
    move |c| parsers.iter_mut().find_map(|parser| parser(c))
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
    move |mut c| {
        let mut out = TokenStream::new();
        let mut parsed = false;

        loop {
            if c.eof() {
                return parsed.then_some((out, c));
            }

            if let Some((_, c)) = until(c) {
                return Some((out, c));
            }

            let (stream, cursor) = parser(c)?;
            out.extend(stream);
            c = cursor;
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
