#[derive(derive_more::Borrow)]
struct Foo(
    #[borrow]
    #[borrow(skip)]
    i32,
);

fn main() {}
