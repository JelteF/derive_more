#[derive(derive_more::Borrow)]
#[borrow(forward)]
struct Foo {
    #[borrow]
    bar: i32,
}

fn main() {}
