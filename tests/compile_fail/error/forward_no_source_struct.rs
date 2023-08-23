use derive_more::Error;

#[derive(Debug, Error)]
#[error(forward)]
struct Foo;

impl ::core::fmt::Display for Foo {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        write!(f, "")
    }
}

fn main() {}
