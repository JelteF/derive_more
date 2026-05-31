#![allow(unused_parens)]

trait Trait {
    type Assoc;
}

#[derive(derive_more::Borrow)]
struct Foo<T: Trait>((T::Assoc));

fn main() {}
