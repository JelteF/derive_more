struct F1;

#[derive(derive_more::TryFrom)]
#[try_from(usize)]
enum Enum {
    Field(F1),
    Field2(F1),
}

fn main() {}
