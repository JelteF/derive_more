#![allow(dead_code)]
#[macro_use]
extern crate derive_more;

#[derive(From)]
struct Wrapped<T>(T);

#[derive(From)]
struct WrappedDouble<T, U>(T, U);

#[derive(From)]
struct Struct<T> {
    t: T,
}

#[derive(From)]
struct DoubleStruct<T, U> {
    t: T,
    u: U,
}
