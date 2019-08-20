#![allow(dead_code, unused_imports)]
#[macro_use]
extern crate derive_more;

#[derive(IntoIterator, IntoIteratorRef)]
struct MyVec(Vec<i32>);

#[derive(IntoIterator, IntoIteratorRef)]
struct Numbers {
    numbers: Vec<i32>,
}
