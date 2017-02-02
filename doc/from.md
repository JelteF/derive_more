% What #[derive(From)] generates

The point of deriving this type is that it makes it easy to create a new
instance of the type by using the `.into()` method on the value(s) that it
should contain.
For (tuple) structs with a single field this is done by calling `.into()` on
the desired content itself.
For structs that have multiple fields `.into()` needs to be called on a tuple
containing the desired content for each field.
Enums have a bit different semantics as can be read in the [Enums](#enums)
section below.

# Tuple structs

When deriving for a tuple struct with a single field (i.e. a newtype) like this:

```rust
#[derive(From)]
struct MyInt(i32)
```

Code like this will be generated:

```rust
impl ::std::convert::From<i32> for MyInt {
    fn from(original: i32) -> MyInt {
        MyInt(original)
    }
}
```

The behaviour is a bit different when deriving for a struct with multiple
fields. For instance when deriving for a tuple struct with two fields like this:

```rust
#[derive(From)]
struct MyInts(i32, i32)
```

Code like this will be generated:

```rust
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
#[derive(From)]
struct Point1D {
    x: i32,
}
```

Code like this will be generated:

```rust
impl ::std::convert::From<i32> for Point1D {
    fn from(original: i32) -> Point1D {
        Point1D { x: original }
    }
}
```

The behaviour is a bit different when deriving for a struct with multiple
fields. For instance when deriving for a tuple struct with two fields like this:

```rust
#[derive(From)]
struct Point2D {
    x: i32,
    y: i32,
}

```

Code like this will be generated:

```rust
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
Currently this is only done for the variants of the enum that are newtypes.
For instance When deriving for the following enum:

```rust
#[derive(From)]
enum MixedInts {
    SmallInt(i32),
    BigInt(i64),
    TwoSmallInts(i32, i32),
    NamedSmallInts { x: i32, y: i32 },
    UnsignedOne(u32),
    UnsignedTwo(u32),
}
```

Code like this will be generated:

```rust
impl ::std::convert::From<i32> for MixedInts {
    fn from(original: i32) -> MixedInts {
        MixedInts::SmallInt(original)
    }
}
impl ::std::convert::From<i64> for MixedInts {
    fn from(original: i64) -> MixedInts {
        MixedInts::BigInt(original)
    }
}
```

Notice that for `UnsignedOne` and `UnsignedTwo` no `impl` is generated, even
though they are newtypes. The reason for this is that it would be impossible for
the compiler to know which implementation to choose, since they have the would
both implement `From<u32>`.
