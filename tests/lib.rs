#![feature(rustc_private, custom_derive, plugin)]
#![plugin(derive_from)]

extern crate syntax;
use syntax::codemap;

#[derive(From)]
#[derive(Eq, PartialEq, Debug)]
#[derive(Add)]
struct MyInt(i32);

#[derive(From)]
//#[derive(Eq, PartialEq, Debug)]
struct MyUInt(u64);

#[derive(From)]
//#[derive(Eq, PartialEq, Debug)]
struct NestedInt(MyInt);

#[derive(From)]
//#[derive(Eq, PartialEq, Debug)]
struct MySpan(codemap::Span);

#[derive(From)]
//#[derive(Eq, PartialEq, Debug)]
struct MySpan2(::syntax::codemap::Span);

#[derive(Eq, PartialEq, Debug)]
#[derive(From)]
enum MyIntEnum{
    Int(i32),
    Bool(bool),
    UnsignedOne(u32),
    UnsignedTwo(u32),
    Nothing,
}

#[test]
fn main() {
    assert_eq!(MyInt(5), 5.into());
    assert_eq!(MyIntEnum::Int(5), 5.into());
    assert_eq!(MyIntEnum::Bool(true), true.into());
    assert!(MyIntEnum::Bool(false) != true.into());

    assert_eq!(MyInt(4) + MyInt(1), 5.into());
}
