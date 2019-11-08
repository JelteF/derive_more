#![allow(dead_code)]
#[macro_use]
extern crate derive_more;

#[derive(Into)]
#[into(owned, ref, ref_mut)]
struct EmptyTuple();

#[derive(Into)]
#[into(owned, ref, ref_mut)]
struct EmptyStruct {}

#[derive(Into)]
#[into(owned, ref, ref_mut)]
struct EmptyUnit;

#[derive(Into)]
#[into(owned, ref, ref_mut)]
struct MyInt(i32);

#[derive(Into)]
#[into(owned, ref, ref_mut)]
struct MyInts(i32, i32);

#[derive(Into)]
#[into(owned, ref, ref_mut)]
struct Point1D {
    x: i32,
}

#[derive(Into)]
#[into(owned, ref, ref_mut)]
struct Point2D {
    x: i32,
    y: i32,
}

#[derive(Into)]
#[into(owned, ref, ref_mut)]
struct Point2DWithIgnored {
    x: i32,
    y: i32,
    #[into(ignore)]
    useless: bool,
}
