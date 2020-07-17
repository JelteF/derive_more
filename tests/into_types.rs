#![allow(dead_code)]
#[macro_use]
extern crate derive_more;

#[derive(Into)]
#[into(owned(types(i64)), ref(types(i64)), ref_mut(types(i128)))]
struct MyInt(i32);

/*
#[derive(Into)]
#[into(owned, ref, ref_mut)]
struct MyInts(i32, i32);
*/
