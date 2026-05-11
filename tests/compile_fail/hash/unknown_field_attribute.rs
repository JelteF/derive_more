#[derive(derive_more::Hash)]
struct Foo(#[hash(unknown)] i32);

fn main() {}