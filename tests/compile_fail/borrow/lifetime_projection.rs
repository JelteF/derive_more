trait Trait {
    type Assoc;
}

#[derive(derive_more::Borrow)]
struct Foo<'a>(<&'a i32 as Trait>::Assoc)
where
    &'a i32: Trait;

fn main() {}
