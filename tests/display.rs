#![allow(dead_code, unused_imports)]
#[macro_use]
extern crate derive_more;
use std::fmt::Binary; // brought in scope to be sure that we don't get compile errors

#[derive(Display)]
struct MyInt(i32);

#[derive(Display)]
struct Point1D {
    x: i32,
}
