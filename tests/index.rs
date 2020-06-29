#![allow(dead_code, unused_imports)]
#[macro_use]
extern crate derive_more;

use std::collections::{BTreeMap, HashMap};

#[derive(Index)]
struct MyVec(Vec<i32>);

#[derive(Index)]
struct Numbers {
    #[index]
    numbers: Vec<i32>,
    useless: bool,
}

#[derive(Index)]
enum MyVecs {
    MyVecVariant(MyVec),
    NumbersVariant(Numbers),
}

#[test]
fn enum_index() {
    let v = MyVecs::MyVecVariant(MyVec(vec![10, 20, 30]));
    assert_eq!(10, v[0]);
    let nums = MyVecs::NumbersVariant(Numbers {
        numbers: vec![100, 200, 300],
        useless: false,
    });
    assert_eq!(200, nums[1]);
}

#[derive(Index)]
enum SomeMapNames {
    Hash {
        h: HashMap<i32, u64>,
        #[index(ignore)]
        useless: bool,
    },
    BTree {
        b: BTreeMap<i32, u64>,
    },
}
