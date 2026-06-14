#[derive(derive_more::Constructor)]
struct Foo(#[constructor(skip)] i32);

fn main() {}
