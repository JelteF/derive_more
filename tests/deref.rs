#![allow(dead_code, unused_imports)]
#[macro_use]
extern crate derive_more;

#[derive(Deref)]
struct MyBoxedInt(Box<i32>);

#[derive(Deref)]
struct NumRef<'a> {
    num: &'a i32,
}

#[derive(Deref)]
struct NumRef2<'a> {
    num: &'a i32,
    #[deref(ignore)]
    useless: bool,
}
