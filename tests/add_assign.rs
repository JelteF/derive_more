#![cfg_attr(not(feature = "std"), no_std)]
#![allow(dead_code)] // some code is tested for type checking only

use core::marker::PhantomData;

use derive_more::AddAssign;

#[derive(AddAssign)]
struct MyInts(i32, i32);

#[derive(AddAssign)]
struct Point2D {
    x: i32,
    y: i32,
}

#[derive(AddAssign)]
struct TupleWithZst<T>(i32, #[add_assign(skip)] PhantomData<T>);

#[derive(AddAssign)]
struct StructWithZst<T> {
    x: i32,
    #[add_assign(skip)]
    _marker: PhantomData<T>,
}

mod skip {
    use super::*;

    #[test]
    fn tuple_non_add_generic() {
        let mut a: TupleWithZst<(String,)> = TupleWithZst(12, PhantomData);
        a += TupleWithZst(2, PhantomData);
        assert_eq!(a.0, 14);
    }

    #[test]
    fn struct_non_add_generic() {
        let mut a: StructWithZst<(String,)> = StructWithZst {
            x: 12,
            _marker: PhantomData,
        };

        let b: StructWithZst<(String,)> = StructWithZst {
            x: 2,
            _marker: PhantomData,
        };

        a += b;
        assert_eq!(a.x, 14);
    }
}
