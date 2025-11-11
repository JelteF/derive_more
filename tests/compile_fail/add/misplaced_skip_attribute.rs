#[derive(derive_more::Add)]
#[add(ignore)]
struct Foo(i32);

#[derive(derive_more::Add)]
enum Enum {
    #[add(skip)]
    Bar { i: i32 },
}

fn main() {}
