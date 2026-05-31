#[derive(derive_more::BorrowMut)]
struct Foo(#[borrow_mut(baz)] i32);

fn main() {}
