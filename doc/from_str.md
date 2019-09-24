% What #[derive(FromStr)] generates

Deriving `FromStr` only works for newtypes, i.e structs with only a single
field. The result is that you will be able to call the `parse()` method on a
string to convert it to your newtype. This only works when the type that is
contained in the type implements `FromStr`.

# Example usage

```rust
# #[macro_use] extern crate derive_more;
#[derive(FromStr, Debug, Eq, PartialEq)]
struct MyInt(i32);

#[derive(FromStr, Debug, Eq, PartialEq)]
struct Point1D{
    x: i32,
}

fn main() {
    assert_eq!(MyInt(5), "5".parse().unwrap());
    assert_eq!(Point1D{x: 100}, "100".parse().unwrap());
}
```

# Tuple structs

When deriving `FromStr` for a tuple struct with one field:

```rust
# #[macro_use] extern crate derive_more;
# fn main(){}
#[derive(FromStr)]
struct MyInt(i32);
```

Code like this will be generated:

```rust
# struct MyInt(i32);
impl ::std::str::FromStr for MyInt {
    type Err = <i32 as ::std::str::FromStr>::Err;
    fn from_str(src: &str) -> Result<Self, Self::Err> {
        return Ok(MyInt(i32::from_str(src)?));
    }
}
```

# Regular structs

When deriving `FromStr` for a regular struct with one field:

```rust
# #[macro_use] extern crate derive_more;
# fn main(){}
#[derive(FromStr)]
struct Point1D {
    x: i32,
}
```

Code like this will be generated:

```rust
# struct Point1D {
#     x: i32,
# }
impl ::std::str::FromStr for Point1D {
    type Err = <i32 as ::std::str::FromStr>::Err;
    fn from_str(src: &str) -> Result<Self, Self::Err> {
        return Ok(Point1D {
            x: i32::from_str(src)?,
        });
    }
}
```

# Enums

Deriving `FromStr` is not supported for enums.
