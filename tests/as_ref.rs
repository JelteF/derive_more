#![cfg_attr(not(feature = "std"), no_std)]
#![allow(dead_code)]

#[cfg(not(feature = "std"))]
extern crate alloc;

#[cfg(not(feature = "std"))]
use alloc::{borrow::ToOwned, string::String, vec, vec::Vec};
use core::ptr;

use derive_more::AsRef;

#[derive(AsRef)]
struct SingleFieldTuple(String);

#[test]
fn single_field_tuple() {
    let item = SingleFieldTuple("test".into());

    assert!(ptr::eq(&item.0, item.as_ref()));
}

#[derive(AsRef)]
#[as_ref(forward)]
struct SingleFieldTupleForward(Vec<i32>);

#[test]
fn single_field_tuple_forward() {
    let item = SingleFieldTupleForward(vec![]);

    let rf: &[i32] = item.as_ref();
    assert!(ptr::eq(rf, item.0.as_ref()));
}

#[derive(AsRef)]
struct SingleFieldNamed {
    first: String,
}

#[test]
fn single_field_named() {
    let item = SingleFieldNamed {
        first: "test".into(),
    };

    assert!(ptr::eq(&item.first, item.as_ref()));
}

#[derive(AsRef)]
#[as_ref(forward)]
struct SingleFieldNamedForward {
    first: Vec<i32>,
}

#[test]
fn single_field_named_forward() {
    let item = SingleFieldNamedForward { first: vec![] };

    let rf: &[i32] = item.as_ref();
    assert!(ptr::eq(rf, item.first.as_ref()));
}

#[cfg(feature = "std")]
mod pathbuf {
    use std::path::PathBuf;

    use super::*;

    #[derive(AsRef)]
    struct MultiFieldTuple(#[as_ref] String, #[as_ref] PathBuf, Vec<usize>);

    #[test]
    fn multi_field_tuple() {
        let item = MultiFieldTuple("test".into(), PathBuf::new(), vec![]);

        assert!(ptr::eq(&item.0, item.as_ref()));
        assert!(ptr::eq(&item.1, item.as_ref()));
    }

    #[derive(AsRef)]
    struct MultiFieldNamed {
        #[as_ref]
        first: String,
        #[as_ref]
        second: PathBuf,
        third: Vec<usize>,
    }

    #[test]
    fn multi_field_named() {
        let item = MultiFieldNamed {
            first: "test".into(),
            second: PathBuf::new(),
            third: vec![],
        };

        assert!(ptr::eq(&item.first, item.as_ref()));
        assert!(ptr::eq(&item.second, item.as_ref()));
    }
}

#[derive(AsRef)]
struct AnnotatedTupleForward(#[as_ref(forward)] String, i32);

#[test]
fn annotated_tuple_forward() {
    let item = AnnotatedTupleForward("test".into(), 0);

    let rf: &str = item.as_ref();
    assert!(ptr::eq(rf, item.0.as_ref()));
}

#[derive(AsRef)]
struct AnnotatedNamedForward {
    #[as_ref(forward)]
    first: String,
    second: i32,
}

#[test]
fn annotated_named_forward() {
    let item = AnnotatedNamedForward {
        first: "test".into(),
        second: 0,
    };

    let rf: &str = item.as_ref();
    assert!(ptr::eq(rf, item.first.as_ref()));
}

#[derive(AsRef)]
struct SingleFieldGenericStruct<T> {
    first: T,
}

#[test]
fn single_field_generic_struct() {
    let item = SingleFieldGenericStruct { first: "test" };

    assert!(ptr::eq(&item.first, item.as_ref()));
}

#[derive(AsRef)]
#[as_ref(forward)]
struct SingleFieldGenericStructForward<T> {
    first: T,
}

#[test]
fn single_field_generic_struct_forward() {
    let item = SingleFieldGenericStruct {
        first: "test".to_owned(),
    };

    let rf: &str = item.as_ref();
    assert!(ptr::eq(rf, item.first.as_ref()));
}

#[derive(AsRef)]
struct AnnotatedGenericStructForward<T> {
    #[as_ref(forward)]
    first: T,
    second: i32,
}

#[test]
fn annotated_generic_struct_forward() {
    let item = AnnotatedGenericStructForward {
        first: "test".to_owned(),
        second: 0,
    };

    let rf: &str = item.as_ref();
    assert!(ptr::eq(rf, item.first.as_ref()));
}

#[derive(AsRef)]
struct MultiFieldGenericStruct<T, U> {
    #[as_ref]
    first: Vec<T>,
    #[as_ref]
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

    assert!(ptr::eq(&item.first, item.as_ref()));
    assert!(ptr::eq(&item.second, item.as_ref()));
}
