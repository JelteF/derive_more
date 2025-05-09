#![cfg_attr(not(feature = "std"), no_std)]
#![allow(dead_code)] // some code is tested for type checking only

#[cfg(not(feature = "std"))]
extern crate alloc;

#[cfg(not(feature = "std"))]
use alloc::string::ToString;

use derive_more::FromStr;

mod structs {
    use super::*;

    mod forward {
        use super::*;

        #[test]
        fn unnamed() {
            #[derive(Debug, Eq, FromStr, PartialEq)]
            struct Int(i32);

            assert_eq!("3".parse::<Int>().unwrap(), Int(3));
            assert_eq!("0".parse::<Int>().unwrap(), Int(0));
            assert_eq!("2147483647".parse::<Int>().unwrap(), Int(i32::MAX));
            assert_eq!("-2147483648".parse::<Int>().unwrap(), Int(i32::MIN));

            assert_eq!(
                "2147483648".parse::<Int>().unwrap_err().to_string(),
                "number too large to fit in target type",
            );
            assert_eq!(
                "-2147483649".parse::<Int>().unwrap_err().to_string(),
                "number too small to fit in target type",
            );
            assert_eq!(
                "wow".parse::<Int>().unwrap_err().to_string(),
                "invalid digit found in string",
            );
        }

        #[test]
        fn named() {
            #[derive(Debug, Eq, FromStr, PartialEq)]
            struct Point1D {
                x: i32,
            }

            assert_eq!("3".parse::<Point1D>().unwrap(), Point1D { x: 3 });
            assert_eq!("0".parse::<Point1D>().unwrap(), Point1D { x: 0 });
            assert_eq!(
                "2147483647".parse::<Point1D>().unwrap(),
                Point1D { x: i32::MAX },
            );
            assert_eq!(
                "-2147483648".parse::<Point1D>().unwrap(),
                Point1D { x: i32::MIN },
            );

            assert_eq!(
                "2147483648".parse::<Point1D>().unwrap_err().to_string(),
                "number too large to fit in target type",
            );
            assert_eq!(
                "-2147483649".parse::<Point1D>().unwrap_err().to_string(),
                "number too small to fit in target type",
            );
            assert_eq!(
                "wow".parse::<Point1D>().unwrap_err().to_string(),
                "invalid digit found in string",
            );
        }

        mod generic {
            use super::*;

            #[test]
            fn unnamed() {
                #[derive(Debug, Eq, FromStr, PartialEq)]
                struct Int<I>(I);

                assert_eq!("3".parse::<Int<i32>>().unwrap(), Int(3));
                assert_eq!("0".parse::<Int<i32>>().unwrap(), Int(0));
                assert_eq!("2147483647".parse::<Int<i32>>().unwrap(), Int(i32::MAX));
                assert_eq!("-2147483648".parse::<Int<i32>>().unwrap(), Int(i32::MIN));

                assert_eq!(
                    "2147483648".parse::<Int<i32>>().unwrap_err().to_string(),
                    "number too large to fit in target type",
                );
                assert_eq!(
                    "-2147483649".parse::<Int<i32>>().unwrap_err().to_string(),
                    "number too small to fit in target type",
                );
                assert_eq!(
                    "wow".parse::<Int<i32>>().unwrap_err().to_string(),
                    "invalid digit found in string",
                );
            }

            #[test]
            fn named() {
                #[derive(Debug, Eq, FromStr, PartialEq)]
                struct Point1D<I> {
                    x: I,
                }

                assert_eq!("3".parse::<Point1D<i32>>().unwrap(), Point1D { x: 3 });
                assert_eq!("0".parse::<Point1D<i32>>().unwrap(), Point1D { x: 0 });
                assert_eq!(
                    "2147483647".parse::<Point1D<i32>>().unwrap(),
                    Point1D { x: i32::MAX },
                );
                assert_eq!(
                    "-2147483648".parse::<Point1D<i32>>().unwrap(),
                    Point1D { x: i32::MIN },
                );

                assert_eq!(
                    "2147483648"
                        .parse::<Point1D<i32>>()
                        .unwrap_err()
                        .to_string(),
                    "number too large to fit in target type",
                );
                assert_eq!(
                    "-2147483649"
                        .parse::<Point1D<i32>>()
                        .unwrap_err()
                        .to_string(),
                    "number too small to fit in target type",
                );
                assert_eq!(
                    "wow".parse::<Point1D<i32>>().unwrap_err().to_string(),
                    "invalid digit found in string",
                );
            }
        }
    }

    mod flat {
        use super::*;

        #[test]
        fn unit() {
            #[derive(Debug, Eq, FromStr, PartialEq)]
            struct Foo;

            assert_eq!("Foo".parse::<Foo>().unwrap(), Foo);
        }

        #[test]
        fn empty_tuple() {
            #[derive(Debug, Eq, FromStr, PartialEq)]
            struct Bar();

            assert_eq!("Bar".parse::<Bar>().unwrap(), Bar());
        }

        #[test]
        fn empty_struct() {
            #[derive(Debug, Eq, FromStr, PartialEq)]
            struct Baz {}

            assert_eq!("Baz".parse::<Baz>().unwrap(), Baz {});
        }

        #[test]
        fn case_insensitive() {
            #[derive(Debug, Eq, FromStr, PartialEq)]
            struct Foo;

            assert_eq!("Foo".parse::<Foo>().unwrap(), Foo);
            assert_eq!("FOO".parse::<Foo>().unwrap(), Foo);
            assert_eq!("foo".parse::<Foo>().unwrap(), Foo);

            assert_eq!(
                "baz".parse::<Foo>().unwrap_err().to_string(),
                "Invalid `Foo` string representation",
            );
            assert_eq!(
                "other".parse::<Foo>().unwrap_err().to_string(),
                "Invalid `Foo` string representation",
            );
        }
    }
}

mod enums {
    use super::*;

    mod flat {
        use super::*;

        /// Assertion that `FromStr` does not trigger an ambiguous associated item error for `Err`.
        #[derive(FromStr)]
        enum EnumWithErr {
            Err,
        }

        #[test]
        fn empty() {
            #[derive(Debug, Eq, FromStr, PartialEq)]
            enum Enum {}

            assert_eq!(
                "other".parse::<Enum>().unwrap_err().to_string(),
                "Invalid `Enum` string representation",
            );
        }

        #[test]
        fn unit() {
            #[derive(Debug, Eq, FromStr, PartialEq)]
            enum Enum {
                Foo,
            }

            assert_eq!("Foo".parse::<Enum>().unwrap(), Enum::Foo);
            assert_eq!("FOO".parse::<Enum>().unwrap(), Enum::Foo);
            assert_eq!("foo".parse::<Enum>().unwrap(), Enum::Foo);

            assert_eq!(
                "baz".parse::<Enum>().unwrap_err().to_string(),
                "Invalid `Enum` string representation",
            );
            assert_eq!(
                "other".parse::<Enum>().unwrap_err().to_string(),
                "Invalid `Enum` string representation",
            );
        }

        #[test]
        fn empty_tuple() {
            #[derive(Debug, Eq, FromStr, PartialEq)]
            enum Enum {
                Foo(),
            }

            assert_eq!("Foo".parse::<Enum>().unwrap(), Enum::Foo());
            assert_eq!("FOO".parse::<Enum>().unwrap(), Enum::Foo());
            assert_eq!("foo".parse::<Enum>().unwrap(), Enum::Foo());

            assert_eq!(
                "baz".parse::<Enum>().unwrap_err().to_string(),
                "Invalid `Enum` string representation",
            );
            assert_eq!(
                "other".parse::<Enum>().unwrap_err().to_string(),
                "Invalid `Enum` string representation",
            );
        }

        #[test]
        fn empty_struct() {
            #[derive(Debug, Eq, FromStr, PartialEq)]
            enum Enum {
                Foo {},
            }

            assert_eq!("Foo".parse::<Enum>().unwrap(), Enum::Foo {});
            assert_eq!("FOO".parse::<Enum>().unwrap(), Enum::Foo {});
            assert_eq!("foo".parse::<Enum>().unwrap(), Enum::Foo {});

            assert_eq!(
                "baz".parse::<Enum>().unwrap_err().to_string(),
                "Invalid `Enum` string representation",
            );
            assert_eq!(
                "other".parse::<Enum>().unwrap_err().to_string(),
                "Invalid `Enum` string representation",
            );
        }

        #[test]
        fn case_insensitive() {
            #[derive(Debug, Eq, FromStr, PartialEq)]
            enum Enum {
                Foo,
                Bar,
            }

            assert_eq!("Foo".parse::<Enum>().unwrap(), Enum::Foo);
            assert_eq!("FOO".parse::<Enum>().unwrap(), Enum::Foo);
            assert_eq!("foo".parse::<Enum>().unwrap(), Enum::Foo);

            assert_eq!("Bar".parse::<Enum>().unwrap(), Enum::Bar);
            assert_eq!("baR".parse::<Enum>().unwrap(), Enum::Bar);
            assert_eq!("bar".parse::<Enum>().unwrap(), Enum::Bar);
            assert_eq!("BAR".parse::<Enum>().unwrap(), Enum::Bar);

            assert_eq!(
                "baz".parse::<Enum>().unwrap_err().to_string(),
                "Invalid `Enum` string representation",
            );
            assert_eq!(
                "other".parse::<Enum>().unwrap_err().to_string(),
                "Invalid `Enum` string representation",
            );
        }

        #[test]
        fn case_sensitive() {
            #[derive(Debug, Eq, FromStr, PartialEq)]
            enum Enum {
                Baz,
                BaZ,
                Bar,
            }

            assert_eq!("Baz".parse::<Enum>().unwrap(), Enum::Baz);
            assert_eq!("BaZ".parse::<Enum>().unwrap(), Enum::BaZ);

            assert_eq!("Bar".parse::<Enum>().unwrap(), Enum::Bar);
            assert_eq!("baR".parse::<Enum>().unwrap(), Enum::Bar);
            assert_eq!("bar".parse::<Enum>().unwrap(), Enum::Bar);
            assert_eq!("BAR".parse::<Enum>().unwrap(), Enum::Bar);

            assert_eq!(
                "baz".parse::<Enum>().unwrap_err().to_string(),
                "Invalid `Enum` string representation",
            );
            assert_eq!(
                "other".parse::<Enum>().unwrap_err().to_string(),
                "Invalid `Enum` string representation",
            );
        }
    }
}
