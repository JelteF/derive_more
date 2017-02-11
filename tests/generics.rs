#![allow(dead_code, non_camel_case_types)]
#[macro_use]
extern crate derive_more;


#[derive(From, Not, Add, AddAssign, Mul)]
struct Wrapped<T: Clone>(T);


#[derive(From, Not, Add, AddAssign, Mul)]
struct WrappedDouble<T: Clone, U: Clone>(T, U);

#[derive(From, Not, Add, AddAssign, Mul)]
struct Struct<T: Clone> {
    t: T,
}

#[derive(From, Not, Add, AddAssign, Mul)]
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
