#![allow(dead_code)]

#[macro_use]
extern crate derive_more;

use std::borrow::Borrow;

use std::path::PathBuf;
use std::ptr;

#[derive(Borrow)]
struct SingleFieldTuple(String);

#[test]
fn single_field_tuple() {
    let item = SingleFieldTuple(String::from("test"));

    assert!(ptr::eq(&item.0, item.borrow()));
}

#[derive(Borrow)]
#[borrow(forward)]
struct SingleFieldForward(Vec<i32>);

#[test]
fn single_field_forward() {
    let item = SingleFieldForward(vec![]);
    let _: &[i32] = (&item).borrow();
}

#[derive(Borrow)]
struct SingleFieldStruct {
    first: String,
}

#[test]
fn single_field_struct() {
    let item = SingleFieldStruct {
        first: String::from("test"),
    };

    assert!(ptr::eq(&item.first, item.borrow()));
}

#[derive(Borrow)]
struct MultiFieldTuple(#[borrow] String, #[borrow] PathBuf, Vec<usize>);

#[test]
fn multi_field_tuple() {
    let item = MultiFieldTuple(String::from("test"), PathBuf::new(), vec![]);

    assert!(ptr::eq(&item.0, item.borrow()));
    assert!(ptr::eq(&item.1, item.borrow()));
}

#[derive(Borrow)]
struct MultiFieldStruct {
    #[borrow]
    first: String,
    #[borrow]
    second: PathBuf,
    third: Vec<usize>,
}

#[test]
fn multi_field_struct() {
    let item = MultiFieldStruct {
        first: String::from("test"),
        second: PathBuf::new(),
        third: vec![],
    };

    assert!(ptr::eq(&item.first, item.borrow()));
    assert!(ptr::eq(&item.second, item.borrow()));
}

#[derive(Borrow)]
struct SingleFieldGenericStruct<T> {
    first: T,
}

#[test]
fn single_field_generic_struct() {
    let item = SingleFieldGenericStruct {
        first: String::from("test"),
    };

    assert!(ptr::eq(&item.first, item.borrow()));
}

#[derive(Borrow)]
struct MultiFieldGenericStruct<T, U> {
    #[borrow]
    first: Vec<T>,
    #[borrow]
    second: [U; 2],
    third: Vec<usize>,
}

#[test]
fn multi_field_generic_struct() {
    let item = MultiFieldGenericStruct {
        first: b"test".to_vec(),
        second: [0i32, 1i32],
        third: vec![],
    };

    assert!(ptr::eq(&item.first, item.borrow()));
    assert!(ptr::eq(&item.second, item.borrow()));
}
