#![allow(dead_code, unused_imports)]
#[macro_use]
extern crate derive_more;

#[derive(Deref, DerefMut)]
struct MyBoxedInt(Box<i32>);

#[derive(Deref)]
struct NamedBoxedNumber {
    numbers: Box<i32>,
}
