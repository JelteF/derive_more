#![allow(dead_code)]
#[macro_use]
extern crate derive_more;

#[derive(MulAssign)]
struct MyInts(i32, i32);

#[derive(MulAssign)]
struct Point2D {
    x: i32,
    y: i32,
}
