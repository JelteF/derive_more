#![allow(dead_code, unused_imports)]
#[macro_use]
extern crate derive_more;

#[derive(Index)]
struct MyVec(Vec<i32>);

#[derive(Index)]
struct Numbers {
    numbers: Vec<i32>,
}
