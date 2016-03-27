#![feature(rustc_private, custom_derive, plugin)]
#![plugin(derive_from)]

extern crate syntax;
use syntax::codemap;

#[derive(From)]
#[derive(Eq, PartialEq, Debug)]
#[derive(Add)]
struct MyInt(i32);

#[derive(Add)]
#[derive(Eq, PartialEq, Debug)]
struct MyUInt(u64, u64);

#[derive(Add)]
#[derive(Eq, PartialEq, Debug)]
struct NormalStruct{int1: u64, int2: u64}

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
    assert_eq!(MyUInt(4, 5) + MyUInt(1, 2), MyUInt(5, 7));
    let s1 = NormalStruct{int1: 1, int2: 2};
    let s2 = NormalStruct{int1: 2, int2: 3};
    let s3 = NormalStruct{int1: 3, int2: 5};
    assert_eq!(s1 + s2, s3);
}
