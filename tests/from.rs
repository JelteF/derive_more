#![allow(dead_code)]
#[macro_use]
extern crate derive_more;

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
    NamedBigInt { int: i64 },
    TwoSmallInts(i32, i32),
    NamedBigInts { x: i64, y: i64 },
    Unsigned(u32),
    NamedUnsigned { x: u32 },
}
