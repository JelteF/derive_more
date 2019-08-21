#![allow(dead_code)]
#[macro_use]
extern crate derive_more;

#[derive(Into, IntoRef, IntoRefMut)]
struct EmptyTuple();

#[derive(Into, IntoRef, IntoRefMut)]
struct EmptyStruct {}

#[derive(Into, IntoRef, IntoRefMut)]
struct EmptyUnit;

#[derive(Into, IntoRef, IntoRefMut)]
struct MyInt(i32);

#[derive(Into, IntoRef, IntoRefMut)]
struct MyInts(i32, i32);

#[derive(Into, IntoRef, IntoRefMut)]
struct Point1D {
    x: i32,
}

#[derive(Into, IntoRef, IntoRefMut)]
struct Point2D {
    x: i32,
    y: i32,
}
