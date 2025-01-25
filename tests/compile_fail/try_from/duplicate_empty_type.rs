#[derive(derive_more::TryFrom)]
#[try_from(usize)]
enum Enum {
    Field,
    Field2,
}

fn main() {}
