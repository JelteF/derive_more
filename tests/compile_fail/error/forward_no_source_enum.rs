use derive_more::Error;

#[derive(Debug, Error)]
#[error(forward)]
enum Foo {
    Bar,
    Baz {
        source: Box<dyn Error + Send + 'static>,
    },
}

impl ::core::fmt::Display for Foo {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        write!(f, "")
    }
}

fn main() {}
