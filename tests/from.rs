#![allow(dead_code)]

use std::borrow::Cow;

use derive_more::From;
use static_assertions::assert_not_impl_any;

mod structs {
    use super::*;

    mod unit {
        use super::*;

        #[derive(Debug, From, PartialEq)]
        struct Unit;

        #[derive(Debug, From, PartialEq)]
        struct Tuple();

        #[derive(Debug, From, PartialEq)]
        struct Struct {}

        #[test]
        fn assert() {
            assert_eq!(Unit, ().into());
            assert_eq!(Tuple(), ().into());
            assert_eq!(Struct {}, ().into());
        }
    }

    mod single_field {
        use super::*;

        #[derive(Debug, From, PartialEq)]
        struct Tuple(i32);

        #[derive(Debug, From, PartialEq)]
        struct Struct {
            field: i32,
        }

        #[test]
        fn assert() {
            assert_eq!(Tuple(42), 42.into());
            assert_eq!(Struct { field: 42 }, 42.into());
        }

        mod types {
            use super::*;

            #[derive(Debug, From, PartialEq)]
            #[from(i16)]
            struct Tuple(i32);

            #[derive(Debug, From, PartialEq)]
            #[from(i16)]
            struct Struct {
                field: i32,
            }

            #[test]
            fn assert() {
                assert_not_impl_any!(Tuple: From<i32>);
                assert_not_impl_any!(Struct: From<i32>);

                assert_eq!(Tuple(42), 42_i16.into());
                assert_eq!(Struct { field: 42 }, 42_i16.into());
            }
        }

        mod forward {
            use super::*;

            #[derive(Debug, From, PartialEq)]
            #[from(forward)]
            struct Tuple(i32);

            #[derive(Debug, From, PartialEq)]
            #[from(forward)]
            struct Struct {
                field: i32,
            }

            #[test]
            fn assert() {
                assert_eq!(Tuple(42), 42_i8.into());
                assert_eq!(Tuple(42), 42_i16.into());
                assert_eq!(Tuple(42), 42_i32.into());
                assert_eq!(Struct { field: 42 }, 42_i8.into());
                assert_eq!(Struct { field: 42 }, 42_i16.into());
                assert_eq!(Struct { field: 42 }, 42_i32.into());
            }
        }
    }

    mod multi_field {
        use super::*;

        #[derive(Debug, From, PartialEq)]
        struct Tuple(i32, i16);

        #[derive(Debug, From, PartialEq)]
        struct Struct {
            field1: i32,
            field2: i16,
        }

        #[test]
        fn assert() {
            assert_eq!(Tuple(0, 1), (0, 1_i16).into());
            assert_eq!(
                Struct {
                    field1: 0,
                    field2: 1
                },
                (0, 1_i16).into()
            );
        }

        mod types {
            use super::*;

            #[derive(Debug, From, PartialEq)]
            #[from((i16, i16))]
            struct Tuple(i32, i16);

            #[derive(Debug, From, PartialEq)]
            #[from((i16, i16))]
            struct Struct {
                field1: i32,
                field2: i16,
            }

            #[test]
            fn assert() {
                assert_not_impl_any!(Tuple: From<(i32, i16)>);
                assert_not_impl_any!(Struct: From<(i32, i16)>);

                assert_eq!(Tuple(0, 1), (0_i16, 1_i16).into());
                assert_eq!(
                    Struct {
                        field1: 0,
                        field2: 1
                    },
                    (0_i16, 1_i16).into(),
                );
            }
        }

        mod forward {
            use super::*;

            #[derive(Debug, From, PartialEq)]
            #[from(forward)]
            struct Tuple(i32, i16);

            #[derive(Debug, From, PartialEq)]
            #[from(forward)]
            struct Struct {
                field1: i32,
                field2: i16,
            }

            #[test]
            fn assert() {
                assert_eq!(Tuple(0, 1), (0_i8, 1_i8).into());
                assert_eq!(Tuple(0, 1), (0_i8, 1_i16).into());
                assert_eq!(Tuple(0, 1), (0_i16, 1_i8).into());
                assert_eq!(Tuple(0, 1), (0_i16, 1_i16).into());
                assert_eq!(Tuple(0, 1), (0_i32, 1_i8).into());
                assert_eq!(Tuple(0, 1), (0_i32, 1_i16).into());
                assert_eq!(
                    Struct {
                        field1: 0,
                        field2: 1
                    },
                    (0_i8, 1_i8).into(),
                );
                assert_eq!(
                    Struct {
                        field1: 0,
                        field2: 1
                    },
                    (0_i8, 1_i16).into(),
                );
                assert_eq!(
                    Struct {
                        field1: 0,
                        field2: 1
                    },
                    (0_i16, 1_i8).into(),
                );
                assert_eq!(
                    Struct {
                        field1: 0,
                        field2: 1
                    },
                    (0_i16, 1_i16).into(),
                );
                assert_eq!(
                    Struct {
                        field1: 0,
                        field2: 1
                    },
                    (0_i32, 1_i8).into(),
                );
                assert_eq!(
                    Struct {
                        field1: 0,
                        field2: 1
                    },
                    (0_i32, 1_i16).into(),
                );
            }
        }
    }
}

#[derive(From)]
struct EmptyTuple();

#[derive(From)]
struct EmptyStruct {}

#[derive(From)]
struct EmptyUnit;

#[derive(From)]
struct MyInt(i32);

#[derive(From)]
struct MyInts(i32, i32);

#[derive(From)]
struct Point1D {
    x: i32,
}

#[derive(From)]
struct Point2D {
    x: i32,
    y: i32,
}

#[derive(From)]
enum MixedInts {
    SmallInt(i32),
    NamedBigInt {
        int: i64,
    },
    TwoSmallInts(i32, i32),
    NamedBigInts {
        x: i64,
        y: i64,
    },
    #[from(ignore)]
    Unsigned(u32),
    NamedUnsigned {
        x: u32,
    },
}

#[derive(Debug, Eq, PartialEq)]
#[derive(From)]
#[from(forward)]
struct MyIntForward(u64);

#[test]
fn forward_struct() {
    assert_eq!(MyIntForward(42), 42u32.into());
    assert_eq!(MyIntForward(42), 42u16.into());
    assert_eq!(MyIntForward(42), 42u64.into());
}

#[derive(Debug, Eq, PartialEq)]
#[derive(From)]
enum MixedIntsForward {
    #[from(forward)]
    SmallInt(i32),
    NamedBigInt {
        int: i64,
    },
}

#[test]
fn forward_enum() {
    assert_eq!(MixedIntsForward::SmallInt(42), 42i32.into());
    assert_eq!(MixedIntsForward::SmallInt(42), 42i16.into());
}

#[derive(From, PartialEq)]
enum AutoIgnore {
    SmallInt(i32),
    Uninteresting,
    Uninteresting2,
}

#[test]
fn auto_ignore_variants() {
    assert!(AutoIgnore::SmallInt(42) == 42i32.into());
}

#[derive(From, PartialEq)]
enum AutoIgnoreWithDefaultTrue {
    #[from(ignore)]
    SmallInt(i32),
    Uninteresting,
    Uninteresting2,
}

#[derive(From, PartialEq)]
enum AutoIgnoreWithForwardFields2 {
    #[from(forward)]
    SmallInt(i32),
    SmallIntIgnore(i32),
}

#[test]
fn auto_ignore_with_forward_field2() {
    assert!(AutoIgnoreWithForwardFields2::SmallInt(42) == 42i32.into());
    assert!(AutoIgnoreWithForwardFields2::SmallInt(42) == 42i16.into());
}

#[derive(Debug, Eq, PartialEq)]
#[derive(From)]
#[from(u8, u16, u32, u64)]
struct MyIntExplicit(u64);

#[test]
fn explicit_types_struct() {
    assert_eq!(MyIntExplicit(42), 42u8.into());
    assert_eq!(MyIntExplicit(42), 42u16.into());
    assert_eq!(MyIntExplicit(42), 42u32.into());
    assert_eq!(MyIntExplicit(42), 42u64.into());
}

#[derive(Debug, Eq, PartialEq)]
#[derive(From)]
#[from((i8, i8), (i16, i16), (i32, i32))]
struct MyIntsExplicit(i32, i32);

#[test]
fn explicit_types_struct_tupled() {
    assert_eq!(MyIntsExplicit(42, 42), (42i32, 42i32).into());
    assert_eq!(MyIntsExplicit(42, 42), (42i8, 42i8).into());
    assert_eq!(MyIntsExplicit(42, 42), (42i16, 42i16).into());
}

#[derive(Debug, Eq, PartialEq)]
#[derive(From)]
enum MixedIntsExplicit {
    #[from(i8, i32)]
    SmallInt(i32),
    #[from(i16, i64, i128)]
    AnotherInt(i128),
    #[from(skip)]
    NamedBigInt { int: i64 },
}

#[test]
fn explicit_types_enum() {
    assert_eq!(MixedIntsExplicit::SmallInt(42), 42i32.into());
    assert_eq!(MixedIntsExplicit::SmallInt(42), 42i8.into());

    assert_eq!(MixedIntsExplicit::AnotherInt(42), 42i128.into());
    assert_eq!(MixedIntsExplicit::AnotherInt(42), 42i64.into());
    assert_eq!(MixedIntsExplicit::AnotherInt(42), 42i16.into());
}

#[derive(Debug, Eq, PartialEq)]
#[derive(From)]
#[from((i8, i8), (i16, i16), (i32, i32))]
struct Point2DExplicit {
    x: i32,
    y: i32,
}

#[test]
fn explicit_types_point_2d() {
    let expected = Point2DExplicit { x: 42, y: 42 };
    assert_eq!(expected, (42i32, 42i32).into());
    assert_eq!(expected, (42i8, 42i8).into());
    assert_eq!(expected, (42i16, 42i16).into());
}

#[derive(Debug, Eq, From, PartialEq)]
#[from(String, Cow<'_, str>, &str)]
struct Name(String);

#[test]
fn explicit_complex_types_name() {
    let name = "Eärendil";
    let expected = Name(name.into());
    assert_eq!(expected, name.to_owned().into());
    assert_eq!(expected, name.into());
    assert_eq!(expected, Cow::Borrowed(name).into());
}
