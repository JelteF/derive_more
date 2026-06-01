#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(nightly, feature(never_type))]
#![allow(dead_code)] // some code is tested for type checking only

#[cfg(not(feature = "std"))]
extern crate alloc;
#[cfg(not(feature = "std"))]
use alloc::string::String;

use derive_more::Constructor;

#[derive(Constructor)]
struct EmptyTuple();

const EMPTY_TUPLE: EmptyTuple = EmptyTuple::new();

#[derive(Constructor)]
struct EmptyStruct {}

const EMPTY_STRUCT: EmptyStruct = EmptyStruct::new();

#[derive(Constructor)]
struct EmptyUnit;

const EMPTY_UNIT: EmptyUnit = EmptyUnit::new();

#[derive(Constructor)]
struct MyInts(i32, i32);

const MY_INTS: MyInts = MyInts::new(1, 2);

#[derive(Constructor)]
struct IntoTuple(#[constructor(into)] String);

#[derive(Constructor)]
struct IntoStruct {
    #[constructor(into)]
    value: String,
}

#[derive(Constructor)]
struct IntoMixed {
    id: i32,
    #[constructor(into)]
    name: String,
}

#[test]
fn into_arguments_are_converted() {
    // Accepts any `Into<String>`
    let tuple = IntoTuple::new("tuple");
    assert_eq!(tuple.0, "tuple");

    let named = IntoStruct::new("named");
    assert_eq!(named.value, "named");

    // Non-`Into<...>` fields keep requiring their exact type.
    let mixed = IntoMixed::new(1, "mixed");
    assert_eq!(mixed.id, 1);
    assert_eq!(mixed.name, "mixed");

    // An owned `String` should still work
    let owned = IntoTuple::new(String::from("owned"));
    assert_eq!(owned.0, "owned");
}

#[derive(Constructor)]
struct Point2D {
    x: i32,
    y: i32,
}

const POINT_2D: Point2D = Point2D::new(-4, 7);

#[cfg(nightly)]
mod never {
    use super::*;

    #[derive(Constructor)]
    struct Tuple(!);

    #[derive(Constructor)]
    struct Struct {
        field: !,
    }

    #[derive(Constructor)]
    struct TupleMulti(i32, !);

    #[derive(Constructor)]
    struct StructMulti {
        field: !,
        other: i32,
    }
}

mod deprecated {
    use super::*;

    #[derive(Constructor)]
    #[deprecated(note = "struct")]
    struct Tuple(#[deprecated(note = "field")] i32);

    #[derive(Constructor)]
    #[deprecated(note = "struct")]
    struct Struct {
        #[deprecated(note = "field")]
        field: i32,
    }
}

mod proxy_lint_attr {
    #![deny(non_snake_case, clippy::too_many_arguments)]

    use super::*;

    #[allow(clippy::too_many_arguments)]
    #[derive(Constructor)]
    struct ManyArguments(
        u8,
        u8,
        u8,
        u8,
        u8,
        u8,
        u8,
        u8,
        u8,
        u8,
        u8,
        u8,
        u8,
        u8,
        u8,
        u8,
        u8,
        u8,
    );

    #[expect(non_snake_case)]
    #[derive(Constructor)]
    struct User {
        Num: i32,
    }
}
