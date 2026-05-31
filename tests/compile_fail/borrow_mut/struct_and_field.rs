#[derive(derive_more::BorrowMut)]
#[borrow_mut(forward)]
struct Foo(
    #[borrow_mut]
    i32,
);

fn main() {}
