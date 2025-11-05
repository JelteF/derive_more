#![cfg_attr(not(feature = "std"), no_std)]
#![allow(dead_code)] // some code is tested for type checking only

use derive_more::AddAssign;

#[derive(AddAssign)]
struct MyInts(i32, i32);

#[derive(AddAssign)]
struct Point2D {
    x: i32,
    y: i32,
}

mod ignore {
    use core::marker::PhantomData;

    use super::*;

    #[test]
    fn tuple() {
        #[derive(AddAssign)]
        struct TupleWithZst<T = ()>(i32, #[add_assign(ignore)] PhantomData<T>);

        let mut a: TupleWithZst = TupleWithZst(12, PhantomData);
        a += TupleWithZst(2, PhantomData);

        assert_eq!(a.0, 14);
    }

    #[test]
    fn struct_() {
        #[derive(AddAssign)]
        struct StructWithZst<T = String> {
            x: i32,
            #[add_assign(skip)]
            _marker: PhantomData<T>,
        }

        let mut a: StructWithZst = StructWithZst {
            x: 12,
            _marker: PhantomData,
        };
        let b: StructWithZst = StructWithZst {
            x: 2,
            _marker: PhantomData,
        };
        a += b;

        assert_eq!(a.x, 14);
    }
}
