#[derive(derive_more::Borrow)]
struct Foo(
    #[borrow]
    #[borrow(forward)]
    String,
);

fn main() {}
