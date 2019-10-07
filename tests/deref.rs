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

#[derive(Deref)]
#[deref(forward)]
struct MyInt(i32);

#[derive(Deref)]
#[deref(forward)]
struct Point1D {
    x: i32,
}

#[derive(Deref)]
#[deref(forward)]
struct Point1D2 {
    x: i32,
    #[deref(ignore)]
    useless: bool,
}

#[derive(Deref)]
struct Point1D3 {
    #[deref(forward)]
    x: i32,
    useless: bool,
}
