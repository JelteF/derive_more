# What `#[derive(Into)]` generates

This derive creates the the exact opposite of [`#[derive(From)]`](crate::From).
Instead of allowing you to create a new instance of the struct from the values
it should contain, it allows you to extract the values from the struct.
One thing to note is that this derive doesn't actually generate an
implementation for the `Into` trait.
Instead it derives `From` for the values contained in the struct and thus has an
indirect implementation of `Into` as recommended by the
[docs](https://doc.rust-lang.org/core/convert/trait.Into.html).




## Example usage

```rust
# use derive_more::Into;
#
// Allow converting into i32
#[derive(Into, PartialEq)]
struct MyInt(i32);

// Additionally convert refs to the inner type refs
#[derive(Into, PartialEq)]
#[into(owned, ref, ref_mut)]
struct MyInt64(i64);

// Specify additional conversions
#[derive(Into, PartialEq)]
#[into(types(i16, i32))]
struct MyInt8(i8);

// Even for ref types
#[derive(Into, PartialEq)]
#[into(owned, ref(types(i64)))]
struct MyInt64Wrapped(MyInt64);

assert!(i32::from(MyInt(2)) == 2i32);
assert!(i64::from(MyInt64(6)) == 6i64);
assert!(<&i64>::from(&MyInt64(6)) == &6i64);
assert!(<&mut i64>::from(&mut MyInt64(6)) == &mut 6i64);
assert!(i8::from(MyInt8(7)) == 7i8);
assert!(i16::from(MyInt8(7)) == 7i16);
assert!(i32::from(MyInt8(7)) == 7i32);
assert!(MyInt64::from(MyInt64Wrapped(MyInt64(1))) == MyInt64(1));
assert!(<&MyInt64>::from(&MyInt64Wrapped(MyInt64(1))) == &MyInt64(1));
assert!(<&i64>::from(&MyInt64Wrapped(MyInt64(1))) == &1i64);
```




## Tuple structs

When deriving `Into` for a tuple struct with a single field (i.e. a newtype) like this:

```rust
# use derive_more::Into;
#
#[derive(Into)]
struct MyInt(i32);
```

Code like this will be generated:

```rust
# struct MyInt(i32);
impl ::core::convert::From<MyInt> for (i32) {
    fn from(original: MyInt) -> (i32) {
        (original.0)
    }
}
```

The behaviour is a bit different when deriving for a struct with multiple
fields, since it returns a tuple. For instance when deriving for a tuple struct
with two fields like this:

```rust
# use derive_more::Into;
#
#[derive(Into)]
struct MyInts(i32, i32);
```

Code like this will be generated:

```rust
# struct MyInts(i32, i32);
impl ::core::convert::From<MyInts> for (i32, i32) {
    fn from(original: MyInts) -> (i32, i32) {
        (original.0, original.1)
    }
}
```




## Regular structs

For regular structs almost the same code is generated as for tuple structs
except in the way the field values are assigned to the new struct.
When deriving for a regular struct with a single field like this:

```rust
# use derive_more::Into;
#
#[derive(Into)]
struct Point1D {
    x: i32,
}
```

Code like this will be generated:

```rust
# struct Point1D {
#     x: i32,
# }
impl ::core::convert::From<Point1D> for (i32) {
    fn from(original: Point1D) -> (i32) {
        (original.x)
    }
}
```

The behaviour is again a bit different when deriving for a struct with multiple
fields, because this also returns a tuple. For instance when deriving for a
tuple struct with two fields like this:

```rust
# use derive_more::Into;
#
#[derive(Into)]
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
impl ::core::convert::From<Point2D> for (i32, i32) {
    fn from(original: Point2D) -> (i32, i32) {
        (original.x, original.y)
    }
}
```




## Enums

Deriving `Into` for enums is not supported as it would not always be successful.
This is what the currently unstable
[`TryInto`](https://doc.rust-lang.org/core/convert/trait.TryInto.html) should be
used for, which is currently not supported by this library.
