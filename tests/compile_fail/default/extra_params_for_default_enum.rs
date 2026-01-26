#[derive(derive_more::Default)]
enum Foo { #[default(nope)] Bar, Baz}

fn main() {}