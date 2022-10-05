#![allow(dead_code)]

use derive_more::FromStr;

#[derive(FromStr)]
struct MyInt(i32);

#[derive(FromStr)]
struct Point1D {
    x: i32,
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
        "other".parse::<EnumNoFields>().unwrap_err(),
        ParseEnumNoFieldsError {}
    );
    assert_eq!(
        ParseEnumNoFieldsError {}.to_string(),
        "invalid enum no fields"
    );
}

#[test]
fn enum_test_case_sensitive() {
    assert_eq!("Baz".parse::<EnumNoFields>().unwrap(), EnumNoFields::Baz);
    assert_eq!("BaZ".parse::<EnumNoFields>().unwrap(), EnumNoFields::BaZ);
    assert_eq!(
        "baz".parse::<EnumNoFields>().unwrap_err(),
        ParseEnumNoFieldsError {}
    );
}
