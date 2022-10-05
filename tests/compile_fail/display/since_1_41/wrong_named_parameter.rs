#[derive(derive_more::Display)]
#[display(fmt = "Stuff({bars})")]
pub struct Foo {
    bar: String,
}

fn main() {}
