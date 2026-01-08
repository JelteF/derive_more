# Using `#[derive(Clone)]`

Deriving `Clone` for enums/structs works similarly to the one in `std`,
by cloning all the available fields, but, in contrast:
1. Does not constrain generic parameters.

## Example usage

```rust
use derive_more::{Clone};

#[derive(Clone, Copy, PartialEq, Debug)]
struct Simple(i32);

let a = Simple(42);
let b = a.clone();
assert_eq!(a, b);
```

### Cloning non-`Clone` types references

Because this derive doesn't add any bounds to generic parameters, it can be used with references to non-`Clone` types:

```rust
use derive_more::{Clone};

struct NotClone;

#[derive(Clone)]
struct ReferenceToGenericType<'a, T>(&'a T);

let not_clone = NotClone;
let should_be_clonable = ReferenceToGenericType(&not_clone);
let clone = should_be_clonable.clone();

assert!(core::ptr::eq(should_be_clonable.0, clone.0))
```

this generates code equivalent to:
```rust
# struct NotClone;
# struct ReferenceToGenericType<'a, T>(&'a T);

impl<'a, T> Clone for ReferenceToGenericType<'a, T> {
    fn clone(&self) -> Self {
        Self(core::clone::Clone::clone(&self.0))
    }
}
```

The derive from std would have put a `T: Clone` bound on the clone impl. This is a bit too much as references are 
perfectly clonable even if the type behind in not.

### Custom trait bounds

Sometimes you may want to specify custom trait bounds on your generic type parameters.
This can be done with a `#[clone(bound(...))]` attribute.

`#[clone(bound(...))]` accepts code tokens in a format similar to the format used in
angle bracket list (or `where` clause predicates): `T: MyTrait, U: Trait1 + Trait2`.

```rust
use derive_more::Clone;

trait Foo {
    type Bar;
}

#[derive(Clone)]
#[clone(bound(T::Bar: Clone))]
struct Baz<T: Foo>(T::Bar);

// FooImpl doesn't implement Clone, but that's fine
// because only T::Bar needs to be Clone
struct FooImpl;

impl Foo for FooImpl {
    type Bar = i32;
}

let baz: Baz<FooImpl> = Baz(42);
let cloned = baz.clone();
assert_eq!(baz.0, cloned.0);
```
This generates code equivalent to:
```rust
# trait Foo { type Bar; }
# struct FooImpl;
# impl Foo for FooImpl { type Bar = i32; }
#
# struct Baz<T: Foo>(T::Bar);

impl<T: Foo> Clone for Baz<T>
where
    T::Bar: Clone, // specified via `#[clone(bound(...))]`
{
    fn clone(&self) -> Self {
        Self(core::clone::Clone::clone(&self.0))
    }
}
```
