#![allow(dead_code, unused_imports)]
#[macro_use]
extern crate derive_more;

use std::collections::{BTreeMap, HashMap};

#[derive(IndexMut)]
struct MyVec(Vec<i32>);
//Index implementation is required for IndexMut
impl<__IdxT, __IdxOutputT: ?::core::marker::Sized> ::core::ops::Index<__IdxT> for MyVec
where
    Vec<i32>: ::core::ops::Index<__IdxT, Output = __IdxOutputT>,
{
    type Output = __IdxOutputT;
    #[inline]
    fn index(&self, idx: __IdxT) -> &Self::Output {
        let indexable = &self.0;
        <Vec<i32> as ::core::ops::Index<__IdxT>>::index(indexable, idx)
    }
}

#[derive(IndexMut)]
struct Numbers {
    #[index_mut]
    numbers: Vec<i32>,
    useless: bool,
}

//Index implementation is required for IndexMut
impl<__IdxT> ::core::ops::Index<__IdxT> for Numbers
where
    Vec<i32>: ::core::ops::Index<__IdxT>,
{
    type Output = <Vec<i32> as ::core::ops::Index<__IdxT>>::Output;
    #[inline]
    fn index(&self, idx: __IdxT) -> &Self::Output {
        <Vec<i32> as ::core::ops::Index<__IdxT>>::index(&self.numbers, idx)
    }
}

#[derive(IndexMut)]
enum MyVecs {
    MyVecVariant(MyVec),
    NumbersVariant(Numbers),
}
impl<__IdxT, __IdxOutputT: ?::core::marker::Sized> ::core::ops::Index<__IdxT> for MyVecs
where
    Numbers: ::core::ops::Index<__IdxT, Output = __IdxOutputT>,
    MyVec: ::core::ops::Index<__IdxT, Output = __IdxOutputT>,
{
    type Output = __IdxOutputT;
    #[inline]
    fn index(&self, idx: __IdxT) -> &Self::Output {
        match self {
            MyVecs::MyVecVariant(indexable) => {
                <MyVec as ::core::ops::Index<__IdxT>>::index(indexable, idx)
            }
            MyVecs::NumbersVariant(indexable) => {
                <Numbers as ::core::ops::Index<__IdxT>>::index(indexable, idx)
            }
        }
    }
}

#[test]
fn enum_index() {
    let v = MyVecs::MyVecVariant(MyVec(vec![10, 20, 30]));
    assert_eq!(10, v[0]);
    let mut nums = MyVecs::NumbersVariant(Numbers {
        numbers: vec![100, 200, 300],
        useless: false,
    });
    assert_eq!(200, nums[1]);
    nums[2] = 400;
    assert_eq!(400, nums[2]);
}

#[derive(IndexMut)]
enum SomeMapNames {
    Hash {
        h: HashMap<i32, u64>,
        #[index_mut(ignore)]
        useless: bool,
    },
    BTree {
        b: BTreeMap<i32, u64>,
    },
}
//Index implementation is required for IndexMut
impl<__IdxT, __IdxOutputT: ?::core::marker::Sized> ::core::ops::Index<__IdxT>
    for SomeMapNames
where
    BTreeMap<i32, u64>: ::core::ops::Index<__IdxT, Output = __IdxOutputT>,
    HashMap<i32, u64>: ::core::ops::Index<__IdxT, Output = __IdxOutputT>,
{
    type Output = __IdxOutputT;
    #[inline]
    fn index(&self, idx: __IdxT) -> &Self::Output {
        match self {
            SomeMapNames::Hash {
                h: indexable,
                useless: _,
            } => {
                <HashMap<i32, u64> as ::core::ops::Index<__IdxT>>::index(indexable, idx)
            }
            SomeMapNames::BTree { b: indexable } => {
                <BTreeMap<i32, u64> as ::core::ops::Index<__IdxT>>::index(
                    indexable, idx,
                )
            }
        }
    }
}
