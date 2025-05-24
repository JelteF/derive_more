#![cfg_attr(not(feature = "std"), no_std)]
#![allow(dead_code)] // some code is tested for type checking only

use core::marker::PhantomData;

use derive_more::MulAssign;

#[derive(MulAssign)]
struct MyInt(i32);

#[derive(MulAssign)]
struct MyInts(i32, i32);

#[derive(MulAssign)]
#[mul_assign(forward)]
struct MyIntForward(i32);

#[derive(MulAssign)]
struct Point1D {
    x: i32,
}

#[derive(MulAssign)]
struct Point2D {
    x: i32,
    y: i32,
}

#[derive(MulAssign)]
struct MyInt2<T> {
    x: i32,
    ph: PhantomData<T>,
}

#[derive(MulAssign)]
#[mul_assign(forward)]
struct TupleWithZst<T>(i32, #[mul_assign(skip)] PhantomData<T>);

#[derive(MulAssign)]
#[mul_assign(forward)]
struct StructWithZst<T> {
    x: i32,
    #[mul_assign(skip)]
    _marker: PhantomData<T>,
}

mod forward {
    use super::*;

    #[test]
    fn tuple_non_add_generic() {
        let mut a: TupleWithZst<()> = TupleWithZst(12, PhantomData);
        a *= TupleWithZst(2, PhantomData);
        assert_eq!(a.0, 24);
    }

    #[test]
    fn struct_non_add_generic() {
        let mut a: StructWithZst<()> = StructWithZst {
            x: 12,
            _marker: PhantomData,
        };

        let b: StructWithZst<()> = StructWithZst {
            x: 2,
            _marker: PhantomData,
        };

        a *= b;
        assert_eq!(a.x, 24);
    }
}
