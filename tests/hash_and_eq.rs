#![cfg_attr(not(feature = "std"), no_std)]

use std::hash::{DefaultHasher, Hash, Hasher};

fn do_hash<T: Hash>(t: &T) -> u64 {
    let mut hasher = DefaultHasher::default();
    t.hash(&mut hasher);
    hasher.finish()
}

mod hash_respects_eq_skip {
    use super::*;
    use derive_more::{Eq, Hash, PartialEq};

    #[derive(Hash, Eq, PartialEq)]
    struct Struct {
        field: i32,
        #[eq(skip)]
        _skipped: String,
    }

    #[derive(Hash, Eq, PartialEq)]
    enum Enum {
        A {
            field: i32,
            #[partial_eq(skip)]
            _skipped: String,
        },
    }

    #[test]
    fn assert() {
        assert_eq!(
            do_hash(&Struct {
                field: 42,
                _skipped: "ignored".to_string()
            }),
            do_hash(&42)
        );
        assert_eq!(
            do_hash(&Enum::A {
                field: 42,
                _skipped: "ignored".to_string()
            }),
            do_hash(&(
                std::mem::discriminant(&Enum::A {
                    field: 0,
                    _skipped: String::new()
                }),
                42
            ))
        );
    }
}
