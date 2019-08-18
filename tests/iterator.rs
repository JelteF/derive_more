#![allow(dead_code, unused_imports)]
#[macro_use]
extern crate derive_more;

#[derive(Iterator)]
struct MyVec<'a>(::core::slice::Iter<'a, i32>);

#[derive(Iterator)]
struct Numbers<'a> {
    numbers: ::core::slice::Iter<'a, i32>,
}
