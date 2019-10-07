#![allow(dead_code, non_camel_case_types)]
#[macro_use]
extern crate derive_more;

#[derive(
    From,
    FromStr,
    Display,
    Index,
    Not,
    Add,
    Mul,
    Sum,
    IndexMut,
    AddAssign,
    Deref,
    DerefMut,
    IntoIterator,
    IntoIteratorRef,
    IntoIteratorRefMut,
    Constructor
)]
#[deref(forward)]
#[deref_mut(forward)]
struct Wrapped<T: Clone>(T);

#[derive(Iterator, Deref, DerefMut)]
struct Wrapped2<T: Clone>(T);

#[derive(From, Not, Add, Mul, AddAssign, Constructor, Sum)]
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
    Deref,
    DerefMut,
    IntoIterator,
    IntoIteratorRef,
    IntoIteratorRefMut,
    Constructor,
    Sum
)]
struct Struct<T: Clone> {
    t: T,
}

#[derive(Iterator, Deref, DerefMut)]
struct Struct2<T: Clone> {
    t: T,
}

#[derive(From, Not, Add, Mul, AddAssign, Constructor, Sum)]
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
