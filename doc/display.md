% What #[derive(Display)] generates

Deriving `Display` only works for structs with only a single field, e.g.
newtypes. The result is that you will be able to call `format!()` and
`println!()` with `"{}"` on your type.

# Example usage

```rust
# #[macro_use] extern crate derive_more;
#[derive(Display)]
struct MyInt(i32);

#[derive(Display)]
struct Point1D{
    x: i32,
}

fn main() {
    assert_eq!("5", format!("{}", MyInt(5)));
    assert_eq!("100", format!("{}", Point1D{x: 100}));
}
```


# Tuple structs

When deriving `Display` for a tuple struct with one field:

```rust
# #[macro_use] extern crate derive_more;
# fn main(){}
#[derive(Display)]
struct MyInt(i32);
```

Code like this will be generated:

```rust
# struct MyInt(i32);
impl ::std::fmt::Display for MyInt {
    fn fmt(&self, formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        <i32 as ::std::fmt::Display>::fmt(&self.0, formatter)
    }
}
```


# Regular structs


When deriving `Display` for a regular struct with one field:

```rust
# #[macro_use] extern crate derive_more;
# fn main(){}
#[derive(Display)]
struct Point1D {
    x: i32,
}
```

Code like this will be generated:

```rust
# struct Point1D {
#     x: i32,
# }
impl ::std::fmt::Display for Point1D {
    fn fmt(&self, formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        <i32 as ::std::fmt::Display>::fmt(&self.x, formatter)
    }
}
```

# Enums

Deriving `Display` is not supported for enums.
