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
enum SomeMap {
    Hash(HashMap<i32, u64>),
    BTree(BTreeMap<i32, u64>),
}

#[test]
fn enum_index() {
    let mut hmap = HashMap::new();
    hmap.insert(123, 456);
    assert_eq!(456, SomeMap::Hash(hmap)[&123]);
    let mut bmap = BTreeMap::new();
    bmap.insert(4, 8);
    assert_eq!(8, SomeMap::BTree(bmap)[&4]);
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
