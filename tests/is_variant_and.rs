#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(nightly, feature(never_type))]
#![allow(dead_code)] // some code is tested for type checking only

use derive_more::IsVariantAnd;

#[derive(IsVariantAnd)]
enum Either<TLeft, TRight> {
    Left(TLeft),
    Right(TRight),
}

#[test]
fn test_single_field() {
    let either: Either<u8, i16> = Either::Right(7);
    assert!(either.is_right_and(|x| *x == 7));
    assert!(!either.is_right_and(|x| *x == 0));
    assert!(!either.is_left_and(|_| true));

    let either: Either<u8, i16> = Either::Left(7);
    assert!(either.is_left_and(|x| *x == 7));
    assert!(!either.is_left_and(|x| *x == 0));
    assert!(!either.is_right_and(|_| true));
}

#[derive(IsVariantAnd)]
enum Maybe<T> {
    Nothing,
    Just(T),
}

#[test]
fn test_unit_and_single() {
    let maybe: Maybe<u8> = Maybe::Just(7);
    assert!(maybe.is_just_and(|x| *x == 7));
    assert!(!maybe.is_just_and(|x| *x == 0));
    assert!(!maybe.is_nothing_and(|| true));

    let maybe: Maybe<u8> = Maybe::Nothing;
    assert!(maybe.is_nothing_and(|| true));
    assert!(!maybe.is_nothing_and(|| false));
    assert!(!maybe.is_just_and(|_| true));
}

#[derive(IsVariantAnd)]
enum Color {
    Rgb(u8, u8, u8),
    Cmyk { c: u8, m: u8, y: u8, k: u8 },
}

#[test]
fn test_multi_field_tuple_and_struct() {
    let color = Color::Rgb(0, 40, 80);
    assert!(color.is_rgb_and(|(r, g, b)| *r == 0 && *g == 40 && *b == 80));
    assert!(!color.is_rgb_and(|(r, _, _)| *r == 255));
    assert!(!color.is_cmyk_and(|_| true));

    let color = Color::Cmyk {
        c: 1,
        m: 2,
        y: 3,
        k: 4,
    };
    assert!(color.is_cmyk_and(|(c, m, y, k)| *c + *m + *y + *k == 10));
    assert!(!color.is_cmyk_and(|(c, _, _, _)| *c == 0));
    assert!(!color.is_rgb_and(|_| true));
}

#[derive(IsVariantAnd)]
enum Nonsense<'a, T> {
    Ref(&'a T),
    NoRef,
    #[is_variant_and(ignore)]
    NoRefIgnored,
}

#[test]
fn test_ignore_and_references() {
    let nonsense: Nonsense<u8> = Nonsense::Ref(&7);
    assert!(nonsense.is_ref_and(|x| **x == 7));
    assert!(!nonsense.is_no_ref_and(|| true));

    let nonsense: Nonsense<u8> = Nonsense::NoRef;
    assert!(nonsense.is_no_ref_and(|| true));
    assert!(!nonsense.is_ref_and(|_| true));
}

#[derive(IsVariantAnd)]
enum WithConstraints<T>
where
    T: Copy,
{
    One(T),
    Two,
}

#[test]
fn test_generic_constraints() {
    let wc: WithConstraints<u8> = WithConstraints::One(1);
    assert!(wc.is_one_and(|x| *x == 1));
    assert!(!wc.is_two_and(|| true));

    let wc: WithConstraints<u8> = WithConstraints::Two;
    assert!(wc.is_two_and(|| true));
    assert!(!wc.is_one_and(|_| true));
}

#[derive(IsVariantAnd)]
enum KitchenSink<'a, 'b, T1: Copy, T2: Clone>
where
    T2: Into<T1> + 'b,
{
    Left(&'a T1),
    Right(&'b T2),
    OwnBoth { left: T1, right: T2 },
    Empty,
    NeverMind(),
    NothingToSeeHere {},
}

#[test]
fn test_kitchen_sink() {
    let ks: KitchenSink<u16, u8> = KitchenSink::OwnBoth { left: 1, right: 2 };
    assert!(ks.is_own_both_and(|(left, right)| *left == 1 && *right == 2));
    assert!(!ks.is_own_both_and(|(left, _)| *left == 0));
    assert!(!ks.is_left_and(|_| true));

    let ks: KitchenSink<u16, u8> = KitchenSink::Empty;
    assert!(ks.is_empty_and(|| true));
    assert!(!ks.is_never_mind_and(|| true));

    let ks: KitchenSink<u16, u8> = KitchenSink::NeverMind();
    assert!(ks.is_never_mind_and(|| true));
    assert!(!ks.is_nothing_to_see_here_and(|| true));

    let ks: KitchenSink<u16, u8> = KitchenSink::NothingToSeeHere {};
    assert!(ks.is_nothing_to_see_here_and(|| true));
    assert!(!ks.is_empty_and(|| true));
}

// A single-variant enum exercises the `unreachable_patterns` path.
#[derive(IsVariantAnd)]
enum Single {
    Only(u8),
}

#[test]
fn test_single_variant() {
    let single = Single::Only(5);
    assert!(single.is_only_and(|x| *x == 5));
    assert!(!single.is_only_and(|x| *x == 0));
}

#[cfg(nightly)]
mod never {
    use super::*;

    #[derive(IsVariantAnd)]
    enum Enum {
        Tuple(!),
        Struct { field: ! },
        TupleMulti(i32, !),
        StructMulti { field: !, other: i32 },
    }
}

mod deprecated {
    use super::*;

    #[derive(IsVariantAnd)]
    #[deprecated(note = "enum")]
    enum Enum {
        #[deprecated(note = "variant")]
        Tuple(#[deprecated(note = "field")] i32),
        #[deprecated(note = "variant")]
        Struct {
            #[deprecated(note = "field")]
            field: i32,
        },
    }
}
