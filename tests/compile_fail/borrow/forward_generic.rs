#[derive(derive_more::Borrow)]
#[borrow(forward)]
struct Foo<T>(T);

fn main() {}
