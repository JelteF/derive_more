#![allow(dead_code)]
#[macro_use]
extern crate derive_more;


#[derive(From, Not, Add, AddAssign)]
struct Wrapped<T: Clone>(T);


#[derive(From, Not, Add, AddAssign)]
struct WrappedDouble<T: Clone, U: Clone>(T, U);

#[derive(From, Not, Add, AddAssign)]
struct Struct<T: Clone> {
    t: T,
}

#[derive(From, Not, Add, AddAssign)]
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
