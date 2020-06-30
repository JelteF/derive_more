% What #[derive(Index)] generates

Deriving `Index` only works for a single field of a struct.
The result is that you will index it's member directly.

It's also possible to derive for an enum as long as it can be derived for each
of the variants of the enum. Each variant should have a single field for which
`Index` should be derived (as if the variant was a struct).

With `#[index]` or `#[index(ignore)]` it's possible to indicate the field that
you want to derive `Index` for.

# Example usage

```rust
# #[macro_use] extern crate derive_more;
#[derive(Index)]
struct MyVec(Vec<i32>);

// You can specify the field you want to derive Index for
#[derive(Index)]
struct Numbers {
    #[index]
    numbers: Vec<i32>,
    useless: bool,
}

#[derive(Index)]
enum MyVecs {
    MyVec(Vec<i32>),
    Numbers {
        #[index]
        numbers: Vec<i32>,
        useless: bool,
    },
}

fn main() {
    assert_eq!(5, MyVec(vec![5, 8])[0]);
    assert_eq!(200, Numbers{numbers: vec![100, 200], useless: false}[1]);
    assert_eq!(20, MyVecs::MyVec(vec![10, 20, 30])[1]);
}
```

# Structs

When deriving `Index` for a struct:

```rust
# #[macro_use] extern crate derive_more;
# fn main(){}
#[derive(Index)]
struct Numbers {
    #[index]
    numbers: Vec<i32>,
    useless: bool,
}
```

Code like this will be generated:

```rust
# struct Numbers {
#     numbers: Vec<i32>,
#     useless: bool,
# }
impl<__IdxT, __IdxOutputT: ?::core::marker::Sized> ::core::ops::Index<__IdxT>
    for Numbers
where
    Vec<i32>: ::core::ops::Index<__IdxT, Output = __IdxOutputT>,
{
    type Output = __IdxOutputT;
    #[inline]
    fn index(&self, idx: __IdxT) -> &Self::Output {
        let indexable = &self.numbers;
        <Vec<i32> as ::core::ops::Index<__IdxT>>::index(indexable, idx)
    }
}
```

# Enums

When deriving `Index` for an enum:

```rust
# #[macro_use] extern crate derive_more;
# fn main(){}
#[derive(Index)]
enum MyVecs {
    MyVec(Vec<i32>),
    Numbers {
        #[index]
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
