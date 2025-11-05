#![cfg_attr(not(feature = "std"), no_std)]
#![allow(dead_code)] // some code is tested for type checking only

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

mod ignore {
    use core::marker::PhantomData;

    use super::*;

    #[test]
    fn tuple() {
        #[derive(Mul)]
        struct TupleWithZst<T = ()>(i32, #[mul(ignore)] PhantomData<T>);

        let a: TupleWithZst = TupleWithZst(12, PhantomData);
        let b: TupleWithZst = TupleWithZst(2, PhantomData);

        assert_eq!((a * b).0, 24);
    }

    #[test]
    fn struct_() {
        #[derive(Mul)]
        struct StructWithZst<T = String> {
            x: i32,
            #[mul(skip)]
            _marker: PhantomData<T>,
        }

        let a: StructWithZst<()> = StructWithZst {
            x: 12,
            _marker: PhantomData,
        };
        let b: StructWithZst<()> = StructWithZst {
            x: 2,
            _marker: PhantomData,
        };

        assert_eq!((a * b).x, 24);
    }
}
