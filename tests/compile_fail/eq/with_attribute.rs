fn always_eq<T>(_: &T, _: &T) -> bool {
    true
}

#[derive(derive_more::Eq)]
struct Foo(#[eq(with(always_eq))] i32);

impl PartialEq for Foo {
    fn eq(&self, _: &Self) -> bool {
        unimplemented!()
    }
}

#[derive(derive_more::Eq)]
enum Enum {
    Bar {
        #[eq(with(always_eq))]
        i: i32,
    },
}

impl PartialEq for Enum {
    fn eq(&self, _: &Self) -> bool {
        unimplemented!()
    }
}

fn main() {}
