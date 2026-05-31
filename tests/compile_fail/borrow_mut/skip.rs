#[derive(derive_more::BorrowMut)]
struct Foo(#[borrow_mut(skip)] i32);

fn main() {}
