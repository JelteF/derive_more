#![cfg_attr(not(feature = "std"), no_std)]
#![allow(dead_code)] // some code is tested for type checking only

#[cfg(not(feature = "std"))]
extern crate alloc;

#[cfg(not(feature = "std"))]
use alloc::string::ToString;

use derive_more::FromStr;

#[derive(FromStr)]
struct MyInt(i32);

#[derive(FromStr)]
struct Point1D {
    x: i32,
}

/// Making sure that `FromStr` does not trigger an ambiguous associated item error for `Err`.
#[derive(FromStr)]
enum EnumWithErr {
    Err,
}

#[derive(Debug, FromStr, PartialEq, Eq)]
enum EnumNoFields {
    Foo,
    Bar,
    Baz,
    BaZ,
}

#[test]
fn enum_test() {
    assert_eq!("Foo".parse::<EnumNoFields>().unwrap(), EnumNoFields::Foo);
    assert_eq!("FOO".parse::<EnumNoFields>().unwrap(), EnumNoFields::Foo);
    assert_eq!("foo".parse::<EnumNoFields>().unwrap(), EnumNoFields::Foo);
    assert_eq!(
        "other".parse::<EnumNoFields>().unwrap_err().to_string(),
        "Invalid `EnumNoFields` string representation",
    );
}

#[test]
fn enum_test_case_sensitive() {
    assert_eq!("Baz".parse::<EnumNoFields>().unwrap(), EnumNoFields::Baz);
    assert_eq!("BaZ".parse::<EnumNoFields>().unwrap(), EnumNoFields::BaZ);
    assert_eq!(
        "baz".parse::<EnumNoFields>().unwrap_err().to_string(),
        "Invalid `EnumNoFields` string representation",
    );
}

mod enums {
    use super::*;

    mod rename_all {
        use super::*;

        mod smoke {
            use super::*;

            #[derive(Debug, FromStr, PartialEq, Eq)]
            #[from_str(rename_all = "snake_case")]
            enum Enum {
                Foo,
                Bar,
                Baz,
                BaZ,
            }

            #[test]
            fn works() {
                assert_eq!("foo".parse::<Enum>().unwrap(), Enum::Foo);
                assert_eq!("bar".parse::<Enum>().unwrap(), Enum::Bar);
                assert_eq!("baz".parse::<Enum>().unwrap(), Enum::Baz);
                assert_eq!("ba_z".parse::<Enum>().unwrap(), Enum::BaZ);
            }

            #[test]
            fn errors() {
                assert_eq!(
                    "Foo".parse::<Enum>().unwrap_err().to_string(),
                    "Invalid `Enum` string representation",
                );
                assert_eq!(
                    "Bar".parse::<Enum>().unwrap_err().to_string(),
                    "Invalid `Enum` string representation",
                );
                assert_eq!(
                    "Baz".parse::<Enum>().unwrap_err().to_string(),
                    "Invalid `Enum` string representation",
                );
                assert_eq!(
                    "BaZ".parse::<Enum>().unwrap_err().to_string(),
                    "Invalid `Enum` string representation",
                );
                assert_eq!(
                    "other".parse::<Enum>().unwrap_err().to_string(),
                    "Invalid `Enum` string representation",
                );
            }
        }

        macro_rules! casing_test {
            ($name:ident, $casing:literal, $VariantOne:literal, $Two:literal) => {
                mod $name {
                    use super::*;

                    #[test]
                    fn enum_top_level() {
                        #[derive(Debug, FromStr, PartialEq, Eq)]
                        #[from_str(rename_all = $casing)]
                        enum Enum {
                            VariantOne,
                            Two,
                        }

                        assert_eq!(
                            $VariantOne.parse::<Enum>().unwrap(),
                            Enum::VariantOne
                        );
                        assert_eq!($Two.parse::<Enum>().unwrap(), Enum::Two);
                    }

                    /*
                    #[test]
                    fn enum_variant_level() {
                        #[derive(Display)]
                        #[display(rename_all = "lowercase")] // ignored
                        enum Enum {
                            #[display(rename_all = $casing)]
                            VariantOne,
                            #[display(rename_all = $casing)]
                            Two,
                        }

                        assert_eq!(Enum::VariantOne.to_string(), $VariantOne);
                        assert_eq!(Enum::Two.to_string(), $Two);
                    }*/
                }
            };
        }

        casing_test!(lower_case, "lowercase", "variantone", "two");
        casing_test!(upper_case, "UPPERCASE", "VARIANTONE", "TWO");
        casing_test!(pascal_case, "PascalCase", "VariantOne", "Two");
        casing_test!(camel_case, "camelCase", "variantOne", "two");
        casing_test!(snake_case, "snake_case", "variant_one", "two");
        casing_test!(
            screaming_snake_case,
            "SCREAMING_SNAKE_CASE",
            "VARIANT_ONE",
            "TWO"
        );
        casing_test!(kebab_case, "kebab-case", "variant-one", "two");
        casing_test!(
            screaming_kebab_case,
            "SCREAMING-KEBAB-CASE",
            "VARIANT-ONE",
            "TWO"
        );
    }
}
