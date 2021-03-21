% What #[derive(IsVariant)] generates

When an enum is decorated with `#[derive(IsVariant)]`, for each variant `foo` in the enum,
a public instance method `is_foo(&self) -> bool` is generated.

## Example
```rust
# #[macro_use] extern crate derive_more;
# fn main(){
#   assert!(Maybe::<()>::Nothing.is_nothing());
#   assert!(!Maybe::<()>::Nothing.is_just());
# }
#[derive(IsVariant)]
enum Maybe<T> {
    Just(T),
    Nothing
}
```
generates these methods:

```rust
# enum Maybe<T> {
#     Just(T),
#     Nothing
# }
impl <T> Maybe<T>{
    pub fn is_just(&self) -> bool {
        match self {Self::Just(..) => true, _ => false}
    }
    pub fn is_nothing(&self) -> bool {
        match self {Self::Nothing => true, _ => false}
    }
}
```
