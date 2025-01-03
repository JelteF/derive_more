#[derive(derive_more::TryFrom)]
#[try_from(usize)]
enum Enum {
    Field,
}

fn main() {}
