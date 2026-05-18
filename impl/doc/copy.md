# Using `#[derive(Copy)]`


Deriving ` Copy` for enums/structs works similarly to the one in `std`, but, in contrast:
1. Does not constrain generic parameters.

Note: `Copy` requires `Clone`, so you should derive it as well using [`Clone`].

## Example usage

```rust
use derive_more::{Clone, Copy};

#[derive(Clone, Copy, PartialEq, Debug)]
struct Simple(i32);

let a = Simple(42);
let b = a;
assert_eq!(a, b);
```

### Copying non-`Copy` types references

Because this derive doesn't add any bounds to generic parameters, it can be used with references to non-`Copy` types:

```rust
use derive_more::{Clone, Copy};

struct NotCopy;

#[derive(Clone, Copy)]
struct ReferenceToGenericType<'a, T>(&'a T);

let not_copy = NotCopy;
let should_be_copyable = ReferenceToGenericType(&not_copy);
let copy = should_be_copyable;

assert!(core::ptr::eq(should_be_copyable.0, copy.0))
```

this generates code equivalent to:
```rust
# use derive_more::{Clone};
# struct NotCopy;
# #[derive(Clone)]
# struct ReferenceToGenericType<'a, T>(&'a T);
impl<'a, T> Copy for ReferenceToGenericType<'a, T> { }
```

The derive from std would have put a `T: Copy` bound on the copy impl. This is a bit too much as references are 
perfectly copyable even if the type behind in not.

### Custom trait bounds

Sometimes you may want to specify custom trait bounds on your generic type parameters.
This can be done with a `#[copy(bound(...))]` attribute.

`#[copy(bound(...))]` accepts code tokens in a format similar to the format used in
angle bracket list (or `where` clause predicates): `T: MyTrait, U: Trait1 + Trait2`.

```rust
use derive_more::{Clone, Copy};

trait Foo {
    type Bar;
}

#[derive(Clone, Copy)]
#[copy(bound(T::Bar: Copy))]
#[clone(bound(T::Bar: Clone))]
struct Baz<T: Foo>(T::Bar);

// FooImpl doesn't implement Clone or Copy, but that's fine
// because only T::Bar needs to be Copy
struct FooImpl;

impl Foo for FooImpl {
    type Bar = i32;
}

let baz: Baz<FooImpl> = Baz(42);
let copy = baz;
assert_eq!(baz.0, copy.0);
```

This generates code equivalent to:
```rust
# use derive_more::{Clone};
# trait Foo { type Bar; }
# struct FooImpl;
# impl Foo for FooImpl { type Bar = i32; }
#
# #[derive(Clone)]
# #[clone(bound(T::Bar: Clone))]
# struct Baz<T: Foo>(T::Bar);

impl<T: Foo> Copy for Baz<T>
where
    T::Bar: Copy, // specified via `#[copy(bound(...))]`
{
}
```