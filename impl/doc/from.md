# What `#[derive(From)]` generates

The point of deriving this type is that it makes it easy to create a new
instance of the type by using the `.into()` method on the value(s) that it
should contain. This is done by implementing the `From` trait for the type
that is passed to the derive.




## Structs

For structs with a single field you can call `.into()` on the desired content
itself after deriving `From`.

```rust
# use derive_more::From;
#
#[derive(Debug, From, PartialEq)]
struct Int(i32);

assert_eq!(Int(2), 2.into());
```

For structs that have multiple fields `.into()` needs to be called on a tuple
containing the desired content for each field.

```rust
# use derive_more::From;
#
#[derive(Debug, From, PartialEq)]
struct Point(i32, i32);

assert_eq!(Point(1, 2), (1, 2).into());
```

To specify concrete types to derive convert from use `#[from(<types>)]`.

```rust
# use std::borrow::Cow;
#
# use derive_more::From;
#
#[derive(Debug, From, PartialEq)]
#[from(Cow<'static, str>, String, &'static str)]
struct Str(Cow<'static, str>);

assert_eq!(Str("&str".into()), "&str".into());
assert_eq!(Str("String".into()), "String".to_owned().into());
assert_eq!(Str("Cow".into()), Cow::Borrowed("Cow").to_owned().into());

#[derive(Debug, From, PartialEq)]
#[from((i16, i16), (i32, i32))]
struct Point {
    x: i32,
    y: i32,
}

assert_eq!(Point { x: 1_i32, y: 2_i32 }, (1_i16, 2_i16).into());
assert_eq!(Point { x: 3_i32, y: 4_i32 }, (3_i32, 4_i32).into());
```

Also, you can forward implementation to the inner type, which means deriving `From` for any type, that derives `From`
inner type.

```rust
# use std::borrow::Cow;
#
# use derive_more::From;
#
#[derive(Debug, From, PartialEq)]
#[from(forward)]
struct Str {
    inner: Cow<'static, str>,
}

assert_eq!(Str { inner: "&str".into() }, "&str".into());
assert_eq!(Str { inner: "String".into() }, "String".to_owned().into());
assert_eq!(Str { inner: "Cow".into() }, Cow::Borrowed("Cow").to_owned().into());
```

Finally, for extra flexibility, you can directly specify which fields to include
in the tuple and specify defaults for the rest. NOTE: this is currently not
supported for `#[from(forward)]` or `#[from(<types>]`; this may be alleviated in
the future.

If you add a `#[from(<default value>)]` attribute to any fields of the struct,
then those fields will be omitted from the tuple and be set to the default value
in the implementation:

```rust
# use std::collections::HashMap;
#
# use derive_more::From;
#
#[derive(Debug, From, PartialEq)]
struct MyWrapper {
    inner: u8,
    #[from(1)]
    not_important: u32,
    #[from(HashMap::new())]
    extra_properties: HashMap<String, String>,
}

assert_eq!(MyWrapper { inner: 123, not_important: 1, extra_properties: HashMap::new(), }, 123.into());
```


If you add a `#[from]` value to any fields of the struct, then only those
fields will be present in the tuple and the rest will be either set to
`Default::default()` or taken from their default values specified in
`#[from(<default value>)]`:

```rust

# use std::collections::HashMap;
#
# use derive_more::From;
#
#[derive(Debug, From, PartialEq)]
struct Location {
    #[from]
    lat: f32,
    #[from]
    lon: f32,
    #[from(String::from("Check out my location!"))]
    description: String,
    extra_properties: HashMap<String, String>,
}

// This is equivalent to:

// #[derive(Debug, From, PartialEq)]
// struct Location {
//     lat: f32,
//     lon: f32,
//     #[from(String::from("Check out my location!"))]
//     description: String,
//     #[from(Default::default())]
//     extra_properties: HashMap<String, String>,
// }


assert_eq!(
    Location {
        lat: 41.7310,
        lon: 44.8067,
        description: String::from("Check out my location!"),
        extra_properties: Default::default(),
    },
    (41.7310, 44.8067).into()
);
```


## Enums

For enums `.into()` works for each variant as if they were structs. This
includes specifying concrete types via `#[from(<types>)]` or forwarding
implementation with `#[from(forward)]`.

```rust
# use derive_more::From;
#
#[derive(Debug, From, PartialEq)]
enum IntOrPoint {
    Int(i32),
    Point {
        x: i32,
        y: i32,
    },
}

assert_eq!(IntOrPoint::Int(1), 1.into());
assert_eq!(IntOrPoint::Point { x: 1, y: 2 }, (1, 2).into());
```

By default, `From` is generated for every enum variant, but you can skip some
variants via `#[from(skip)]` (or `#[from(ignore)]`) or only concrete fields via `#[from]`.

```rust
# mod from {
# use derive_more::From;
#[derive(Debug, From, PartialEq)]
enum Int {
    #[from]
    Derived(i32),
    NotDerived(i32),
}
# }

// Is equivalent to:

# mod skip {
# use derive_more::From;
#[derive(Debug, From, PartialEq)]
enum Int {
    Derived(i32),
    #[from(skip)] // or #[from(ignore)]
    NotDerived(i32),
}
# }
```


`#[from]`/`#[from(<default value>)]` may also be used on fields of enum variants
in the same way as for struct fields.

## Example usage

```rust
# use derive_more::From;
#
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

// Or explicitly annotate the ones you need
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
    #[from(i8, i32)]
    SmallInt(i32),
    #[from(i16, i64)]
    NamedBigInt { int: i64 },
    NoFromImpl(i64),
}

assert!(MyInt(2) == 2.into());
assert!(MyInt64(6) == 6u8.into());
assert!(MyEnum::SmallInt(123) == 123i32.into());
assert!(MyEnum::SmallInt(123) != 123i64.into());
assert!(MyEnum::NamedBigInt{int: 123} == 123i64.into());
assert!(MyEnum3::SmallInt(123) == 123i8.into());
assert!(MyEnum3::NamedBigInt{int: 123} == 123i16.into());
```
