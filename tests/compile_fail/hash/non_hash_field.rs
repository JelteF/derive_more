struct NonHashable(i32);

#[derive(derive_more::Hash)]
struct CantHash(NonHashable);

fn main() {}