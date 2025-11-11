#[derive(derive_more::Add)]
enum Enum {
    #[add(unknown)]
    Bar { i: i32 },
}

fn main() {}
