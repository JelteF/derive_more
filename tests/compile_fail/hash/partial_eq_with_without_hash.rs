fn always_eq(_: &i32, _: &i32) -> bool {
    true
}

#[derive(derive_more::Hash, derive_more::PartialEq, derive_more::Eq)]
struct Foo(#[partial_eq(with(always_eq))] i32);

#[derive(derive_more::Hash, derive_more::PartialEq, derive_more::Eq)]
enum Enum {
    Bar {
        #[partial_eq(with(always_eq))]
        i: i32,
    },
}

fn main() {}
