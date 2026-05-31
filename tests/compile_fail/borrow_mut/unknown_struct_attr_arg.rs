#[derive(derive_more::BorrowMut)]
#[borrow_mut(baz)]
struct Foo(i32);

fn main() {}
