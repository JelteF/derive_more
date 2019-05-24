% What #[derive(TryInto)] generates

This derive allows you to convert enum variants into their corresponding
variant types.
One thing to note is that this derive doesn't actually generate an
implementation for the `TryInto` trait.
Instead it derives `TryFrom` for each variant in the enum and thus has an
indirect implementation of `TryInto` as recommended by the
[docs](https://doc.rust-lang.org/core/convert/trait.TryInto.html).

# Example usage

```rust
#![feature(try_from)]
# #[macro_use] extern crate derive_more;
use std::convert::TryFrom;
use std::convert::TryInto;
#[derive(TryInto, Clone)]
enum MixedData {
    Int(u32),
    String(String),
}

fn main() {
    let string = MixedData::String("foo".to_string());
    let int = MixedData::Int(123);
    assert_eq!(123u32, int.try_into().unwrap());
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
#![feature(try_from)]
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
}
```

Code like this will be generated:

```rust
# #![feature(try_from)]
# enum MixedInts {
#     SmallInt(i32),
#     BigInt(i64),
#     TwoSmallInts(i32, i32),
#     NamedSmallInts { x: i64, y: i64 },
#     UnsignedOne(u32),
#     UnsignedTwo(u32),
# }
impl ::std::convert::TryFrom<MixedInts> for (i32) {
    type Error = &'static str;
    fn try_from(value: MixedInts) -> Result<Self, Self::Error> {
        match value {
            MixedInts::SmallInt(__0) => Ok(__0),
            _ => Err("Only SmallInt can be converted to i32"),
        }
    }
}
impl ::std::convert::TryFrom<MixedInts> for (i64) {
    type Error = &'static str;
    fn try_from(value: MixedInts) -> Result<Self, Self::Error> {
        match value {
            MixedInts::BigInt(__0) => Ok(__0),
            _ => Err("Only BigInt can be converted to i64"),
        }
    }
}
impl ::std::convert::TryFrom<MixedInts> for (i32, i32) {
    type Error = &'static str;
    fn try_from(value: MixedInts) -> Result<Self, Self::Error> {
        match value {
            MixedInts::TwoSmallInts(__0, __1) => Ok((__0, __1)),
            _ => Err("Only TwoSmallInts can be converted to (i32, i32)"),
        }
    }
}
impl ::std::convert::TryFrom<MixedInts> for (i64, i64) {
    type Error = &'static str;
    fn try_from(value: MixedInts) -> Result<Self, Self::Error> {
        match value {
            MixedInts::NamedSmallInts { x: __0, y: __1 } => Ok((__0, __1)),
            _ => Err("Only NamedSmallInts can be converted to (i64, i64)"),
        }
    }
}
impl ::std::convert::TryFrom<MixedInts> for (u32) {
    type Error = &'static str;
    fn try_from(value: MixedInts) -> Result<Self, Self::Error> {
        match value {
            MixedInts::UnsignedOne(__0) | MixedInts::UnsignedTwo(__0) => Ok(__0),
            _ => Err("Only UnsignedOne, UnsignedTwo can be converted to u32"),
        }
    }
}
```

When deriving `TryInto` for an enum with Unit variants like this:

```rust
#![feature(try_from)]
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
# #![feature(try_from)]
# enum EnumWithUnit {
#     SmallInt(i32),
#     Unit,
# }
impl ::std::convert::TryFrom<EnumWithUnit> for (i32) {
    type Error = &'static str;
    fn try_from(value: EnumWithUnit) -> Result<Self, Self::Error> {
        match value {
            EnumWithUnit::SmallInt(__0) => Ok(__0),
            _ => Err("Only SmallInt can be converted to i32"),
        }
    }
}
impl ::std::convert::TryFrom<EnumWithUnit> for () {
    type Error = &'static str;
    fn try_from(value: EnumWithUnit) -> Result<Self, Self::Error> {
        match value {
            EnumWithUnit::Unit => Ok(()),
            _ => Err("Only Unit can be converted to ()"),
        }
    }
}
```
