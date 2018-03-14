% What #[derive(FromStr)] generates

Deriving `FromStr` only works for newtypes, i.e structs with only a single
field. The result is that you will be able to call the `parse()` method on a
string to convert it to your newtype. This only works when the type that is
contained in the type implements `FromStr`.

# Example usage

```rust
struct MyInt(i32)

fn main() {
    let my_5 = "5"::parse()
}
```

# Tuple structs

When deriving for a tuple struct with a single field (i.e. a newtype) like this:

```rust
#[derive(Mul)]
struct MyInt(i32)
```

Code like this will be generated:

```rust
impl<__RhsT> ::std::ops::Mul<__RhsT> for MyInt
    where i32: ::std::ops::Mul<__RhsT, Output = i32>
{
    type Output = MyInt;
    fn mul(self, rhs: __RhsT) -> MyInt {
        MyInt(self.0.mul(rhs))
    }
}
```

# Regular structs

When deriving `Mul` for a regular struct with a single field like this:

```rust
#[derive(Mul)]
struct Point1D {
    x: i32,
}
```

Code like this will be generated:

```rust
impl<__RhsT> ::std::ops::Mul<__RhsT> for Point1D
    where i32: ::std::ops::Mul<__RhsT, Output = i32>
{
    type Output = Point1D;
    fn mul(self, rhs: __RhsT) -> Point1D {
        Point1D { x: self.x.mul(rhs) }
    }
}
```

# Enums

Deriving `FromStr` for enums is not supported. The main reason for this being
that it is not clear into which variant the result should be parsed.
