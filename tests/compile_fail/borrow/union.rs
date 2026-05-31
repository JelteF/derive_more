#[derive(derive_more::Borrow)]
union Foo {
    bar: i32,
    baz: f32,
}

fn main() {}
