#[derive(derive_more::Display)]
#[display(fmt = "Stuff({})", bar)]
#[display(unknown = "unknown")]
pub struct Foo {
    bar: String,
}

fn main() {}
