#![allow(dead_code, unused_imports)]
#[macro_use]
extern crate derive_more;

#[derive(Index, IndexMut)]
struct MyVec(Vec<i32>);

#[derive(Index, IndexMut)]
struct Numbers {
    numbers: Vec<i32>,
}
