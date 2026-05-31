trait Trait {
    type Assoc;
}

#[derive(derive_more::Borrow)]
#[borrow(forward)]
struct Foo<'a, T: Trait>(&'a T::Assoc);

fn main() {}
