#[derive(derive_more::Mul)]
#[mul(forward)]
struct Foo(#[mul(skip)] i32);

#[derive(derive_more::Mul)]
#[mul(forward)]
enum Enum {
    Bar { #[mul(skip)] x: i32, #[mul(ignore)] y: i32 },
}

fn main() {}
