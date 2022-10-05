#[derive(derive_more::Display)]
#[display(fmt = "Stuff({_1})")]
pub struct Foo(String);

fn main() {}
