#![allow(dead_code)]
#[macro_use]
extern crate derive_more;

#[derive(From)]
#[from(types(u8, u16))]
struct Some(u32, i16);
