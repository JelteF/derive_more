#![allow(dead_code, unused_imports)]
#[macro_use]
extern crate derive_more;

use std::io::{empty, repeat, Empty, Read, Repeat};

#[derive(Read)]
struct MyEmpty(Empty);

#[derive(Read)]
struct MyRepeat {
    #[read]
    endless: Repeat,
    useless: bool,
}

#[derive(Read)]
enum MyReaders {
    MyEmpty(Empty),
    MyRepeat {
        #[read]
        endless: Repeat,
        useless: bool,
    },
}

#[test]
fn main() {
    let mut buffer = [0; 2];
    assert_eq!(0, MyEmpty(empty()).read(&mut buffer).unwrap());
    assert_eq!(
        2,
        MyRepeat {
            endless: repeat(0xaa),
            useless: false
        }
        .read(&mut buffer)
        .unwrap()
    );
    assert_eq!(0, MyReaders::MyEmpty(empty()).read(&mut buffer).unwrap());
}
