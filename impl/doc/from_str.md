# What `#[derive(FromStr)]` generates

Deriving `FromStr` only works for enums/structs with no fields
or newtypes (structs with only a single field). The result is
that you will be able to call the `parse()` method on a string
to convert it to your newtype. This only works when the wrapped
type implements `FromStr` itself.




## Forwarding

Deriving forwarding implementation is only supported for newtypes
(structs with only a single field).


### Tuple structs

When deriving `FromStr` for a tuple struct with one field:
```rust
# use derive_more::FromStr;
#
#[derive(FromStr, Debug, Eq, PartialEq)]
struct MyInt(i32);

assert_eq!("5".parse::<MyInt>().unwrap(), MyInt(5));
```

Code like this is generated:
```rust
# struct MyInt(i32);
impl derive_more::core::str::FromStr for MyInt {
    type Err = <i32 as derive_more::core::str::FromStr>::Err;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(i32::from_str(s)?))
    }
}
```


### Regular structs

When deriving `FromStr` for a regular struct with one field:
```rust
# use derive_more::FromStr;
#
#[derive(FromStr, Debug, Eq, PartialEq)]
struct Point1D {
    x: i32,
}

assert_eq!("100".parse::<Point1D>().unwrap(), Point1D { x: 100 });
```

Code like this is generated:
```rust
# struct Point1D {
#     x: i32,
# }
impl derive_more::core::str::FromStr for Point1D {
    type Err = <i32 as derive_more::core::str::FromStr>::Err;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self {
            x: i32::from_str(s)?,
        })
    }
}
```




## Flat representation

Deriving flat string representation is only supported for empty enums and
structs (with no fields).


### Empty enums

When deriving `FromStr` for enums with empty variants, it will generate a
`from_str()` method converting strings matching the variant name to the variant.
If using a case-insensitive match would give a unique variant (i.e. you don't have
both `MyEnum::Foo` and `MyEnum::foo` variants), then case-insensitive matching will
be used, otherwise it will fall back to exact string matching.

Since the string may not match any variants an error type is needed, so the
`derive_more::FromStrError` is used for that purpose.

Given the following enum:
```rust
# use derive_more::FromStr;
#
#[derive(FromStr, Debug, Eq, PartialEq)]
enum EnumNoFields {
    Foo,
    Bar,
    Baz,
    BaZ,
}

assert_eq!("foo".parse::<EnumNoFields>().unwrap(), EnumNoFields::Foo);
assert_eq!("Foo".parse::<EnumNoFields>().unwrap(), EnumNoFields::Foo);
assert_eq!("FOO".parse::<EnumNoFields>().unwrap(), EnumNoFields::Foo);

assert_eq!("Bar".parse::<EnumNoFields>().unwrap(), EnumNoFields::Bar);
assert_eq!("bar".parse::<EnumNoFields>().unwrap(), EnumNoFields::Bar);

assert_eq!("Baz".parse::<EnumNoFields>().unwrap(), EnumNoFields::Baz);
assert_eq!("BaZ".parse::<EnumNoFields>().unwrap(), EnumNoFields::BaZ);
assert_eq!(
    "other".parse::<EnumNoFields>().unwrap_err().to_string(),
    "Invalid `EnumNoFields` string representation",
);
```

Code like this is generated:
```rust
# enum EnumNoFields {
#     Foo,
#     Bar,
#     Baz,
#     BaZ,
# }
#
impl derive_more::core::str::FromStr for EnumNoFields {
    type Err = derive_more::FromStrError;
    fn from_str(s: &str) -> Result<Self, derive_more::FromStrError> {
        Ok(match s.to_lowercase().as_str() {
            "foo" => Self::Foo,
            "bar" => Self::Bar,
            "baz" if s == "Baz" => Self::Baz,
            "baz" if s == "BaZ" => Self::BaZ,
            _ => return Err(derive_more::FromStrError::new("EnumNoFields")),
        })
    }
}
```


### Empty structs

Deriving `FromStr` for structs with no fields is similar to enums,
but involves only case-insensitive matching by now.

Given the following struct:
```rust
# use derive_more::FromStr;
#
#[derive(FromStr, Debug, Eq, PartialEq)]
struct Foo;

assert_eq!("foo".parse::<Foo>().unwrap(), Foo);
assert_eq!("Foo".parse::<Foo>().unwrap(), Foo);
assert_eq!("FOO".parse::<Foo>().unwrap(), Foo);
```

Code like this is generated:
```rust
# struct Foo;
#
impl derive_more::core::str::FromStr for Foo {
    type Err = derive_more::FromStrError;
    fn from_str(s: &str) -> Result<Self, derive_more::FromStrError> {
        Ok(match s.to_lowercase().as_str() {
            "foo" => Self,
            _ => return Err(derive_more::FromStrError::new("Foo")),
        })
    }
}
```
