#![no_std]

#[macro_use]
extern crate derive_more;

#[derive(AddAssign, MulAssign, Add, Mul, Not, Index, Display, FromStr, Into, From, IndexMut, Constructor)]
struct MyInts(u64);
