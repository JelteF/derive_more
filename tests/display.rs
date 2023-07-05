#![allow(dead_code, unused_imports)]

#[derive(derive_more::Display)]
enum FooBar {
    #[display("hello {}", _0)]
    Baz(u32),
}

#[test]
fn foobar() {
    assert_eq!(FooBar::Baz(1).to_string(), "hello 1");
}
