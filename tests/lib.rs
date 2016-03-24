#![feature(rustc_private, custom_derive, plugin)]
#![plugin(derive_from)]

extern crate syntax;
use syntax::codemap;

#[derive(From, Eq, PartialEq, Debug)]
struct MyInt(i32);

#[derive(From, Eq, PartialEq, Debug)]
struct MyUInt(u64);

#[derive(From, Eq, PartialEq, Debug)]
struct NestedInt(MyInt);

#[derive(From, Eq, PartialEq, Debug)]
struct MySpan(codemap::Span);

//#[derive(From)]
#[derive(Eq, PartialEq, Debug)]
enum MyIntEnum{Int(i32)}

#[test]
fn main() {
    assert_eq!(MyInt::from(5), MyInt(5))
}
