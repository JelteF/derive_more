#![feature(rustc_private, custom_derive, plugin)]
#![plugin(derive_more)]

extern crate syntax;
use syntax::codemap;

#[derive(Eq, PartialEq, Debug)]
#[derive(Add, Mul)]
struct MyInt(u32, u32);

#[derive(Eq, PartialEq, Debug)]
#[derive(Add, Mul)]
struct MyStruct{x:u32, y:u32}

#[test]
fn main() {
    assert_eq!(MyInt(5, 6) * 10, MyInt(50, 60));
    assert_eq!(MyStruct{x:5, y:6} * 10, MyStruct{x:50, y:60});
}
