#[derive(derive_more::Display)]
#[display(fmt = "Stuff({:M})", bar)]
pub struct Foo {
    bar: String,
}

fn main() {}
