#![allow(dead_code)]
#[macro_use]
extern crate derive_more;

#[derive(Into)]
struct EmptyTuple();

#[derive(Into)]
struct EmptyStruct {}

#[derive(Into)]
struct EmptyUnit;

#[derive(Into)]
struct MyInt(i32);

#[derive(Into)]
struct MyInts(i32, i32);

#[derive(Into)]
struct Point1D {
    x: i32,
}

#[derive(Into)]
struct Point2D {
    x: i32,
    y: i32,
}
