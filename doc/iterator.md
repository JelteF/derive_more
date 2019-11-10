% Using #[derive(Iterator)]

Deriving `Iterator` only works for a single field of a struct.
The result is that all the iterator method calls will forwarded to this field.

With `#[iterator]` or `#[iterator(ignore)]` it's possible to indicate the field
that you want to derive `Iterator` for.

# Example usage

```rust
# #[macro_use] extern crate derive_more;
use ::std::slice::Iter;
#[derive(Iterator)]
struct MyIter<'a>(Iter<'a, i32>);

// You can specify the field you want to derive Iterator for
#[derive(Iterator)]
struct CoolIter<'a> {
    #[iterator]
    numbers: Iter<'a, i32>,
    cool: bool,
}

fn main() {
    let vec1 = vec![1, 2, 3];
    let mut my_iter = MyIter(vec1.iter());
    assert_eq!(Some(&1), iter.());
    assert_eq!(Some(&1), iter.next());
    assert_eq!(Some(&1), my_iter.next());
    assert_eq!(Some(&2), my_iter.next());
    assert_eq!(Some(&3), my_iter.next());
    let vec2 = vec![4, 5, 6];
    let mut my_iter = CoolIter{numbers: vec2.iter(), cool: true};
    assert_eq!(Some(&4), my_iter.next());
    assert_eq!(Some(&5), my_iter.next());
    assert_eq!(Some(&6), my_iter.next());
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
impl<__IdxT> ::core::ops::Index<__IdxT> for Numbers
where
    Vec<i32>: ::core::ops::Index<__IdxT>,
{
    type Output = <Vec<i32> as ::core::ops::Index<__IdxT>>::Output;
    #[inline]
    fn index(&self, idx: __IdxT) -> &Self::Output {
        <Vec<i32> as ::core::ops::Index<__IdxT>>::index(&self.numbers, idx)
    }
}

```

# Enums

Deriving `Index` is not supported for enums.
