% What #[derive(MulAssign)] generates

This code is very similar to the code that is generated for `#[derive(Add)]`.
The difference is that it mutates the existing instance instead of creating a
new one.

# Tuple structs

When deriving `MulAssign` for a tuple struct with two fields like this:

```rust
# #[macro_use] extern crate derive_more;

#[derive(MulAssign)]
struct MyInts(i32, i32);

# fn main(){}
```

Code like this will be generated:

```rust
# struct MyInts(i32, i32);
# fn main(){}

impl<__RhsT: ::std::marker::Copy> ::std::ops::MulAssign<__RhsT> for MyInts
    where i32: ::std::ops::MulAssign<__RhsT>
{
    fn mul_assign(&mut self, rhs: __RhsT) {
        self.0.mul_assign(rhs);
        self.1.mul_assign(rhs);
    }
}
```

The behaviour is similar with more or less fields, except for the fact that
`__RhsT` does not need to implement `Copy` when only a single field is present.



# Regular structs


When deriving `MulAssign` for a regular struct with two fields like this:

```rust
# #[macro_use] extern crate derive_more;

#[derive(MulAssign)]
struct Point2D {
    x: i32,
    y: i32,
}
# fn main(){}
```

Code like this will be generated:

```rust
# struct Point2D {
#     x: i32,
#     y: i32,
# }

impl<__RhsT: ::std::marker::Copy> ::std::ops::MulAssign<__RhsT> for Point2D
    where i32: ::std::ops::MulAssign<__RhsT>
{
    fn mul_assign(&mut self, rhs: __RhsT) {
        self.x.mul_assign(rhs);
        self.y.mul_assign(rhs);
    }
}
```

The behaviour is again similar with more or less fields, except that `Copy`
doesn't have to be implemented for `__Rhst` when the struct has only a single
field.


# Enums

Deriving `MulAssign` for enums is not (yet) supported.
This has two reason, the first being that deriving `Mul` is also not implemented
for enums yet.
The second reason is the same as for `AddAssign`.
Even if it would be deriving `Mul` was implemented it would likely return a
`Result<EnumType>` instead of an `EnumType`.
Handling the case where it errors would be hard and maybe impossible.
