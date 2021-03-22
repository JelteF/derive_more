#![allow(dead_code)]

#[macro_use]
extern crate derive_more;

use std::borrow::{Borrow, BorrowMut};

use std::path::PathBuf;
use std::ptr;

#[derive(Borrow, BorrowMut)]
struct SingleFieldTuple(String);

#[test]
fn single_field_tuple() {
    let mut item = SingleFieldTuple(String::from("test"));

    assert!(ptr::eq(&mut item.0, item.borrow_mut()));
}

#[derive(Borrow, BorrowMut)]
#[borrow(forward)]
#[borrow_mut(forward)]
struct SingleFieldForward(Vec<i32>);

#[test]
fn single_field_forward() {
    let mut item = SingleFieldForward(vec![]);
    let _: &mut [i32] = (&mut item).borrow_mut();
}

#[derive(Borrow, BorrowMut)]
struct SingleFieldStruct {
    first: String,
}

#[test]
fn single_field_struct() {
    let mut item = SingleFieldStruct {
        first: String::from("test"),
    };

    assert!(ptr::eq(&mut item.first, item.borrow_mut()));
}

#[derive(Borrow, BorrowMut)]
struct MultiFieldTuple(#[borrow_mut] String, #[borrow_mut] PathBuf, Vec<usize>);

#[test]
fn multi_field_tuple() {
    let mut item = MultiFieldTuple(String::from("test"), PathBuf::new(), vec![]);

    assert!(ptr::eq(&mut item.0, item.borrow_mut()));
    assert!(ptr::eq(&mut item.1, item.borrow_mut()));
}

#[derive(Borrow, BorrowMut)]
struct MultiFieldStruct {
    #[borrow_mut]
    first: String,
    #[borrow_mut]
    second: PathBuf,
    third: Vec<usize>,
}

#[test]
fn multi_field_struct() {
    let mut item = MultiFieldStruct {
        first: String::from("test"),
        second: PathBuf::new(),
        third: vec![],
    };

    assert!(ptr::eq(&mut item.first, item.borrow_mut()));
    assert!(ptr::eq(&mut item.second, item.borrow_mut()));
}

#[derive(Borrow, BorrowMut)]
struct SingleFieldGenericStruct<T> {
    first: T,
}

#[test]
fn single_field_generic_struct() {
    let mut item = SingleFieldGenericStruct {
        first: String::from("test"),
    };

    assert!(ptr::eq(&mut item.first, item.borrow_mut()));
}

#[derive(Borrow, BorrowMut)]
struct MultiFieldGenericStruct<T> {
    #[borrow]
    #[borrow_mut]
    first: Vec<T>,
    #[borrow]
    #[borrow_mut]
    second: PathBuf,
    third: Vec<usize>,
}

#[test]
fn multi_field_generic_struct() {
    let mut item = MultiFieldGenericStruct {
        first: b"test".to_vec(),
        second: PathBuf::new(),
        third: vec![],
    };

    assert!(ptr::eq(&mut item.first, item.borrow_mut()));
    assert!(ptr::eq(&mut item.second, item.borrow_mut()));
}
