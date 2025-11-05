#![cfg_attr(not(feature = "std"), no_std)]
#![allow(dead_code)] // some code is tested for type checking only

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

mod ignore {
    use core::marker::PhantomData;

    use super::*;

    #[test]
    fn tuple() {
        #[derive(Add)]
        struct TupleWithZst<T = ()>(i32, #[add(ignore)] PhantomData<T>);

        let a: TupleWithZst = TupleWithZst(12, PhantomData);
        let b: TupleWithZst = TupleWithZst(2, PhantomData);

        assert_eq!((a + b).0, 14);
    }

    #[test]
    fn struct_() {
        #[derive(Add)]
        struct StructWithZst<T = String> {
            x: i32,
            #[add(skip)]
            _marker: PhantomData<T>,
        }

        let a: StructWithZst = StructWithZst {
            x: 12,
            _marker: PhantomData,
        };
        let b: StructWithZst = StructWithZst {
            x: 2,
            _marker: PhantomData,
        };

        assert_eq!((a + b).x, 14);
    }
}
