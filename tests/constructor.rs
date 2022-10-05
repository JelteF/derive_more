#![allow(dead_code)]

use derive_more::Constructor;

#[derive(Constructor)]
struct EmptyTuple();

#[derive(Constructor)]
struct EmptyStruct {}

#[derive(Constructor)]
struct EmptyUnit;

#[derive(Constructor)]
struct MyInts(i32, i32);

#[derive(Constructor)]
struct Point2D {
    x: i32,
    y: i32,
}
