#![cfg_attr(not(feature = "std"), no_std)]
#![allow(dead_code)]

#[cfg(not(feature = "std"))]
extern crate alloc;

#[cfg(not(feature = "std"))]
use alloc::{borrow::ToOwned, string::String, vec, vec::Vec};
use core::ptr;

use derive_more::AsMut;

#[derive(AsMut)]
struct SingleFieldTuple(String);

#[test]
fn single_field_tuple() {
    let mut item = SingleFieldTuple("test".into());

    assert!(ptr::eq(&mut item.0, item.as_mut()));
}

#[derive(AsMut)]
struct SingleFieldGenericTuple<T>(T);

#[test]
fn single_field_generic_tuple() {
    let mut item = SingleFieldGenericTuple("test".to_owned());

    assert!(ptr::eq(&mut item.0, item.as_mut()));
}

#[derive(AsMut)]
#[as_mut(forward)]
struct SingleFieldTupleForward(Vec<i32>);

#[test]
fn single_field_tuple_forward() {
    let mut item = SingleFieldTupleForward(vec![]);

    let rf: &mut [i32] = item.as_mut();
    assert!(ptr::eq(rf, item.0.as_mut()));
}

#[derive(AsMut)]
#[as_mut(forward)]
struct SingleFieldGenericTupleForward<T>(T);

#[test]
fn single_field_generic_tuple_forward() {
    let mut item = SingleFieldGenericTupleForward("test".to_owned());

    let rf: &mut str = item.as_mut();
    assert!(ptr::eq(rf, item.0.as_mut()));
}

#[derive(AsMut)]
struct SingleFieldTupleFieldForward(#[as_mut(forward)] Vec<i32>);

#[test]
fn single_field_tuple_field_forward() {
    let mut item = SingleFieldTupleForward(vec![]);

    let rf: &mut [i32] = item.as_mut();
    assert!(ptr::eq(rf, item.0.as_mut()));
}

#[derive(AsMut)]
struct SingleFieldGenericTupleFieldForward<T>(#[as_mut(forward)] T);

#[test]
fn single_field_generic_tuple_field_forward() {
    let mut item = SingleFieldGenericTupleFieldForward("test".to_owned());

    let rf: &mut str = item.as_mut();
    assert!(ptr::eq(rf, item.0.as_mut()));
}

#[derive(AsMut)]
struct SingleFieldNamed {
    first: String,
}

#[test]
fn single_field_named() {
    let mut item = SingleFieldNamed {
        first: "test".into(),
    };

    assert!(ptr::eq(&mut item.first, item.as_mut()));
}

#[derive(AsMut)]
struct SingleFieldGenericNamed<T> {
    first: T,
}

#[test]
fn single_field_generic_named() {
    let mut item = SingleFieldGenericNamed { first: "test" };

    assert!(ptr::eq(&mut item.first, item.as_mut()));
}

#[derive(AsMut)]
#[as_mut(forward)]
struct SingleFieldNamedForward {
    first: String,
}

#[test]
fn single_field_named_forward() {
    let mut item = SingleFieldNamedForward {
        first: "test".into(),
    };

    let rf: &mut str = item.as_mut();
    assert!(ptr::eq(rf, item.first.as_mut()));
}

#[derive(AsMut)]
#[as_mut(forward)]
struct SingleFieldGenericNamedForward<T> {
    first: T,
}

#[test]
fn single_field_generic_named_forward() {
    let mut item = SingleFieldGenericNamedForward {
        first: "test".to_owned(),
    };

    let rf: &mut str = item.as_mut();
    assert!(ptr::eq(rf, item.first.as_mut()));
}

#[derive(AsMut)]
struct SingleFieldNamedFieldForward {
    #[as_mut(forward)]
    first: String,
}

#[test]
fn single_field_named_field_forward() {
    let mut item = SingleFieldNamedFieldForward {
        first: "test".into(),
    };

    let rf: &mut str = item.as_mut();
    assert!(ptr::eq(rf, item.first.as_mut()));
}

#[derive(AsMut)]
struct SingleFieldGenericNamedFieldForward<T> {
    #[as_mut(forward)]
    first: T,
}

#[test]
fn single_field_generic_named_field_forward() {
    let mut item = SingleFieldGenericNamedFieldForward {
        first: "test".to_owned(),
    };

    let rf: &mut str = item.as_mut();
    assert!(ptr::eq(rf, item.first.as_mut()));
}

#[derive(AsMut)]
struct AnnotatedTupleForward(#[as_mut(forward)] String, i32);

#[test]
fn annotated_tuple_forward() {
    let mut item = AnnotatedTupleForward("test".into(), 0);

    let rf: &mut str = item.as_mut();
    assert!(ptr::eq(rf, item.0.as_mut()));
}

#[derive(AsMut)]
struct AnnotatedGenericTupleForward<T>(#[as_mut(forward)] T, i32);

#[test]
fn annotated_generic_tuple_forward() {
    let mut item = AnnotatedGenericTupleForward("test".to_owned(), 0);

    let rf: &mut str = item.as_mut();
    assert!(ptr::eq(rf, item.0.as_mut()));
}

#[derive(AsMut)]
struct AnnotatedNamedForward {
    #[as_mut(forward)]
    first: String,
    second: i32,
}

#[test]
fn annotated_named_forward() {
    let mut item = AnnotatedNamedForward {
        first: "test".into(),
        second: 0,
    };

    let rf: &mut str = item.as_mut();
    assert!(ptr::eq(rf, item.first.as_mut()));
}

#[derive(AsMut)]
struct AnnotatedGenericNamedForward<T> {
    #[as_mut(forward)]
    first: T,
    second: i32,
}

#[test]
fn annotated_generic_named_forward() {
    let mut item = AnnotatedGenericNamedForward {
        first: "test".to_owned(),
        second: 0,
    };

    let rf: &mut str = item.as_mut();
    assert!(ptr::eq(rf, item.first.as_mut()));
}

#[cfg(feature = "std")]
mod pathbuf {
    use std::path::PathBuf;

    use super::*;

    #[derive(AsMut)]
    struct MultiFieldTuple(#[as_mut] String, #[as_mut] PathBuf, Vec<usize>);

    #[test]
    fn multi_field_tuple() {
        let mut item = MultiFieldTuple("test".into(), PathBuf::new(), vec![]);

        assert!(ptr::eq(&mut item.0, item.as_mut()));
        assert!(ptr::eq(&mut item.1, item.as_mut()));
    }

    #[derive(AsMut)]
    struct MultiFieldNamed {
        #[as_mut]
        first: String,
        #[as_mut]
        second: PathBuf,
        third: Vec<usize>,
    }

    #[test]
    fn multi_field_named() {
        let mut item = MultiFieldNamed {
            first: "test".into(),
            second: PathBuf::new(),
            third: vec![],
        };

        assert!(ptr::eq(&mut item.first, item.as_mut()));
        assert!(ptr::eq(&mut item.second, item.as_mut()));
    }

    #[derive(AsMut)]
    struct MultiFieldGenericStruct<T> {
        #[as_mut]
        first: Vec<T>,
        #[as_mut]
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

        assert!(ptr::eq(&mut item.first, item.as_mut()));
        assert!(ptr::eq(&mut item.second, item.as_mut()));
    }
}
