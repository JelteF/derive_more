# What `#[derive(Borrow)]` generates

Deriving `Borrow` generates an implementation of `core::borrow::Borrow` that
borrows a single-field struct as its field.

`Borrow` has stronger semantic requirements than `AsRef`: equality, ordering and
hashing of the borrowed value are expected to match those of the owning value.
For that reason, this derive only supports structs with exactly one field.




## Newtypes and Structs with One Field

When `Borrow` is derived for a newtype or a struct with one field, a single
implementation is generated for the field type.

```rust
# use derive_more::Borrow;
#
#[derive(Borrow)]
struct MyWrapper(String);
```

Generates code equivalent to:

```rust
# struct MyWrapper(String);
impl core::borrow::Borrow<String> for MyWrapper {
    fn borrow(&self) -> &String {
        &self.0
    }
}
```

It's also possible to use the `#[borrow(forward)]` attribute to forward to the
field's `Borrow` implementation.

```rust
# use derive_more::Borrow;
# use derive_more::core::borrow::Borrow as _;
#
#[derive(Borrow)]
#[borrow(forward)]
struct MyWrapper(String);

let item = MyWrapper("test".to_owned());
let _: &str = item.borrow();
```

This generates code equivalent to:

```rust
# struct MyWrapper(String);
impl<T: ?Sized> core::borrow::Borrow<T> for MyWrapper
where
    String: core::borrow::Borrow<T>,
{
    #[inline]
    fn borrow(&self) -> &T {
        self.0.borrow()
    }
}
```

Forwarding cannot be derived through a generic parameter, an associated type of
a generic parameter, or a forwarding pointer to either. Directly borrowing an
associated type of a generic parameter is not supported either. These shapes can
overlap with `core`'s blanket `impl<T> Borrow<T> for T`.




## Structs with Multiple Fields

Deriving `Borrow` for structs with more than one field is not supported.

```rust,compile_fail
# use derive_more::Borrow;
#
#[derive(Borrow)]
struct User(String, bool);
```




## Enums

Deriving `Borrow` for enums is not supported.
