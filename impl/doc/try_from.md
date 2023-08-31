# What `#[derive(TryFrom)]` generates

This derive allows you to convert enum discriminants into their corresponding variants.
By default a `TryFrom<isize>` is generated, matching the [type of the discriminant](https://doc.rust-lang.org/reference/items/enumerations.html#discriminants).
The type can be changed with a `#[repr(u/i*)]` attribute, e.g., `#[repr(u8)]` or `#[repr(i32)]`.
Only field-less variants can be constructed from their variant, therefor the `TryFrom` implementation will return an error for a discriminant representing a variant with fields.

## Example usage

```rust
# #[rustversion::since(1.66)]
# mod discriminant_on_non_unit_enum {
# use derive_more::TryFrom;
#[derive(TryFrom, Debug, PartialEq)]
#[repr(u32)]
enum Enum {
    Implicit,
    Explicit = 5,
    Field(usize),
    Empty{},
}

# #[rustversion::since(1.66)]
# pub fn test(){
assert_eq!(Enum::Implicit, Enum::try_from(0).unwrap());
assert_eq!(Enum::Explicit, Enum::try_from(5).unwrap());
assert_eq!(Enum::Empty{}, Enum::try_from(7).unwrap());

// variants with fields are not supported
assert!(Enum::try_from(6).is_err());
# }
# }
# // We need to use a `function` declaration, because we cannot put `rustversion` on a statement.
# #[rustversion::since(1.66)] use discriminant_on_non_unit_enum::test;
# #[rustversion::before(1.66)] fn test() {}
# test();
```
