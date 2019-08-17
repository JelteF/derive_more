#![no_std]

#[macro_use]
extern crate derive_more;

#[derive(
    AddAssign,
    MulAssign,
    Add,
    Mul,
    Not,
    Index,
    Display,
    FromStr,
    Into,
    From,
    IndexMut,
    Sum,
    Constructor
)]
struct MyInts(u64);

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
    Constructor
)]
struct Wrapped<T: Clone>(T);

#[derive(From, Not, Add, Mul, AddAssign, Constructor, Sum)]
struct WrappedDouble<T: Clone, U: Clone>(T, U);
