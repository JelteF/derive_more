#![cfg_attr(not(feature = "std"), no_std)]

mod hash_utils;
use hash_utils::do_hash;

mod hash_respects_eq_skip {
    use super::*;
    use derive_more::{Eq, Hash, PartialEq};

    #[derive(Hash, Eq, PartialEq)]
    struct Struct {
        field: i32,
        #[eq(skip)]
        _skipped: &'static str,
    }

    #[derive(Hash, Eq, PartialEq)]
    enum Enum {
        A {
            field: i32,
            #[partial_eq(skip)]
            _skipped: &'static str,
        },
    }

    #[test]
    fn assert() {
        assert_eq!(
            do_hash(&Struct {
                field: 42,
                _skipped: "ignored"
            }),
            do_hash(&42)
        );
        assert_eq!(
            do_hash(&Enum::A {
                field: 42,
                _skipped: "ignored"
            }),
            do_hash(&(
                core::mem::discriminant(&Enum::A {
                    field: 0,
                    _skipped: "ignored"
                }),
                42
            ))
        );
    }
}
