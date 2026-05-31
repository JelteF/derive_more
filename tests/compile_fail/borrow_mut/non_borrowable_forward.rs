#[derive(derive_more::Borrow, derive_more::BorrowMut)]
#[borrow(forward)]
#[borrow_mut(forward)]
struct Foo(i32);

fn main() {
    use core::borrow::BorrowMut as _;

    let mut item = Foo(1);
    let _: &mut str = item.borrow_mut();
}
