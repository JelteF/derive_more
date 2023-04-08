#![allow(dead_code)]

use derive_more::TryUnwrap;

#[derive(TryUnwrap)]
enum Either<TLeft, TRight> {
    Left(TLeft),
    Right(TRight),
}

#[derive(TryUnwrap)]
#[derive(Debug, PartialEq)]
#[try_unwrap(ref, ref_mut)]
enum Maybe<T: std::fmt::Debug + PartialEq> {
    Nothing,
    Just(T),
}

#[derive(TryUnwrap)]
enum Color {
    RGB(u8, u8, u8),
    CMYK(u8, u8, u8, u8),
}

#[derive(TryUnwrap)]
enum Nonsense<'a, T> {
    Ref(&'a T),
    NoRef,
    #[try_unwrap(ignore)]
    NoRefIgnored,
}

#[derive(TryUnwrap)]
enum WithConstraints<T>
where
    T: Copy,
{
    One(T),
    Two,
}
#[derive(TryUnwrap)]
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

#[derive(TryUnwrap)]
enum Single {
    Value(i32),
}

#[test]
pub fn test_try_unwrap() {
    assert_eq!(Maybe::<()>::Nothing.try_unwrap_nothing().ok(), Some(()));
    assert_eq!((&Maybe::Just(1)).try_unwrap_just_ref().ok(), Some(&1));
    assert_eq!(
        (&mut Maybe::Just(42)).try_unwrap_just_mut().ok(),
        Some(&mut 42)
    );

    assert_eq!(
        Maybe::<()>::Nothing
            .try_unwrap_just()
            .map_err(|e| e.to_string()),
        Err(
            "Attempt to call `Maybe::try_unwrap_just()` on a `Maybe::Nothing` value"
                .into()
        ),
    );
    assert_eq!(
        (&Maybe::Just(1))
            .try_unwrap_nothing_ref()
            .map_err(|e| e.to_string()),
        Err(
            "Attempt to call `Maybe::try_unwrap_nothing_ref()` on a `Maybe::Just` value"
                .into()
        ),
    );
    assert_eq!(
        (&mut Maybe::Just(42))
            .try_unwrap_nothing_mut()
            .map_err(|e| e.to_string()),
        Err(
            "Attempt to call `Maybe::try_unwrap_nothing_mut()` on a `Maybe::Just` value"
                .into()
        ),
    );
}
