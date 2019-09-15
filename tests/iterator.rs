#![allow(dead_code, unused_imports)]
#[macro_use]
extern crate derive_more;

#[derive(Iterator)]
struct MyVec<'a>(::core::slice::Iter<'a, i32>);

#[derive(Iterator)]
struct Numbers<'a> {
    numbers: ::core::slice::Iter<'a, i32>,
}

#[derive(Iterator)]
struct Numbers2<'a> {
    numbers: ::core::slice::Iter<'a, i32>,
    #[iterator(ignore)]
    useless: bool,
}

#[derive(Iterator)]
struct Numbers3<'a> {
    #[iterator]
    numbers: ::core::slice::Iter<'a, i32>,
    useless: bool,
    useless2: bool,
}
