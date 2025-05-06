#[derive(derive_more::FromStr)]
#[from_str(unknown = "unknown")]
pub struct Foo {
    bar: i32,
}

fn main() {}
