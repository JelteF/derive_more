% What #[derive(Deref)] generates

Deriving `Deref` only works for structs with a single field, e.g.
newtypes. The result is that you will deref it's member directly. So this is
mostly useful for newtypes that contain a pointer type such as `Box` or `Rc`.

# Example usage

```rust
# #[macro_use] extern crate derive_more;
#[derive(Deref)]
struct MyBoxedInt(Box<i32>);

#[derive(Deref)]
struct NumRef<'a> {
    num: &'a i32,
}

fn main() {
    let int = 123i32;
    let boxed = MyBoxedInt(Box::new(int));
    let num_ref = NumRef{num: &int};
    assert_eq!(123, *boxed);
    assert_eq!(123, *num_ref);
}
```

# Tuple structs

When deriving `Deref` for a tuple struct with one field:

```rust
# #[macro_use] extern crate derive_more;
# fn main(){}
#[derive(Deref)]
struct MyBoxedInt(Box<i32>);
```

Code like this will be generated:

```rust
# struct MyBoxedInt(Box<i32>);
impl ::std::ops::Deref for MyBoxedInt {
    type Target = <Box<i32> as ::std::ops::Deref>::Target;
    #[inline]
    fn deref(&self) -> &Self::Target {
        <Box<i32> as ::std::ops::Deref>::deref(&self.0)
    }
}
```

# Regular structs

When deriving `Deref` for a regular struct with one field:

```rust
# #[macro_use] extern crate derive_more;
# fn main(){}
#[derive(Deref)]
struct NumRef<'a> {
    num: &'a i32,
}
```

Code like this will be generated:

```rust
# struct NumRef<'a> {
#     num: &'a i32,
# }
impl<'a> ::std::ops::Deref for NumRef<'a> {
    type Target = <&'a i32 as ::std::ops::Deref>::Target;
    #[inline]
    fn deref(&self) -> &Self::Target {
        <&'a i32 as ::std::ops::Deref>::deref(&self.num)
    }
}
```

# Enums

Deriving `Deref` is not supported for enums.
