% What #[derive(DerefMut)] generates

Deriving `Deref` only works for structs with a single field, e.g.
newtypes. Furthermore it requires that the type also implements `Deref`, so
usually `Deref` should also be derived. The resulting implementation of `Deref
will allow you to mutably dereference the struct its member directly.

# Example usage

```rust
# #[macro_use] extern crate derive_more;
#[derive(Deref, DerefMut)]
struct MyBoxedInt(Box<i32>);

#[derive(Deref, DerefMut)]
struct NumRef<'a> {
    num: &'a mut i32,
}

fn main() {
    let mut int = 123i32;
    let mut boxed = MyBoxedInt(Box::new(int));
    let mut num_ref = NumRef{num: &mut int};
    *boxed += 1000;
    assert_eq!(1123, *boxed);
    *num_ref += 123;
    assert_eq!(246, *num_ref);
}
```

# Tuple structs

When deriving `DerefMut` for a tuple struct with one field:

```rust
# #[macro_use] extern crate derive_more;
# fn main(){}
#[derive(Deref, DerefMut)]
struct MyBoxedInt(Box<i32>);
```

Code like this will be generated to implement `DerefMut`:

```rust
# struct MyBoxedInt(Box<i32>);
# impl ::std::ops::Deref for MyBoxedInt {
#     type Target = <Box<i32> as ::std::ops::Deref>::Target;
#     #[inline]
#     fn deref(&self) -> &Self::Target {
#         <Box<i32> as ::std::ops::Deref>::deref(&self.0)
#     }
# }
impl ::std::ops::DerefMut for MyBoxedInt {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        <Box<i32> as ::std::ops::DerefMut>::deref_mut(&mut self.0)
    }
}
```

# Regular structs

When deriving `DerefMut` for a regular struct with one field:

```rust
# #[macro_use] extern crate derive_more;
# fn main(){}
#[derive(Deref, DerefMut)]
struct NumRef<'a> {
    num: &'a mut i32,
}
```

Code like this will be generated to implement `DerefMut`:

```rust
# struct NumRef<'a> {
#     num: &'a mut i32,
# }
# impl<'a> ::std::ops::Deref for NumRef<'a> {
#     type Target = <&'a mut i32 as ::std::ops::Deref>::Target;
#     #[inline]
#     fn deref(&self) -> &Self::Target {
#         <&'a mut i32 as ::std::ops::Deref>::deref(&self.num)
#     }
# }
impl<'a> ::std::ops::DerefMut for NumRef<'a> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        <&'a mut i32 as ::std::ops::DerefMut>::deref_mut(&mut self.num)
    }
}
```

# Enums

Deriving `DerefMut` is not supported for enums.
