#![allow(dead_code, unused_imports)]
#[macro_use]
extern crate derive_more;

use std::collections::{BTreeMap, HashMap};
#[derive(Index)]
struct Hash {
    #[index]
    h: HashMap<i32, u64>,
    useless: bool,
}
#[derive(Index)]
enum SomeMapNames {
    Hash {
        #[index]
        h: HashMap<i32, u64>,
        useless: bool,
    },
    BTree {
        b: BTreeMap<i32, u64>,
    },
}
