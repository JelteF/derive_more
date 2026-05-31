#[derive(derive_more::Borrow)]
#[borrow(forward)]
struct Foo(i32);

fn main() {
    use core::borrow::Borrow as _;

    let item = Foo(1);
    let _: &str = item.borrow();
}
