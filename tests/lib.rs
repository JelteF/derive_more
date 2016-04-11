#![feature(rustc_private, custom_derive, plugin)]
#![plugin(derive_more)]

extern crate syntax;
use syntax::codemap;

#[derive(Eq, PartialEq, Debug)]
#[derive(Add, Mul)]
struct MyInt(u32, u32);


#[test]
fn main() {
    assert_eq!(MyInt(5, 6) * 10, MyInt(50, 60));
}
