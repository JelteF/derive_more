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
#[as_mut(forward)]
struct SingleFieldTupleForward(Vec<i32>);

#[test]
fn single_field_tuple_forward() {
    let mut item = SingleFieldTupleForward(vec![]);

    let rf: &mut [i32] = item.as_mut();
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

#[derive(AsMut)]
struct AnnotatedTupleForward(#[as_mut(forward)] String, i32);

#[test]
fn annotated_tuple_forward() {
    let mut item = AnnotatedTupleForward("test".into(), 0);

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
struct SingleFieldGenericStruct<T> {
    first: T,
}

#[test]
fn single_field_generic_struct() {
    let mut item = SingleFieldGenericStruct { first: "test" };

    assert!(ptr::eq(&mut item.first, item.as_mut()));
}

#[derive(AsMut)]
#[as_mut(forward)]
struct SingleFieldGenericStructForward<T> {
    first: T,
}

#[test]
fn single_field_generic_struct_forward() {
    let mut item = SingleFieldGenericStructForward {
        first: "test".to_owned(),
    };

    let rf: &mut str = item.as_mut();
    assert!(ptr::eq(rf, item.first.as_mut()));
}

#[derive(AsMut)]
struct AnnotatedGenericStructForward<T> {
    #[as_mut(forward)]
    first: T,
    second: i32,
}

#[test]
fn annotated_generic_struct_forward() {
    let mut item = AnnotatedGenericStructForward {
        first: "test".to_owned(),
        second: 0,
    };

    let rf: &mut str = item.as_mut();
    assert!(ptr::eq(rf, item.first.as_mut()));
}
