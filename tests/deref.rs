#![allow(dead_code, unused_imports)]
#[macro_use]
extern crate derive_more;

#[derive(Deref)]
#[deref(forward)]
struct MyBoxedInt(Box<i32>);

#[derive(Deref)]
#[deref(forward)]
struct NumRef<'a> {
    num: &'a i32,
}

#[derive(Deref)]
struct NumRef2<'a> {
    #[deref(forward)]
    num: &'a i32,
    useless: bool,
}

#[derive(Deref)]
#[deref(forward)]
struct NumRef3<'a> {
    num: &'a i32,
    #[deref(ignore)]
    useless: bool,
}

#[derive(Deref)]
struct MyInt(i32);

#[derive(Deref)]
struct Point1D {
    x: i32,
}

#[derive(Deref)]
struct Point1D2 {
    x: i32,
    #[deref(ignore)]
    useless: bool,
}

#[derive(Deref)]
struct Point1D3 {
    #[deref]
    x: i32,
    useless: bool,
}
