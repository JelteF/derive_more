#[derive(derive_more::FromStr)]
#[from_str(error(CustomError, CustomError::new, custom_error_fn()))]
pub enum Foo {
    Bar,
}

#[derive(derive_more::FromStr)]
#[from_str(error(CustomError, custom_error_fn(), CustomError::new))]
struct Baz;

#[derive(derive_more::FromStr)]
#[from_str(error())]
struct Empty;

fn main() {}
