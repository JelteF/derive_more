#[derive(derive_more::BorrowMut)]
#[borrow_mut(forward)]
struct Foo<T>(T);

fn main() {}
