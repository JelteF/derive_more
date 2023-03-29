# What `#[derive(TryIntoUnwrap)]` generates

This works almost like `Unwrap`.
When an enum is decorated with `#[derive(TryIntoUnwrap)]`, for each variant `foo` in the enum, with fields `(a, b, c, ...)` a public instance method `try_into_foo(self) -> Result<(a, b, c, ...), Self>` is generated.
If you don't want the `try_into_foo` method generated for a variant, you can put the `#[try_into_variant(ignore)]` attribute on that variant.
If you want to treat a reference, you can put the `#[try_into_variant(ref)]` attribute on the enum declaration or that variant, then `try_into_foo_ref(self) -> Result<(&a, &b, &c, ...), &Self>` will be generated. You can also use mutable references by putting `#[unwrap(ref_mut)]`.
However, unlike `Unwrap`, it does not panic if the conversion fails. Also, values that fail to convert are not dropped but returned as `Err`.

## Example usage

```rust
use derive_more::TryIntoVariant;

#[derive(TryIntoVariant)]
#[try_into_variant(ref)]
enum Maybe<T> {
    Nothing,
    Just(T),
}

fn main() {
    assert_eq!(Maybe::Just(1).try_into_just(), Ok(1));

    // Unlike `Unwrap`, it does not panic.
    assert_eq!(Maybe::Nothing.try_into_just(), Err(Maybe::Nothing)); // and the value is returned!
    assert_eq!(Maybe::Just(2).try_into_nothing(), Err(Maybe::Just(2)));

    assert_eq!(*(&Maybe::Just(42)).unwrap_just_ref(), Ok(42));
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
    pub fn try_into_nothing(self) -> Result<(), Self> {
        match self {
            Maybe::Nothing() => Ok(()),
            val @ _ => Err(val),
        }
    }
    pub fn try_into_nothing_ref(&self) -> Result<(), &Self> {
        match self {
            Maybe::Nothing() => Ok(()),
            val @ _ => Err(val),
        }
    }
    pub fn try_into_just(self) -> Result<T, Self> {
        match self {
            Maybe::Just(field_0) => Ok(field_0),
            val @ _ => Err(val),
        }
    }
    pub fn try_into_just_ref(&self) -> Result<&T, &Self> {
        match self {
            Maybe::Just(field_0) => Ok(field_0),
            val @ _ => Err(val),
        }
    }
}
```
