#[derive(derive_more::Display)]
#[display(fmt = "Stuff({})", .0)]
pub struct Foo(String);

fn main() {}
