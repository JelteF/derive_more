#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(nightly, feature(never_type))]
#![allow(dead_code)] // some code is tested for type checking only

use derive_more::AsVariant;

#[derive(AsVariant)]
enum Either<TLeft, TRight> {
    Left(TLeft),
    Right(TRight),
}

#[derive(AsVariant)]
#[derive(Debug, PartialEq)]
#[as_variant(ref, ref_mut)]
enum Maybe<T> {
    Nothing,
    Just(T),
}

#[derive(AsVariant)]
enum Color {
    Rgb(u8, u8, u8),
    Cmyk(u8, u8, u8, u8),
}

/// With lifetime
#[derive(AsVariant)]
enum Nonsense<'a, T> {
    Ref(&'a T),
    NoRef,
    #[as_variant(ignore)]
    NoRefIgnored,
}

#[derive(AsVariant)]
enum WithConstraints<T>
where
    T: Copy,
{
    One(T),
    Two,
}

#[derive(AsVariant)]
enum KitchenSink<'a, 'b, T1: Copy, T2: Clone>
where
    T2: Into<T1> + 'b,
{
    Left(&'a T1),
    Right(&'b T2),
    OwnBoth(T1, T2),
    Empty,
    NeverMind(),
    NothingToSeeHere(),
}

/// Single variant enum
#[derive(AsVariant)]
enum Single {
    Value(i32),
}

#[derive(AsVariant)]
#[derive(Debug, PartialEq)]
#[as_variant(ref, ref_mut)]
enum Tuple<T> {
    None,
    Single(T),
    Double(T, T),
    Triple(T, T, T),
}

#[test]
pub fn test_as_variant() {
    assert_eq!(Maybe::<()>::Nothing.as_nothing(), Some(()));
    assert_eq!(Maybe::Just(1).as_just_ref(), Some(&1));
    assert_eq!(Maybe::Just(42).as_just_mut(), Some(&mut 42));

    assert_eq!(Maybe::<()>::Nothing.as_just(), None);
    assert_eq!(Maybe::Just(1).as_nothing_ref(), None);
    assert_eq!(Maybe::Just(42).as_nothing_mut(), None);
}

#[test]
pub fn test_as_variant_mut() {
    let mut value = Tuple::Double(1, 12);

    if let Some((a, b)) = value.as_double_mut() {
        *a = 9;
        *b = 10;
    }

    assert_eq!(value, Tuple::Double(9, 10));
}

#[cfg(nightly)]
mod never {
    use super::*;

    #[derive(AsVariant)]
    enum Enum {
        Tuple(!),
        TupleMulti(i32, !),
    }
}
