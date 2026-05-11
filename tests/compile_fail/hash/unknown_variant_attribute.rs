#[derive(derive_more::Hash)]
enum MyEnum {
    #[hash(unknown)]
    A,
}
fn main() {}