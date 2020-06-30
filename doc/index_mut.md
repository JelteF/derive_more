% What #[derive(IndexMut)] generates

Deriving `IndexMut` only works for a single field of a struct.
Furthermore it requires that the type also implements `Index`, so usually
`Index` should also be derived.
The result is that you will mutably index it's member directly.

It's also possible to derive for an enum as long as it can be derived for each
of the variants of the enum. Each variant should have a single field for which
`IndexMut` should be derived (as if the variant was a struct).

With `#[index_mut]` or `#[index_mut(ignore)]` it's possible to indicate the
field that you want to derive `IndexMut` for.

# Example usage

```rust
# #[macro_use] extern crate derive_more;
#[derive(Index, IndexMut)]
struct MyVec(Vec<i32>);

#[derive(Index, IndexMut)]
struct Numbers {
    #[index]
    #[index_mut]
    numbers: Vec<i32>,
    useless: bool,
}

#[derive(Index, IndexMut)]
enum MyVecs {
    MyVec(Vec<i32>),
    Numbers {
        #[index]
        #[index_mut]
        numbers: Vec<i32>,
        useless: bool,
    },
}


fn main() {
    let mut myvec = MyVec(vec![5, 8]);
    myvec[0] = 50;
    assert_eq!(50, myvec[0]);

    let mut numbers = Numbers{numbers: vec![100, 200], useless: false};
    numbers[1] = 400;
    assert_eq!(400, numbers[1]);
    let mut my_vec_enum = MyVecs::MyVec(vec![10, 20, 30]);
    my_vec_enum[2] = 40;
    assert_eq!(40, my_vec_enum[2]);

}
```

# Regular structs

When deriving `IndexMut` for a struct:

```rust
# #[macro_use] extern crate derive_more;
# fn main(){}
#[derive(Index, IndexMut)]
struct Numbers {
    #[index]
    #[index_mut]
    numbers: Vec<i32>,
    useless: bool,
}
```

Code like this will be generated to implement `IndexMut`:

```rust
# struct Numbers {
#     numbers: Vec<i32>,
#     useless: bool,
# }
# impl<__IdxT> ::core::ops::Index<__IdxT> for Numbers
# where
#     Vec<i32>: ::core::ops::Index<__IdxT>,
# {
#     type Output = <Vec<i32> as ::core::ops::Index<__IdxT>>::Output;
#     #[inline]
#     fn index(&self, idx: __IdxT) -> &Self::Output {
#         <Vec<i32> as ::core::ops::Index<__IdxT>>::index(&self.numbers, idx)
#     }
# }
impl<__IdxT> ::core::ops::IndexMut<__IdxT> for Numbers
where
    Vec<i32>: ::core::ops::IndexMut<__IdxT>,
{
    #[inline]
    fn index_mut(&mut self, idx: __IdxT) -> &mut Self::Output {
        <Vec<i32> as ::core::ops::IndexMut<__IdxT>>::index_mut(&mut self.numbers, idx)
    }
}
```


# Enums

When deriving `Index` for an enum:

```rust
# #[macro_use] extern crate derive_more;
# fn main(){}
#[derive(Index, IndexMut)]
enum MyVecs {
    MyVec(Vec<i32>),
    Numbers {
        #[index]
        #[index_mut]
        numbers: Vec<i32>,
        useless: bool,
    },
}

```

Code like this will be generated:

```rust
# enum MyVecs {
#     MyVec(Vec<i32>),
#     Numbers {
#         numbers: Vec<i32>,
#         useless: bool,
#     },
# }
# impl<__IdxT, __IdxOutputT: ?::core::marker::Sized> ::core::ops::IndexMut<__IdxT>
#     for MyVecs
# where
#     Vec<i32>: ::core::ops::IndexMut<__IdxT, Output = __IdxOutputT>,
#     Vec<i32>: ::core::ops::IndexMut<__IdxT, Output = __IdxOutputT>,
# {
#     #[inline]
#     fn index_mut(&mut self, idx: __IdxT) -> &mut Self::Output {
#         match self {
#             MyVecs::MyVec(indexable) => {
#                 <Vec<i32> as ::core::ops::IndexMut<__IdxT>>::index_mut(indexable, idx)
#             }
#             MyVecs::Numbers {
#                 numbers: indexable,
#                 useless: _,
#             } => <Vec<i32> as ::core::ops::IndexMut<__IdxT>>::index_mut(indexable, idx),
#         }
#     }
# }
impl<__IdxT, __IdxOutputT: ?::core::marker::Sized> ::core::ops::Index<__IdxT> for MyVecs
where
    Vec<i32>: ::core::ops::Index<__IdxT, Output = __IdxOutputT>,
    Vec<i32>: ::core::ops::Index<__IdxT, Output = __IdxOutputT>,
{
    type Output = __IdxOutputT;
    #[inline]
    fn index(&self, idx: __IdxT) -> &Self::Output {
        match self {
            MyVecs::MyVec(indexable) => {
                <Vec<i32> as ::core::ops::Index<__IdxT>>::index(indexable, idx)
            }
            MyVecs::Numbers {
                numbers: indexable,
                useless: _,
            } => <Vec<i32> as ::core::ops::Index<__IdxT>>::index(indexable, idx),
        }
    }
}
```
