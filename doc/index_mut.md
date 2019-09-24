% What #[derive(IndexMut)] generates

Deriving `IndexMut` only works for structs with only a single field, e.g.
newtypes. Furthermore it requires that the type also implements `Index`, so
usually `Index` should also be derived. The result is that you will mutably
index it's member directly.

# Example usage

```rust
# #[macro_use] extern crate derive_more;
#[derive(Index, IndexMut)]
struct MyVec(Vec<i32>);

#[derive(Index, IndexMut)]
struct Numbers {
    numbers: Vec<i32>,
}

fn main() {
    let mut myvec = MyVec(vec![5, 8]);
    myvec[0] = 50;
    assert_eq!(50, myvec[0]);

    let mut numbers = Numbers{numbers: vec![100, 200]};
    numbers[1] = 400;
    assert_eq!(400, numbers[1]);
}
```

# Tuple structs

When deriving `IndexMut` for a tuple struct with one field:

```rust
# #[macro_use] extern crate derive_more;
# fn main(){}
#[derive(Index, IndexMut)]
struct MyVec(Vec<i32>);
```

Code like this will be generated to implement `IndexMut`:

```rust
# struct MyVec(Vec<i32>);
# impl<__IdxT> ::std::ops::Index<__IdxT> for MyVec
# where
#     Vec<i32>: ::std::ops::Index<__IdxT>,
# {
#     type Output = <Vec<i32> as ::std::ops::Index<__IdxT>>::Output;
#     #[inline]
#     fn index(&self, idx: __IdxT) -> &Self::Output {
#         <Vec<i32> as ::std::ops::Index<__IdxT>>::index(&self.0, idx)
#     }
# }
impl<__IdxT> ::std::ops::IndexMut<__IdxT> for MyVec
where
    Vec<i32>: ::std::ops::IndexMut<__IdxT>,
{
    #[inline]
    fn index_mut(&mut self, idx: __IdxT) -> &mut Self::Output {
        <Vec<i32> as ::std::ops::IndexMut<__IdxT>>::index_mut(&mut self.0, idx)
    }
}
```

# Regular structs

When deriving `IndexMut` for a regular struct with one field:

```rust
# #[macro_use] extern crate derive_more;
# fn main(){}
#[derive(Index, IndexMut)]
struct Numbers {
    numbers: Vec<i32>,
}
```

Code like this will be generated to implement `IndexMut`:

```rust
# struct Numbers {
#     numbers: Vec<i32>,
# }
# impl<__IdxT> ::std::ops::Index<__IdxT> for Numbers
# where
#     Vec<i32>: ::std::ops::Index<__IdxT>,
# {
#     type Output = <Vec<i32> as ::std::ops::Index<__IdxT>>::Output;
#     #[inline]
#     fn index(&self, idx: __IdxT) -> &Self::Output {
#         <Vec<i32> as ::std::ops::Index<__IdxT>>::index(&self.numbers, idx)
#     }
# }
impl<__IdxT> ::std::ops::IndexMut<__IdxT> for Numbers
where
    Vec<i32>: ::std::ops::IndexMut<__IdxT>,
{
    #[inline]
    fn index_mut(&mut self, idx: __IdxT) -> &mut Self::Output {
        <Vec<i32> as ::std::ops::IndexMut<__IdxT>>::index_mut(&mut self.numbers, idx)
    }
}

```

# Enums

Deriving `IndexMut` is not supported for enums.
