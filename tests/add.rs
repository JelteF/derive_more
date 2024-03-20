#![cfg_attr(not(feature = "std"), no_std)]
#![allow(dead_code)]

use derive_more::Add;

#[derive(Add)]
struct MyInts(i32, i32);

#[derive(Add)]
struct Point2D {
    x: i32,
    y: i32,
}

#[derive(Add)]
enum MixedInts {
    SmallInt(i32),
    BigInt(i64),
    TwoSmallInts(i32, i32),
    NamedSmallInts { x: i32, y: i32 },
    UnsignedOne(u32),
    UnsignedTwo(u32),
    Unit,
}

#[derive(Add)]
#[derive(Default)]
struct StructRecursive {
    a: i32,
    b: [i32; 2],
    c: [[i32; 2]; 3],
    d: (i32, i32),
    e: ((u8, [i32; 3]), i32),
    f: ((u8, i32), (u8, ((i32, u64, ((u8, u8), u16)), u8))),
    g: i32,
}

#[test]
fn test_sanity() {
    let mut a: StructRecursive = Default::default();
    let mut b: StructRecursive = Default::default();
    a.c[0][1] = 1;
    b.c[0][1] = 2;
    let c = a + b;
    assert_eq!(c.c[0][1], 3);
}

#[derive(Add)]
struct TupleRecursive((i32, u8), [(i32, u8); 10]);

#[derive(Add)]
pub struct GenericArrayStruct<T> {
    pub a: [T; 2],
}
