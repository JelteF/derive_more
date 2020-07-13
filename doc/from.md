% What #[derive(From)] generates

The point of deriving this type is that it makes it easy to create a new
instance of the type by using the `.into()` method on the value(s) that it
should contain.
This is done by implementing the `From` trait for the type that is passed to the
derive.
For structs with a single field you can call `.into()` on the desired content
itself after deriving `From`.
For structs that have multiple fields `.into()` needs to be called on a tuple
containing the desired content for each field.
For enums `.into()` works for each variant as if they were structs.
This way the variant can not only be initialized, but also be chosen based on
the type that `.into()` is called on.

# Example usage

```rust
# #[macro_use] extern crate derive_more;

// Allow converting from i32
#[derive(From, PartialEq)]
struct MyInt(i32);

// Forward from call to the field, so allow converting
// from anything that can be converted into an i64 (so most integers)
#[derive(From, PartialEq)]
#[from(forward)]
struct MyInt64(i64);

// You can ignore a variant
#[derive(From, PartialEq)]
enum MyEnum {
    SmallInt(i32),
    NamedBigInt { int: i64 },
    #[from(ignore)]
    NoFromImpl(i64),
}

// Or explicitly annotate the once you need
#[derive(From, PartialEq)]
enum MyEnum2 {
    #[from]
    SmallInt(i32),
    #[from]
    NamedBigInt { int: i64 },
    NoFromImpl(i64),
}

// And even specify additional conversions for them
#[derive(From, PartialEq)]
enum MyEnum3 {
    #[from(types(i8))]
    SmallInt(i32),
    #[from(types(i16))]
    NamedBigInt { int: i64 },
    NoFromImpl(i64),
}

fn main() {
    assert!(MyInt(2) == 2.into());
    assert!(MyInt64(6) == 6u8.into());
    assert!(MyEnum::SmallInt(123) == 123i32.into());
    assert!(MyEnum::SmallInt(123) != 123i64.into());
    assert!(MyEnum::NamedBigInt{int: 123} == 123i64.into());
    assert!(MyEnum3::SmallInt(123) == 123i8.into());
    assert!(MyEnum3::NamedBigInt{int: 123} == 123i16.into());
}
```

# Tuple structs

When deriving for a tuple struct with a single field (i.e. a newtype) like this:

```rust
# #[macro_use] extern crate derive_more;
# fn main(){}
#[derive(From)]
struct MyInt(i32);

```

Code like this will be generated:

```rust
# struct MyInt(i32);
impl ::core::convert::From<(i32)> for MyInt {
    fn from(original: (i32)) -> MyInt {
        MyInt(original)
    }
}
```

The behaviour is a bit different when deriving for a struct with multiple
fields. For instance when deriving for a tuple struct with two fields like this:

```rust
# #[macro_use] extern crate derive_more;
# fn main(){}
#[derive(From)]
struct MyInts(i32, i32);
```

Code like this will be generated:

```rust
# struct MyInts(i32, i32);
impl ::core::convert::From<(i32, i32)> for MyInts {
    fn from(original: (i32, i32)) -> MyInts {
        MyInts(original.0, original.1)
    }
}
```

# Regular structs

For regular structs almost the same code is generated as for tuple structs
except in the way the field values are assigned to the new struct.
When deriving for a regular struct with a single field like this:

```rust
# #[macro_use] extern crate derive_more;
# fn main(){}
#[derive(From)]
struct Point1D {
    x: i32,
}
```

Code like this will be generated:

```rust
# struct Point1D {
#     x: i32,
# }
impl ::core::convert::From<(i32)> for Point1D {
    fn from(original: (i32)) -> Point1D {
        Point1D { x: original }
    }
}
```

The behaviour is a bit different when deriving for a struct with multiple
fields. For instance when deriving for a tuple struct with two fields like this:

```rust
# #[macro_use] extern crate derive_more;
# fn main(){}
#[derive(From)]
struct Point2D {
    x: i32,
    y: i32,
}

```

Code like this will be generated:

```rust
# struct Point2D {
#     x: i32,
#     y: i32,
# }
impl ::core::convert::From<(i32, i32)> for Point2D {
    fn from(original: (i32, i32)) -> Point2D {
        Point2D {
            x: original.0,
            y: original.1,
        }
    }
}
```

# Enums

When deriving `From` for enums a new `impl` will be generated for each of its
variants.
If you don't want this for a variant you can put the `#[from(ignore)]` attribute
on that variant. One case where this can be useful is when two variants would
overlap.
For instance when deriving `From` for the following enum:

```rust
# #[macro_use] extern crate derive_more;
# fn main(){}
#[derive(From)]
enum MixedInts {
    SmallInt(i32),
    NamedBigInt { int: i64 },
    TwoSmallInts(i32, i32),
    NamedBigInts { x: i64, y: i64 },
    #[from(ignore)]
    Unsigned(u32),
    NamedUnsigned { x: u32 },
}

```

Code like this will be generated:

```rust
# enum MixedInts {
#     SmallInt(i32),
#     NamedBigInt { int: i64 },
#     TwoSmallInts(i32, i32),
#     NamedBigInts { x: i64, y: i64 },
#     Unsigned(u32),
#     NamedUnsigned { x: u32 },
# }
impl ::core::convert::From<(i32)> for MixedInts {
    #[allow(unused_variables)]
    #[inline]
    fn from(original: (i32)) -> MixedInts {
        MixedInts::SmallInt(original)
    }
}

impl ::core::convert::From<(i64)> for MixedInts {
    #[allow(unused_variables)]
    #[inline]
    fn from(original: (i64)) -> MixedInts {
        MixedInts::NamedBigInt { int: original }
    }
}

impl ::core::convert::From<(i32, i32)> for MixedInts {
    #[allow(unused_variables)]
    #[inline]
    fn from(original: (i32, i32)) -> MixedInts {
        MixedInts::TwoSmallInts(original.0, original.1)
    }
}

impl ::core::convert::From<(i64, i64)> for MixedInts {
    #[allow(unused_variables)]
    #[inline]
    fn from(original: (i64, i64)) -> MixedInts {
        MixedInts::NamedBigInts {
            x: original.0,
            y: original.1,
        }
    }
}

impl ::core::convert::From<(u32)> for MixedInts {
    #[allow(unused_variables)]
    #[inline]
    fn from(original: (u32)) -> MixedInts {
        MixedInts::NamedUnsigned { x: original }
    }
}
```

Without the `#[from(ignore)]` on `Unsigned`, no `impl` would be generated for
`Unsigned` and `NamedUnsigned`. The reason for this is that it would be
impossible for the compiler to know which implementation to choose, since they
would both implement `From<u32>`.
