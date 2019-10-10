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
impl ::std::convert::From<(i32)> for MyInt {
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
impl ::std::convert::From<(i32, i32)> for MyInts {
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
impl ::std::convert::From<(i32)> for Point1D {
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
impl ::std::convert::From<(i32, i32)> for Point2D {
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

```edition2018
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

Notice that for `Unsigned` and `NamedUnsigned` no `impl` is generated.
The reason for this is that it would be impossible for the compiler to know
which implementation to choose, since they would both implement `From<u32>`.
