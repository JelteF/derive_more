trait Trait {
    type Assoc;
}

#[derive(derive_more::Borrow)]
#[borrow(forward)]
struct Foo<T: Trait>(Box<T::Assoc>);

fn main() {}
