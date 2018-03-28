% What #[derive(Index)] generates

Deriving `Index` only works for structs with only a single field, e.g.
newtypes. The result is that you will index it's member directly.

# Example usage

```rust
# #[macro_use] extern crate derive_more;
#[derive(Index)]
struct MyVec(Vec<i32>);

#[derive(Index)]
struct Numbers {
    numbers: Vec<i32>,
}

fn main() {
    assert_eq!(5, MyVec(vec![5, 8])[0]);
    assert_eq!(200, Numbers{numbers: vec![100, 200]}[1]);
}
```


# Tuple structs

When deriving `Index` for a tuple struct with one field:

```rust
# #[macro_use] extern crate derive_more;
# fn main(){}
#[derive(Index)]
struct MyVec(Vec<i32>);
```

Code like this will be generated:

```rust
# struct MyVec(Vec<i32>);
impl<__IdxT> ::std::ops::Index<__IdxT> for MyVec
where
    Vec<i32>: ::std::ops::Index<__IdxT>,
{
    type Output = <Vec<i32> as ::std::ops::Index<__IdxT>>::Output;
    #[inline]
    fn index(&self, idx: __IdxT) -> &Self::Output {
        <Vec<i32> as ::std::ops::Index<__IdxT>>::index(&self.0, idx)
    }
}
```


# Regular structs


When deriving `Index` for a regular struct with one field:

```rust
# #[macro_use] extern crate derive_more;
# fn main(){}
#[derive(Index)]
struct Numbers {
    numbers: Vec<i32>,
}
```

Code like this will be generated:

```rust
# struct Numbers {
#     numbers: Vec<i32>,
# }
impl<__IdxT> ::std::ops::Index<__IdxT> for Numbers
where
    Vec<i32>: ::std::ops::Index<__IdxT>,
{
    type Output = <Vec<i32> as ::std::ops::Index<__IdxT>>::Output;
    #[inline]
    fn index(&self, idx: __IdxT) -> &Self::Output {
        <Vec<i32> as ::std::ops::Index<__IdxT>>::index(&self.numbers, idx)
    }
}
```

# Enums

Deriving `Index` is not supported for enums.
