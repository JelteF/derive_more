#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(nightly, feature(never_type))]
#![allow(dead_code)] // some code is tested for type checking only

#[cfg(not(feature = "std"))]
extern crate alloc;

#[cfg(not(feature = "std"))]
use ::alloc::{boxed::Box, vec::Vec};

use derive_more::Deref;

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
struct CoolVec {
    cool: bool,
    #[deref]
    vec: Vec<i32>,
}

#[derive(Deref)]
struct GenericVec<T>(Vec<T>);

#[test]
fn deref_generic() {
    let gv = GenericVec(Vec::<i32>::new());
    assert!(gv.is_empty())
}

#[derive(Deref)]
struct GenericBox<T>(#[deref(forward)] Box<T>);

#[test]
fn deref_generic_forward() {
    let boxed = GenericBox(Box::new(1i32));
    assert_eq!(*boxed, 1i32);
}

#[cfg(nightly)]
mod never {
    use super::*;

    #[derive(Deref)]
    struct Tuple(!);

    #[derive(Deref)]
    struct Struct {
        field: !,
    }
}

#[derive(Deref)]
enum MyEnum<'a> {
    Variant1(&'a [u8]),
    Variant2(&'a [u8]),
    Variant3(&'a [u8]),
}

#[derive(Deref)]
enum Compression {
    Stored(u32),
    Zlib(#[deref(forward)] Box<u32>),
    LZMA1(u32),
}

#[test]
fn deref_enum() {
    let e = Compression::Stored(5);
    assert_eq!(*e, 5);
}

#[derive(Deref)]
enum MyEnum1 {
    Variant1 {
        #[deref]
        named_field1: u8,
        named_field2: bool,
    },
    Variant2 {
        named_field1: u8,
        #[deref(ignore)]
        named_field2: bool,
    },
}

#[derive(Deref)]
enum MyEnum2 {
    Variant1(u8, #[deref(ignore)] bool),
    Variant2(#[deref] u8, bool),
}

mod deprecated {
    use super::*;

    #[derive(Deref)]
    #[deprecated(note = "struct")]
    struct Tuple(#[deprecated(note = "field")] i32);

    #[derive(Deref)]
    #[deprecated(note = "struct")]
    struct Struct {
        #[deprecated(note = "field")]
        field: i32,
    }
}
