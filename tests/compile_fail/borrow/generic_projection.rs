trait Trait {
    type Assoc;
}

#[derive(derive_more::Borrow)]
struct Foo<T>(<Vec<T> as Trait>::Assoc)
where
    Vec<T>: Trait;

fn main() {}
