#[derive(derive_more::PartialEq)]
struct Foo(#[partial_eq(with(unknown))] i32);

fn incompatible_types(a:& str) ->i32 {0}

#[derive(derive_more::PartialEq)]
struct Bar(#[partial_eq(with(incompatible_types))] i32);


#[derive(derive_more::PartialEq)]
enum Enum {
    Bar { #[partial_eq(with(unknown))] i: i32 },
    Baz { #[partial_eq(with(incompatible_types))] i: i32 },
}



fn main() {}
