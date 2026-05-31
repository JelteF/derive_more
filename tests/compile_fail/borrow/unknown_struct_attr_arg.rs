#[derive(derive_more::Borrow)]
#[borrow(baz)]
struct Foo {
    bar: i32,
}

fn main() {}
