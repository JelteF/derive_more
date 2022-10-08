# What `#[derive(Unwrap)]` generates

When an enum is decorated with `#[derive(Unwrap)]`, for each variant `foo` in the enum,
with fields `(a, b, c, ...)` a public instance method `unwrap_foo(self) -> (a, b, c, ...)` and `try_unwrap_foo(self) -> Option<(a, b, c, ...)>`
are generated. If you don't want the `unwrap_foo` method generated for a variant,
you can put the `#[unwrap(ignore)]` attribute on that variant. If you want to treat a reference, you can put the `#[unwrap(ref)]` attribute on enum declaration or that variant and `unwrap_foo_ref(self) -> (&a, &b, &c, ...)` and `try_unwrap_foo_ref(self) -> Option<(&a, &b, &c, ...)>` will be generated.




## Example usage

```rust
# use derive_more::Unwrap;
#
#[derive(Unwrap)]
#[unwrap(ref)]
enum Maybe<T> {
    Just(T),
    Nothing,
}

fn main() {
    assert_eq!(Maybe::Just(1).unwrap_just(), 1);
    assert_eq!(Maybe::Just(2).try_unwrap_just(), Some(2));
    assert_eq!(Maybe::Nothing.try_unwrap_just(), None);
    assert_eq!(*(&Maybe::Just(42)).unwrap_just_ref(), 42);
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
            Maybe::Nothing => panic!(),
        }
    }

    pub fn try_unwrap_just(self) -> Option<(T)> {
        match self {
            Maybe::Just(field_0) => Some((field_0)),
            Maybe::Nothing => None,
        }
    }

    pub fn unwrap_just_ref(&self) -> (&T) {
        match self {
            Maybe::Just(field_0) => (field_0),
            Maybe::Nothing => panic!(),
        }
    }

    pub fn try_unwrap_just_ref(&self) -> Option<(&T)> {
        match self {
            Maybe::Just(field_0) => Some((field_0)),
            Maybe::Nothing => None,
        }
    }

    pub fn unwrap_nothing(self) -> () {
        match self {
            Maybe::Nothing => (),
            Maybe::Just(..) => panic!(),
        }
    }

    pub fn try_unwrap_nothing(self) -> Option<()> {
        match self {
            Maybe::Nothing => Some(()),
            Maybe::Just(..) => None,
        }
    }

    pub fn unwrap_nothing_ref(&self) -> () {
        match self {
            Maybe::Nothing => (),
            Maybe::Just(..) => panic!(),
        }
    }

    pub fn try_unwrap_nothing_ref(&self) -> Option<()> {
        match self {
            Maybe::Nothing => Some(()),
            Maybe::Just(..) => None,
        }
    }
}