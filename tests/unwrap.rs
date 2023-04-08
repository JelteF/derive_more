#![allow(dead_code)]

use derive_more::Unwrap;

#[derive(Unwrap)]
enum Either<TLeft, TRight> {
    Left(TLeft),
    Right(TRight),
}

#[derive(Unwrap)]
#[derive(Debug)]
#[unwrap(ref, ref_mut)]
enum Maybe<T: std::fmt::Debug> {
    Nothing,
    Just(T),
}

#[derive(Unwrap)]
enum Color {
    RGB(u8, u8, u8),
    CMYK(u8, u8, u8, u8),
}

#[derive(Unwrap)]
enum Nonsense<'a, T> {
    Ref(&'a T),
    NoRef,
    #[unwrap(ignore)]
    NoRefIgnored,
}

#[derive(Unwrap)]
enum WithConstraints<T>
where
    T: Copy,
{
    One(T),
    Two,
}

#[derive(Unwrap)]
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

#[derive(Unwrap)]
enum Single {
    Value(i32),
}

#[test]
pub fn test_unwrap() {
    assert_eq!(Maybe::<()>::Nothing.unwrap_nothing(), ());
    assert_eq!(Maybe::Just(1).unwrap_just(), 1);

    assert_eq!((&Maybe::Just(42)).unwrap_just_ref(), &42);
    assert_eq!((&mut Maybe::Just(42)).unwrap_just_mut(), &mut 42);
}

#[test]
#[should_panic]
pub fn test_unwrap_panic_1() {
    Maybe::<()>::Nothing.unwrap_just();
}

#[test]
#[should_panic]
pub fn test_unwrap_panic_2() {
    Maybe::Just(2).unwrap_nothing();
}

#[test]
#[should_panic]
pub fn test_unwrap_ref_panic() {
    Maybe::Just(2).unwrap_nothing_ref();
}
