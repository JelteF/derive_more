# What `#[derive(BorrowMut)]` generates

Deriving `BorrowMut` generates an implementation of
`core::borrow::BorrowMut` that mutably borrows a single-field struct as its
field.

`BorrowMut<T>` requires `Borrow<T>`, so the type must also implement the
matching `Borrow` implementation. Usually this means deriving both `Borrow` and
`BorrowMut`.




## Newtypes and Structs with One Field

When `BorrowMut` is derived for a newtype or a struct with one field, a single
implementation is generated for the field type.

```rust
# use derive_more::{Borrow, BorrowMut};
#
#[derive(Borrow, BorrowMut)]
struct MyWrapper(String);
```

Generates code equivalent to:

```rust
# struct MyWrapper(String);
# impl core::borrow::Borrow<String> for MyWrapper {
#     fn borrow(&self) -> &String {
#         &self.0
#     }
# }
impl core::borrow::BorrowMut<String> for MyWrapper {
    fn borrow_mut(&mut self) -> &mut String {
        &mut self.0
    }
}
```

It's also possible to use the `#[borrow_mut(forward)]` attribute to forward to
the field's `BorrowMut` implementation. The matching `Borrow` implementation
must borrow the same type, so forwarded `BorrowMut` usually goes together with
`#[borrow(forward)]`.

```rust
# use derive_more::{Borrow, BorrowMut};
# use derive_more::core::borrow::BorrowMut as _;
#
#[derive(Borrow, BorrowMut)]
#[borrow(forward)]
#[borrow_mut(forward)]
struct MyWrapper(String);

let mut item = MyWrapper("test".to_owned());
let _: &mut str = item.borrow_mut();
```

This generates code equivalent to:

```rust
# struct MyWrapper(String);
# impl<T: ?Sized> core::borrow::Borrow<T> for MyWrapper
# where
#     String: core::borrow::Borrow<T>,
# {
#     #[inline]
#     fn borrow(&self) -> &T {
#         self.0.borrow()
#     }
# }
impl<T: ?Sized> core::borrow::BorrowMut<T> for MyWrapper
where
    String: core::borrow::BorrowMut<T>,
{
    #[inline]
    fn borrow_mut(&mut self) -> &mut T {
        self.0.borrow_mut()
    }
}
```

Forwarding cannot be derived through a generic parameter, an associated type of
a generic parameter, or a forwarding pointer to either. Directly borrowing an
associated type of a generic parameter is not supported either. These shapes can
overlap with `core`'s blanket `impl<T> BorrowMut<T> for T`.




## Structs with Multiple Fields

Deriving `BorrowMut` for structs with more than one field is not supported.

```rust,compile_fail
# use derive_more::{Borrow, BorrowMut};
#
#[derive(Borrow, BorrowMut)]
struct User(String, bool);
```




## Enums

Deriving `BorrowMut` for enums is not supported.
