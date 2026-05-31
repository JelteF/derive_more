#[derive(derive_more::Borrow)]
#[borrow(forward)]
struct Foo<T>(Box<T>);

fn main() {}
