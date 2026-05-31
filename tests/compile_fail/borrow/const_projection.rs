trait Trait {
    type Assoc;
}

#[derive(derive_more::Borrow)]
struct Foo<const N: usize>(<[i32; N] as Trait>::Assoc)
where
    [i32; N]: Trait;

fn main() {}
