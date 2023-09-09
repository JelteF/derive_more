# What `#[derive(TryFrom)]` generates

This derive allows you to convert enum discriminants into their corresponding variants.
By default a `TryFrom<isize>` is generated, matching the [type of the discriminant](https://doc.rust-lang.org/reference/items/enumerations.html#discriminants).
The type can be changed with a `#[repr(u/i*)]` attribute, e.g., `#[repr(u8)]` or `#[repr(i32)]`.
Only field-less variants can be constructed from their variant, therefor the `TryFrom` implementation will return an error for a discriminant representing a variant with fields.

## Example usage

```rust
# use derive_more::TryFrom;
#[derive(TryFrom, Debug, PartialEq)]
#[try_from(repr)]
#[repr(u32)]
enum Enum {
    Implicit,
    Explicit = 5,
    Field(usize),
    Empty{},
}

assert_eq!(Enum::Implicit, Enum::try_from(0).unwrap());
assert_eq!(Enum::Explicit, Enum::try_from(5).unwrap());
assert_eq!(Enum::Empty{}, Enum::try_from(7).unwrap());

// Variants with fields are not supported, as the value for their fields would be undefined.
assert!(Enum::try_from(6).is_err());
```
