#![allow(dead_code, unused_imports)]
#[macro_use]
extern crate derive_more;

#[derive(Deref, DerefMut)]
struct MyBoxedInt(Box<i32>);

#[derive(Deref, DerefMut)]
struct NumRef<'a> {
    num: &'a mut i32,
}
