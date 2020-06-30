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
impl<__IdxT, __IdxOutputT: ?::core::marker::Sized> ::core::ops::Index<__IdxT>
    for Numbers
where
    Vec<i32>: ::core::ops::Index<__IdxT, Output = __IdxOutputT>,
{
    type Output = __IdxOutputT;
    #[inline]
    fn index(&self, idx: __IdxT) -> &Self::Output {
        let indexable = &self.numbers;
        <Vec<i32> as ::core::ops::Index<__IdxT>>::index(indexable, idx)
    }
}

#[derive(IndexMut)]
enum MyVecs {
    MyVec(Vec<i32>),
    Numbers {
        #[index_mut]
        numbers: Vec<i32>,
        useless: bool,
    },
}

//Index implementation is required for IndexMut
impl<__IdxT, __IdxOutputT: ?::core::marker::Sized> ::core::ops::Index<__IdxT> for MyVecs
where
    Vec<i32>: ::core::ops::Index<__IdxT, Output = __IdxOutputT>,
    Vec<i32>: ::core::ops::Index<__IdxT, Output = __IdxOutputT>,
{
    type Output = __IdxOutputT;
    #[inline]
    fn index(&self, idx: __IdxT) -> &Self::Output {
        match self {
            MyVecs::MyVec(indexable) => {
                <Vec<i32> as ::core::ops::Index<__IdxT>>::index(indexable, idx)
            }
            MyVecs::Numbers {
                numbers: indexable,
                useless: _,
            } => <Vec<i32> as ::core::ops::Index<__IdxT>>::index(indexable, idx),
        }
    }
}

#[test]
fn enum_index() {
    let v = MyVecs::MyVec(vec![10, 20, 30]);
    assert_eq!(10, v[0]);
    let mut nums = MyVecs::Numbers {
        numbers: vec![100, 200, 300],
        useless: false,
    };
    assert_eq!(200, nums[1]);
    nums[2] = 400;
    assert_eq!(400, nums[2]);
}

#[derive(IndexMut)]
enum MyVecTypes {
    MyVecVariant(MyVec),
    NumbersVariant(Numbers),
}

//Index implementation is required for IndexMut
impl<__IdxT, __IdxOutputT: ?::core::marker::Sized> ::core::ops::Index<__IdxT>
    for MyVecTypes
where
    Numbers: ::core::ops::Index<__IdxT, Output = __IdxOutputT>,
    MyVec: ::core::ops::Index<__IdxT, Output = __IdxOutputT>,
{
    type Output = __IdxOutputT;
    #[inline]
    fn index(&self, idx: __IdxT) -> &Self::Output {
        match self {
            MyVecTypes::MyVecVariant(indexable) => {
                <MyVec as ::core::ops::Index<__IdxT>>::index(indexable, idx)
            }
            MyVecTypes::NumbersVariant(indexable) => {
                <Numbers as ::core::ops::Index<__IdxT>>::index(indexable, idx)
            }
        }
    }
}

#[test]
fn enum_index2() {
    let mut v = MyVecTypes::MyVecVariant(MyVec(vec![10, 20, 30]));
    v[0] = 100;
    assert_eq!(100, v[0]);
}
