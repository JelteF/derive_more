#![feature(rustc_private, custom_derive, plugin)]
#![plugin(derive_more)]

extern crate syntax;
use syntax::codemap;

#[derive(From)]
#[derive(Eq, PartialEq, Debug)]
#[derive(Add, Mul)]
struct MyInt(i32);


#[test]
fn main() {
    assert_eq!(MyInt(5) * 10, 50.into());
}
