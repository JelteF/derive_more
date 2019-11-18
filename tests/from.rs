#![allow(dead_code)]
#[macro_use]
extern crate derive_more;

#[derive(From)]
struct EmptyTuple();

#[derive(From)]
struct EmptyStruct {}

#[derive(From)]
struct EmptyUnit;

#[derive(From)]
struct MyInt(i32);

#[derive(From)]
struct MyInts(i32, i32);

#[derive(From)]
struct Point1D {
    x: i32,
}

#[derive(From)]
struct Point2D {
    x: i32,
    y: i32,
}

#[derive(From)]
enum MixedInts {
    SmallInt(i32),
    NamedBigInt {
        int: i64,
    },
    TwoSmallInts(i32, i32),
    NamedBigInts {
        x: i64,
        y: i64,
    },
    #[from(ignore)]
    Unsigned(u32),
    NamedUnsigned {
        x: u32,
    },
}

#[derive(PartialEq, Eq, Debug)]
#[derive(From)]
#[from(forward)]
struct MyIntForward(u64);

#[test]
fn forward_struct() {
    assert_eq!(MyIntForward(42), 42u32.into());
    assert_eq!(MyIntForward(42), 42u16.into());
    assert_eq!(MyIntForward(42), 42u64.into());
}

#[derive(PartialEq, Eq, Debug)]
#[derive(From)]
enum MixedIntsForward {
    #[from(forward)]
    SmallInt(i32),
    NamedBigInt {
        int: i64,
    },
}

#[test]
fn forward_enum() {
    assert_eq!(MixedIntsForward::SmallInt(42), 42i32.into());
    assert_eq!(MixedIntsForward::SmallInt(42), 42i16.into());
}

#[derive(From, PartialEq)]
enum AutoIgnore {
    SmallInt(i32),
    Uninteresting,
    Uninteresting2,
}

#[test]
fn auto_ignore_variants() {
    assert!(AutoIgnore::SmallInt(42) == 42i32.into());
}

#[derive(From, PartialEq)]
enum AutoIgnoreWithDefaultTrue {
    #[from(ignore)]
    SmallInt(i32),
    Uninteresting,
    Uninteresting2,
}

#[derive(From, PartialEq)]
enum AutoIgnoreWithForwardFields2 {
    #[from(forward)]
    SmallInt(i32),
    SmallIntIgnore(i32),
}

#[test]
fn auto_ignore_with_forward_field2() {
    assert!(AutoIgnoreWithForwardFields2::SmallInt(42) == 42i32.into());
    assert!(AutoIgnoreWithForwardFields2::SmallInt(42) == 42i16.into());
}
