# What `#[derive(TryFrom)]` generates

Derive `TryFrom` allows you to convert enum discriminants into their corresponding variants.




## Enums

Enums can be generated either from a `repr` discriminant value or a custom type.

### Repr

In the `repr` mode, by default, a `TryFrom<isize>` is generated, matching the
[type of the
discriminant](https://doc.rust-lang.org/reference/items/enumerations.html#discriminants).
The type can be changed with a `#[repr(u/i*)]` attribute, e.g., `#[repr(u8)]` or
`#[repr(i32)]`.  Only field-less variants can be constructed from their variant,
therefore the `TryFrom` implementation will return an error for a discriminant
representing a variant with fields.

```rust
# use derive_more::TryFrom;
#
#[derive(TryFrom, Debug, PartialEq)]
#[try_from(repr)]
#[repr(u32)]
enum Enum {
    ImplicitZero,
    ExplicitFive = 5,
    FieldSix(usize),
    EmptySeven{},
}

assert_eq!(Enum::ImplicitZero, Enum::try_from(0).unwrap());
assert_eq!(Enum::ExplicitFive, Enum::try_from(5).unwrap());
assert_eq!(Enum::EmptySeven{}, Enum::try_from(7).unwrap());

// Variants with fields are not supported, as the value for their fields would be undefined.
assert!(Enum::try_from(6).is_err());
```

### Custom Types ("non-repr")

Rather situationally, `TryFrom<T>` can be implemented if all the variant types
have `TryFrom<T>`.

```rust
# use derive_more::TryFrom;
#
/// A custom error can be defined or not.
#[derive(Debug, PartialEq, Eq)]
enum Error {
    FromEnum,
    FromVariant,
}

#[derive(Debug, PartialEq, Eq)]
struct F1;

impl TryFrom<usize> for F1 {
    type Error = Error;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        if value == 1 {
            return Ok(Self);
        }
        Err(Error::FromVariant)
    }
}

#[derive(Debug, PartialEq, Eq)]
struct F2;

impl TryFrom<usize> for F2 {
    type Error = Error;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        if value == 2 {
            return Ok(Self);
        }
        Err(Error::FromVariant)
    }
}

assert_eq!(Err(Error::FromVariant), F2::try_from(3));

#[derive(TryFrom, Debug, PartialEq, Eq)]
#[try_from(
    // the type for which all variants have `TryFrom<T>`
    usize,
    // optional: the error type (default is `()`).
    Error,
    // optional: the constructor of the type (optional if err is as `struct E;`)
    Error::FromEnum
)]
enum Enum {
    Field(F1),
    Field2 { x: F2 },
}

assert_eq!(Enum::Field(F1), Enum::try_from(1).unwrap());
assert_eq!(Enum::Field2 { x: F2 }, Enum::try_from(2).unwrap());
assert_eq!(Err(Error::FromEnum), Enum::try_from(3));
```

Multi-field variants as in `Enum::Field(F1, F2)` are also supported however may
rarely be used.

Since `TryFrom<T> for ()` is too universal, non-repr conversions do not support
enums with empty (unit or fieldless) variants.
