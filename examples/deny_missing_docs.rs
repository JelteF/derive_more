#![deny(missing_docs)]
//! Some docs

#[macro_use]
extern crate derive_more;

fn main() {}

/// Some docs
#[cfg_attr(feature = "from", derive(From))]
#[cfg_attr(feature = "into", derive(Into))]
#[cfg_attr(feature = "constructor", derive(Constructor))]
#[cfg_attr(feature = "add_like", derive(Add))]
#[cfg_attr(feature = "mul_like", derive(Mul))]
#[cfg_attr(feature = "not_like", derive(Neg))]
#[cfg_attr(feature = "add_assign_like", derive(AddAssign))]
#[cfg_attr(feature = "mul_assign_like", derive(MulAssign))]
#[derive(Eq, PartialEq, Debug)]
pub struct MyInt(i32);
