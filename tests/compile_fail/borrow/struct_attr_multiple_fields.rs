#[derive(derive_more::Borrow)]
#[borrow(forward)]
struct Foo {
    bar: i32,
    baz: f32,
}

fn main() {}
