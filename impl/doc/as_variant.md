# What `#[derive(AsVariant)]` generates

When an enum is decorated with `#[derive(AsVariant)]`, for each variant `foo` in
the enum, with fields `(a, b, c, ...)`, a public instance method `as_foo(self) -> Option<(a, b, c, ...)>` is generated.
If you don't want the `as_foo` method generated for a variant you can put the
`#[as_variant(ignore)]` attribute on that variant.
If you want to treat a reference, you can put the `#[as_variant(ref)]` attribute on the enum declaration or that variant, then `as_foo_ref(self) -> Option<(&a, &b, &c, ...)>` will be generated. You can also use mutable references by putting `#[as_variant(ref_mut)]`.




## Example usage

```rust
# use derive_more::AsVariant;
#
#[derive(AsVariant)]
#[as_variant(ref)]
enum Maybe<T> {
    Just(T),
    Nothing
}

assert_eq!(Maybe::<()>::Nothing.as_nothing(), Some(()));
assert_eq!(Maybe::<()>::Nothing.as_just(), None);
assert_eq!(Maybe::Just(1).as_just(), Some(1));
assert_eq!((&Maybe::Just(42)).as_just_ref(), Some(&42));
```


### What is generated?

The derive in the above example generates code like this:
```rust
# enum Maybe<T> {
#     Just(T),
#     Nothing
# }
impl<T> Maybe<T>{
    #[must_use]
    pub fn as_just(self) -> Option<(T)> {
        match self {
            Maybe::Just(field_0) => Some((field_0)),
            _ => None,
        }
    }
    #[must_use]
    pub fn as_just_ref(&self) -> Option<(&T)> {
        match self {
            Maybe::Just(field_0) => Some((field_0)),
            _ => None,
        }
    }

    #[must_use]
    pub fn as_nothing(self) -> Option<()> {
        match self {
            Maybe::Nothing => Some(()),
            _ => None,
        }
    }
    #[must_use]
    pub fn as_nothing_ref(&self) -> Option<()> {
        match self {
            Maybe::Nothing => Some(()),
            _ => None,
        }
    }
}
```
