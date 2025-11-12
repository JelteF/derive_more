#[derive(derive_more::Mul)]
#[mul(forward)]
struct Foo(#[mul(unknown)] i32);

#[derive(derive_more::Mul)]
#[mul(forward)]
enum Enum {
    Bar { #[mul(unknown)] i: i32 },
}

fn main() {}
