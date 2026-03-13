# Using `#[derive(Default)]`

Deriving `Default` for structs/enums works similarly to the one in `std`,
by using the default value for all fields, but, in contrast:
1. Does not constrain generic parameters (custom bounds can be specified with `#[default(bound(...))]` attribute).
2. Allows specifying custom default values for fields using `#[default(...)]`.
3. For enums, the variant annotated with `#[default]` does not need to be a unit variant.

**Note: due to the `default` feature being used for default features of crates, this derive is exposed behind the `derive_default` feature.**

## Example usage

```rust
use derive_more::Default;

#[derive(Default, PartialEq, Debug)]
struct Simple {
    field: i32,
}

assert_eq!(Simple::default(), Simple { field: 0 });
```

### Custom default values

You can specify custom default values for fields using `#[default(...)]`:

```rust
use derive_more::Default;

#[derive(Default, PartialEq, Debug)]
struct CustomDefaults {
    #[default(42)]
    field1: i32,
    #[default(Some("hello"))]
    field2: Option<&'static str>,
}

assert_eq!(
    CustomDefaults::default(),
    CustomDefaults { field1: 42, field2: Some("hello") }
);
```

### Enums

For enums, you must mark the default variant with `#[default]`:

```rust
use derive_more::Default;

#[derive(Default, PartialEq, Debug)]
enum MyEnum {
    Variant1,
    #[default]
    Variant2 { field: i32 },
}

assert_eq!(MyEnum::default(), MyEnum::Variant2 { field: 0 });
```

The std derive would only let you annotate the first variant with `#[default]` as it requires unit variant.

### No constraints on generic bounds by default

```rust
use derive_more::Default;

struct NotDefault;

#[derive(Default)]
struct GenericType<T>(Option<T>);

let instance: GenericType<NotDefault> = GenericType::default();
assert!(instance.0.is_none());
```

This generates code equivalent to:
```rust
# struct NotDefault;
# struct GenericType<T>(Option<T>);

impl<T> Default for GenericType<T> {
    fn default() -> Self {
        Self(core::default::Default::default())
    }
}
```

The derive from std would have put a `T: Default` bound on the impl. This is unnecessary when the field type (`Option<T>`) already implements `Default` regardless of `T`.

### Custom trait bounds

Sometimes you may want to specify custom trait bounds on your generic type parameters.
This can be done with a `#[default(bound(...))]` attribute.

`#[default(bound(...))]` accepts code tokens in a format similar to the format used in
angle bracket list (or `where` clause predicates): `T: MyTrait, U: Trait1 + Trait2`.

```rust
use derive_more::Default;

trait Foo {
    type Bar;
}

#[derive(Default, PartialEq, Debug)]
#[default(bound(T::Bar: Default))]
struct Baz<T: Foo> {
    field: T::Bar,
}

// FooImpl doesn't implement Default, but that's fine
// because only T::Bar needs to be Default
struct FooImpl;

impl Foo for FooImpl {
    type Bar = i32;
}

let baz: Baz<FooImpl> = Baz::default();
assert_eq!(baz.field, 0);
```

This generates code equivalent to:
```rust
# trait Foo { type Bar; }
# struct FooImpl;
# impl Foo for FooImpl { type Bar = i32; }
#
# struct Baz<T: Foo> { field: T::Bar }

impl<T: Foo> Default for Baz<T>
where
    T::Bar: Default, // specified via `#[default(bound(...))]`
{
    fn default() -> Self {
        Self {
            field: core::default::Default::default(),
        }
    }
}
```
