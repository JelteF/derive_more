trait Trait {
    type Assoc;
}

#[derive(derive_more::Borrow)]
struct Foo<T: Trait>(<T as Trait>::Assoc);

fn main() {}
