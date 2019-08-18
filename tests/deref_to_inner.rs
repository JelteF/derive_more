#![allow(dead_code)]
#[macro_use]
extern crate derive_more;

#[derive(DerefToInner)]
struct MyInt(i32);

#[derive(DerefToInner)]
struct Point1D {
    x: i32,
}
