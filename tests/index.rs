#![allow(dead_code, unused_imports)]
#[macro_use]
extern crate derive_more;

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
    MyVec(Vec<i32>),
    Numbers {
        #[index]
        numbers: Vec<i32>,
        useless: bool,
    },
}

#[test]
fn enum_index() {
    let v = MyVecs::MyVec(vec![10, 20, 30]);
    assert_eq!(10, v[0]);
    let nums = MyVecs::Numbers {
        numbers: vec![100, 200, 300],
        useless: false,
    };
    assert_eq!(200, nums[1]);
}

#[derive(Index)]
enum MyVecTypes {
    MyVecVariant(MyVec),
    NumbersVariant(Numbers),
}

#[test]
fn enum_index2() {
    let v = MyVecTypes::MyVecVariant(MyVec(vec![10, 20, 30]));
    assert_eq!(10, v[0]);
    let nums = MyVecTypes::NumbersVariant(Numbers {
        numbers: vec![100, 200, 300],
        useless: false,
    });
    assert_eq!(200, nums[1]);
}
