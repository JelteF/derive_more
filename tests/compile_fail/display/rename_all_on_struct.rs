#[derive(derive_more::Display)]
enum Enum {
    #[display(rename_all = "lowercase")]
    RenameAllOnVariant,
}

#[derive(derive_more::Display)]
#[display(rename_all = "lowercase")]
struct Struct;

fn main() {}
