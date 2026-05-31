#[derive(derive_more::Borrow)]
#[borrow(forward)]
struct Foo<'a, T>(&'a T);

fn main() {}
