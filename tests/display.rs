#![allow(dead_code)]
#[macro_use]
extern crate derive_more;

#[derive(Display)]
struct MyInt(i32);

#[derive(Display)]
struct Point1D {
    x: i32,
}
