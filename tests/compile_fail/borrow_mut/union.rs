#[derive(derive_more::BorrowMut)]
union Foo {
    bar: i32,
}

fn main() {}
