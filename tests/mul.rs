#![cfg_attr(not(feature = "std"), no_std)]
#![allow(dead_code)] // some code is tested for type checking only

use core::marker::PhantomData;

use derive_more::Mul;

#[derive(Mul)]
struct MyInt(i32);

#[derive(Mul)]
struct MyInts(i32, i32);

#[derive(Mul)]
struct Point1D {
    x: i32,
}

#[derive(Mul)]
struct Point2D {
    x: i32,
    y: i32,
}

#[derive(Mul)]
#[mul(forward)]
struct TupleWithZst<T>(i32, #[mul(skip)] PhantomData<T>);

#[derive(Mul)]
#[mul(forward)]
struct StructWithZst<T> {
    x: i32,
    #[mul(skip)]
    _marker: PhantomData<T>,
}

mod forward {
    use super::*;

    #[test]
    fn tuple_non_add_generic() {
        let a: TupleWithZst<(String,)> = TupleWithZst(12, PhantomData);
        let b: TupleWithZst<(String,)> = TupleWithZst(2, PhantomData);
        assert_eq!((a * b).0, 24);
    }

    #[test]
    fn struct_non_add_generic() {
        let a: StructWithZst<(String,)> = StructWithZst {
            x: 12,
            _marker: PhantomData,
        };

        let b: StructWithZst<(String,)> = StructWithZst {
            x: 2,
            _marker: PhantomData,
        };

        assert_eq!((a * b).x, 24);
    }
}
