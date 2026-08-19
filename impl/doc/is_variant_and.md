# What `#[derive(IsVariantAnd)]` generates

When an enum is decorated with `#[derive(IsVariantAnd)]`, for each variant `foo`
in the enum a public instance method `is_foo_and(&self, f) -> bool` is generated.
It returns `true` if the value is the `foo` variant *and* the closure `f`,
applied to the variant's fields (by reference), returns `true`. This mirrors
[`Option::is_some_and`], but for arbitrary enums.

The closure receives the variant's fields as a tuple of references, in
declaration order. As with a regular tuple, a variant with a single field
collapses to a plain reference (so `Just(T)` yields `is_just_and(|x: &T| ...)`,
exactly like [`Option::is_some_and`]), while a unit variant's method takes a
`FnOnce() -> bool` closure instead.

If you don't want the `is_foo_and` method generated for a variant you can put
the `#[is_variant_and(ignore)]` attribute on that variant.

[`Option::is_some_and`]: https://doc.rust-lang.org/core/option/enum.Option.html#method.is_some_and




## Example usage

```rust
# use derive_more::IsVariantAnd;
#
#[derive(IsVariantAnd)]
enum Maybe<T> {
    Just(T),
    Nothing,
}

let maybe = Maybe::Just(42);
assert!(maybe.is_just_and(|x| *x == 42));
assert!(!maybe.is_just_and(|x| *x == 0));
assert!(!maybe.is_nothing_and(|| true));

let nothing = Maybe::<i32>::Nothing;
assert!(nothing.is_nothing_and(|| true));
assert!(!nothing.is_just_and(|x| *x == 42));
```


### What is generated?

The derive in the above example generates code like this:
```rust
# enum Maybe<T> {
#     Just(T),
#     Nothing,
# }
impl<T> Maybe<T> {
    #[must_use]
    pub fn is_just_and(&self, f: impl FnOnce(&T) -> bool) -> bool {
        match self {
            Self::Just(field_0) => f(field_0),
            _ => false,
        }
    }
    #[must_use]
    pub fn is_nothing_and(&self, f: impl FnOnce() -> bool) -> bool {
        match self {
            Self::Nothing => f(),
            _ => false,
        }
    }
}
```
