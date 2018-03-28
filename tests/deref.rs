#![allow(dead_code, unused_imports)]
#[macro_use]
extern crate derive_more;

#[derive(Deref)]
struct MyBoxedInt(Box<i32>);

#[derive(Deref)]
struct NumRef<'a> {
    num: &'a i32,
}
