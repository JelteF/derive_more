#![deny(missing_docs)]
//! Some docs

#[macro_use]
extern crate derive_more;

fn main() {}

/// Some docs
#[derive(From)]
#[derive(Into)]
#[derive(Constructor)]
#[derive(Eq, PartialEq, Debug)]
#[derive(Add)]
#[derive(Mul)]
#[derive(Neg)]
#[derive(AddAssign)]
#[derive(MulAssign)]
pub struct MyInt(i32);
