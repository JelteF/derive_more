#[derive(derive_more::From)]
#[from(types(i32, "&str"))]
struct Foo(String);

fn main() {}
