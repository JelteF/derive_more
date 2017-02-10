#![allow(dead_code)]
#[macro_use]
extern crate derive_more;


#[derive(From, Not)]
struct Wrapped<T>(T);


#[derive(From, Not)]
struct WrappedDouble<T, U>(T, U);

#[derive(From, Not)]
struct Struct<T> {
    t: T,
}

#[derive(From, Not)]
struct DoubleStruct<T, U> {
    t: T,
    u: U,
}

#[derive(From, Not)]
enum TupleEnum<T, U> {
    Tuple(T),
    DoubleTuple(T, U),
}

#[derive(From, Not)]
enum StructEnum<T, U> {
    Struct { t: T },
    DoubleStruct { t: T, u: U },
}
