#[derive(derive_more::Borrow)]
struct Foo {
    #[borrow(baz)]
    bar: i32,
}

fn main() {}
