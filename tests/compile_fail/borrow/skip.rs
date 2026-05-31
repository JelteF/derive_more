#[derive(derive_more::Borrow)]
struct Foo(#[borrow(skip)] i32);

fn main() {}
