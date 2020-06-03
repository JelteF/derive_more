% What #[derive(TryInto)] generates

This derive allows you to convert enum variants into their corresponding
variant types.
One thing to note is that this derive doesn't actually generate an
implementation for the `TryInto` trait.
Instead it derives `TryFrom` for each variant in the enum and thus has an
indirect implementation of `TryInto` as recommended by the
[docs](https://doc.rust-lang.org/core/convert/trait.TryInto.html).

By using `#[try_into(owned, ref, ref_mut)]` it's possible to derive a `TryInto`
implementation for reference types as well.
You can pick any combination of `owned`, `ref` and `ref_mut`.
If that's not provided the default is `#[try_into(owned)]`.

With `#[try_into]` or `#[try_into(ignore)]` it's possible to indicate which
variants you want to derive `TryInto` for.

In case of an error, the original value is not destructed but returned.
This enables the chaining of several `try_from` (or `try_into`) calls.

# Example usage

```rust
# #[macro_use] extern crate derive_more;
use core::convert::TryFrom;
use core::convert::TryInto;
#[derive(TryInto, Clone, Eq, PartialEq, Debug)]
#[try_into(owned, ref, ref_mut)]
enum MixedData {
    Int(u32),
    String(String),
}

fn main() {
    let string = MixedData::String("foo".to_string());
    let int = MixedData::Int(123);
    assert_eq!(Ok(123u32), int.clone().try_into());
    assert_eq!(Ok(&123u32), (&int.clone()).try_into());
    assert_eq!(Ok(&mut 123u32), (&mut int.clone()).try_into());
    assert_eq!("foo".to_string(), String::try_from(string.clone()).unwrap());
    assert!(u32::try_from(string).is_err());
}
```

# Structs

Deriving `TryInto` for structs is not supported because there is no failing
mode. Use `#[derive(Into)]` instead. `TryInto` will automatically get a
blanket implementation through `TryFrom`, automatically derived from `From`,
which `#[derive(Into)]` produces.

# Enums

When deriving `TryInto` for an enum, each enum variant gets its own
`TryFrom` implementation.
For instance, when deriving `TryInto` for an enum link this:

```rust
# #[macro_use] extern crate derive_more;
# fn main(){}
#[derive(TryInto)]
enum MixedInts {
    SmallInt(i32),
    BigInt(i64),
    TwoSmallInts(i32, i32),
    NamedSmallInts { x: i64, y: i64 },
    UnsignedOne(u32),
    UnsignedTwo(u32),
    #[try_into(ignore)]
    NotImportant,
}
```

Code like this will be generated:

```rust
# enum MixedInts {
#     SmallInt(i32),
#     BigInt(i64),
#     TwoSmallInts(i32, i32),
#     NamedSmallInts { x: i64, y: i64 },
#     UnsignedOne(u32),
#     UnsignedTwo(u32),
# }
impl ::core::convert::TryFrom<MixedInts> for (i32) {
    type Error = MixedInts;
    fn try_from(value: MixedInts) -> Result<Self, Self::Error> {
        match value {
            MixedInts::SmallInt(__0) => Ok(__0),
            _ => Err(value),
        }
    }
}
impl ::core::convert::TryFrom<MixedInts> for (i64) {
    type Error = MixedInts;
    fn try_from(value: MixedInts) -> Result<Self, Self::Error> {
        match value {
            MixedInts::BigInt(__0) => Ok(__0),
            _ => Err(value),
        }
    }
}
impl ::core::convert::TryFrom<MixedInts> for (i32, i32) {
    type Error = MixedInts;
    fn try_from(value: MixedInts) -> Result<Self, Self::Error> {
        match value {
            MixedInts::TwoSmallInts(__0, __1) => Ok((__0, __1)),
            _ => Err(value),
        }
    }
}
impl ::core::convert::TryFrom<MixedInts> for (i64, i64) {
    type Error = MixedInts;
    fn try_from(value: MixedInts) -> Result<Self, Self::Error> {
        match value {
            MixedInts::NamedSmallInts { x: __0, y: __1 } => Ok((__0, __1)),
            _  => Err(value),
        }
    }
}
impl ::core::convert::TryFrom<MixedInts> for (u32) {
    type Error = MixedInts;
    fn try_from(value: MixedInts) -> Result<Self, Self::Error> {
        match value {
            MixedInts::UnsignedOne(__0) | MixedInts::UnsignedTwo(__0) => Ok(__0),
            _ => Err(value),
        }
    }
}
```

When deriving `TryInto` for an enum with Unit variants like this:

```rust
# #[macro_use] extern crate derive_more;
# fn main(){}
#[derive(TryInto)]
enum EnumWithUnit {
    SmallInt(i32),
    Unit,
}
```

Code like this will be generated:

```rust
# enum EnumWithUnit {
#     SmallInt(i32),
#     Unit,
# }
impl ::core::convert::TryFrom<EnumWithUnit> for (i32) {
    type Error = EnumWithUnit;
    fn try_from(value: EnumWithUnit) -> Result<Self, Self::Error> {
        match value {
            EnumWithUnit::SmallInt(__0) => Ok(__0),
            _ => Err(value),
        }
    }
}
impl ::core::convert::TryFrom<EnumWithUnit> for () {
    type Error = EnumWithUnit;
    fn try_from(value: EnumWithUnit) -> Result<Self, Self::Error> {
        match value {
            EnumWithUnit::Unit => Ok(()),
            _ => Err(value),
        }
    }
}
```
