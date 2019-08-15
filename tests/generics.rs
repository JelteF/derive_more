#![allow(dead_code, non_camel_case_types)]
#[macro_use]
extern crate derive_more;

use std::fmt::Display;

#[derive(
    From,
    FromStr,
    Display,
    Index,
    Not,
    Add,
    Mul,
    IndexMut,
    AddAssign,
    Constructor
)]
struct Wrapped<T: Clone + Display>(T);

#[derive(From, Not, Add, Mul, AddAssign, Constructor)]
struct WrappedDouble<T: Clone, U: Clone>(T, U);

#[derive(
    From,
    FromStr,
    Display,
    Index,
    Not,
    Add,
    Mul,
    IndexMut,
    AddAssign,
    Constructor
)]
struct Struct<T: Clone + Display> {
    t: T,
}

#[derive(From, Not, Add, Mul, AddAssign, Constructor)]
struct DoubleStruct<T: Clone, U: Clone> {
    t: T,
    u: U,
}

#[derive(From, Not, Add)]
enum TupleEnum<T: Clone, U: Clone> {
    Tuple(T),
    DoubleTuple(T, U),
}

#[derive(From, Not, Add)]
enum StructEnum<T: Clone, U: Clone> {
    Struct { t: T },
    DoubleStruct { t: T, u: U },
}
