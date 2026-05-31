trait Trait {
    type Assoc;
}

#[derive(derive_more::BorrowMut)]
struct Foo<T: Trait>(T::Assoc);

fn main() {}
