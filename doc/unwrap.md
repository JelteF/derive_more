# What `#[derive(Unwrap)]` generates

When an enum is decorated with `#[derive(Unwrap)]`, for each variant `foo` in the enum,
with fields `(a, b, c, ...)` a public instance method `unwrap_foo(self) -> (a, b, c, ...)`
is generated. If you don't want the `unwrap_foo` method generated for a variant,
you can put the `#[unwrap(ignore)]` attribute on that variant.




## Example usage

```rust
# use derive_more::Unwrap;
#
#[derive(Unwrap)]
enum Maybe<T> {
    Just(T),
    Nothing,
}
```


### What is generated?

The derive in the above example code generates the following code:
```rust
# enum Maybe<T> {
#     Just(T),
#     Nothing,
# }
impl<T> Maybe<T> {
    pub fn unwrap_just(self) -> (T) {
        match self {
            Maybe::Just(field_0) => (field_0),
            Maybe::Nothing => panic!(concat!("called `", stringify!(Maybe), "::", stringify!(unwrap_just),
                                     "()` on a `", stringify!(Nothing), "` value"))
        }
    }
    pub fn unwrap_nothing(self) -> () {
        match self {
            Maybe::Nothing => (),
            Maybe::Just(..) => panic!(concat!("called `", stringify!(Maybe), "::", stringify!(unwrap_nothing),
                                     "()` on a `", stringify!(Just), "` value"))
        }
    }
}
```
