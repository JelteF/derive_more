#![cfg_attr(not(feature = "std"), no_std)]
#![allow(dead_code)] // some code is tested for type checking only

use derive_more::TryFrom;

/// Making sure that `TryFrom` does not trigger an ambiguous associated item error for `Error`.
#[derive(TryFrom)]
#[try_from(repr)]
#[repr(u8)]
enum EnumWithError {
    Error,
}

#[test]
fn test_with_repr() {
    #[derive(TryFrom, Clone, Copy, Debug, Eq, PartialEq)]
    #[repr(i16)]
    #[try_from(repr)]
    enum Enum {
        A,
        B = -21,
        C,
        D,
    }
    assert_eq!(Enum::A, Enum::try_from(0i16).unwrap());
    assert_eq!(Enum::B, Enum::try_from(-21).unwrap());
    assert_eq!(Enum::C, Enum::try_from(-20).unwrap());
    assert_eq!(Enum::D, Enum::try_from(-19).unwrap());
    assert!(Enum::try_from(-1).is_err());
}

#[test]
fn enum_without_repr() {
    #[derive(TryFrom, Clone, Copy, Debug, Eq, PartialEq)]
    #[try_from(repr)]
    enum Enum {
        A,
        B = -21,
        C,
        D,
    }
    assert_eq!(Enum::A, Enum::try_from(0isize).unwrap());
    assert_eq!(Enum::B, Enum::try_from(-21).unwrap());
    assert_eq!(Enum::C, Enum::try_from(-20).unwrap());
    assert_eq!(Enum::D, Enum::try_from(-19).unwrap());
    assert!(Enum::try_from(-1).is_err());
}

#[test]
fn enum_with_complex_repr() {
    #[derive(TryFrom, Clone, Copy, Debug, Eq, PartialEq)]
    #[try_from(repr)]
    #[repr(align(16), i32)]
    enum Enum {
        A,
        B = -21,
        C,
        D,
    }
    assert_eq!(Enum::A, Enum::try_from(0i32).unwrap());
    assert_eq!(Enum::B, Enum::try_from(-21).unwrap());
    assert_eq!(Enum::C, Enum::try_from(-20).unwrap());
    assert_eq!(Enum::D, Enum::try_from(-19).unwrap());
    assert!(Enum::try_from(-1).is_err());
}

#[test]
fn test_discriminants_on_enum_with_fields() {
    #[derive(TryFrom, Clone, Copy, Debug, Eq, PartialEq)]
    #[try_from(repr)]
    #[repr(i16)]
    enum Enum {
        A,
        Discriminant = 5,
        Field(usize),
        Empty {},
        FieldWithDiscriminant(u8, i64) = -14,
        EmptyTuple(),
    }

    assert_eq!(Enum::A, Enum::try_from(0).unwrap());
    assert_eq!(Enum::Discriminant, Enum::try_from(5).unwrap());
    assert!(Enum::try_from(6).is_err());
    assert_eq!(Enum::Empty {}, Enum::try_from(7).unwrap());
    assert!(Enum::try_from(-14).is_err());
    assert_eq!(Enum::EmptyTuple(), Enum::try_from(-13).unwrap());
}

#[test]
fn test_try_from_types() {
    #[derive(Debug, PartialEq, Eq)]
    struct F1;

    impl TryFrom<usize> for F1 {
        type Error = ();

        fn try_from(value: usize) -> Result<Self, Self::Error> {
            if value == 1 {
                return Ok(Self);
            }
            Err(())
        }
    }

    #[derive(Debug, PartialEq, Eq)]
    struct F2;

    impl TryFrom<usize> for F2 {
        type Error = ();

        fn try_from(value: usize) -> Result<Self, Self::Error> {
            if value == 2 {
                return Ok(Self);
            }
            Err(())
        }
    }

    #[derive(TryFrom, Debug, PartialEq, Eq)]
    #[try_from(usize)]
    enum Enum {
        Field(F1),
        Field2 { x: F2 },
    }

    assert_eq!(Enum::Field(F1), Enum::try_from(1).unwrap());
    assert_eq!(Enum::Field2 { x: F2 }, Enum::try_from(2).unwrap());
    assert!(Enum::try_from(3).is_err());
}

#[test]
fn test_try_from_types_custom_unit_error() {
    #[derive(Debug, PartialEq, Eq)]
    struct Error;

    #[derive(Debug, PartialEq, Eq)]
    struct F1;

    impl TryFrom<usize> for F1 {
        type Error = Error;

        fn try_from(value: usize) -> Result<Self, Self::Error> {
            if value == 1 {
                return Ok(Self);
            }
            Err(Error)
        }
    }

    #[derive(Debug, PartialEq, Eq)]
    struct F2;

    impl TryFrom<usize> for F2 {
        type Error = Error;

        fn try_from(value: usize) -> Result<Self, Self::Error> {
            if value == 2 {
                return Ok(Self);
            }
            Err(Error)
        }
    }

    #[derive(TryFrom, Debug, PartialEq, Eq)]
    #[try_from(usize, Error)]
    enum Enum {
        Field(F1),
        Field2 { x: F2 },
    }

    assert_eq!(Enum::Field(F1), Enum::try_from(1).unwrap());
    assert_eq!(Enum::Field2 { x: F2 }, Enum::try_from(2).unwrap());
    assert_eq!(Err(Error), Enum::try_from(3));
}

#[test]
fn test_try_from_types_custom_error() {
    #[derive(Debug, PartialEq, Eq)]
    enum Error {
        FromEnum,
        FromVariant,
    }

    #[derive(Debug, PartialEq, Eq)]
    struct F1;

    impl TryFrom<usize> for F1 {
        type Error = Error;

        fn try_from(value: usize) -> Result<Self, Self::Error> {
            if value == 1 {
                return Ok(Self);
            }
            Err(Error::FromVariant)
        }
    }

    #[derive(Debug, PartialEq, Eq)]
    struct F2;

    impl TryFrom<usize> for F2 {
        type Error = Error;

        fn try_from(value: usize) -> Result<Self, Self::Error> {
            if value == 2 {
                return Ok(Self);
            }
            Err(Error::FromVariant)
        }
    }

    assert_eq!(Err(Error::FromVariant), F2::try_from(3));

    #[derive(TryFrom, Debug, PartialEq, Eq)]
    #[try_from(usize, Error, Error::FromEnum)]
    enum Enum {
        Field(F1),
        Field2 { x: F2 },
    }

    assert_eq!(Enum::Field(F1), Enum::try_from(1).unwrap());
    assert_eq!(Enum::Field2 { x: F2 }, Enum::try_from(2).unwrap());
    assert_eq!(Err(Error::FromEnum), Enum::try_from(3));
}

#[test]
fn test_try_from_multiple_types_custom_error() {
    #[derive(Debug, PartialEq, Eq)]
    enum Error {
        FromEnum,
        FromVariant,
    }

    #[derive(Debug, PartialEq, Eq)]
    struct F1;

    impl TryFrom<usize> for F1 {
        type Error = Error;

        fn try_from(value: usize) -> Result<Self, Self::Error> {
            if value == 1 {
                return Ok(Self);
            }
            Err(Error::FromVariant)
        }
    }

    #[derive(Debug, PartialEq, Eq)]
    struct F2;

    impl TryFrom<usize> for F2 {
        type Error = Error;

        fn try_from(value: usize) -> Result<Self, Self::Error> {
            if value == 1 {
                return Ok(Self);
            }
            Err(Error::FromVariant)
        }
    }

    assert_eq!(Err(Error::FromVariant), F2::try_from(3));

    #[derive(TryFrom, Debug, PartialEq, Eq)]
    #[try_from(usize, Error, Error::FromEnum)]
    enum Enum {
        Field(F1, F2),
    }

    assert_eq!(Enum::Field(F1, F2), Enum::try_from(1).unwrap());
    assert_eq!(Err(Error::FromEnum), Enum::try_from(2));
}
