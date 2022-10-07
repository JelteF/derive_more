% What #[derive(BorrowMut)] generates

Deriving `BorrowMut` generates one or more implementations of `BorrowMut`, each
corresponding to one of the fields of the decorated type.
This allows types which contain some `T` to be passed anywhere that an
`BorrowMut<T>` is accepted.

Note that `BorrowMut<T>` expects the type to also implement `Borrow<T>`.

# Newtypes and Structs with One Field

When `BorrowMut` is derived for a newtype or struct with one field, a single
implementation is generated to expose the underlying field.

```rust
# #[macro_use] extern crate derive_more;
# use core::borrow::Borrow;
# use core::borrow::BorrowMut;
# fn main(){}
#[derive(Borrow, BorrowMut)]
struct MyWrapper(String);
```

Generates:

```rust
# #[macro_use] extern crate derive_more;
# use core::borrow::Borrow;
# use core::borrow::BorrowMut;
# #[derive(Borrow)]
# struct MyWrapper(String);
impl BorrowMut<String> for MyWrapper {
    fn borrow_mut(&mut self) -> &mut String {
        &mut self.0
    }
}
```

It's also possible to use the `#[borrow_mut(forward)]` attribute to forward
to the `borrow_mut` implementation of the field. So here `SigleFieldForward`
implements all `BorrowMut` for all types that `Vec<i32>` implements `BorrowMut` for.

```rust
# #[macro_use] extern crate derive_more;
# use core::borrow::Borrow;
# use core::borrow::BorrowMut;
#[derive(Borrow, BorrowMut)]
#[borrow(forward)]
#[borrow_mut(forward)]
struct SingleFieldForward(Vec<i32>);

fn main() {
    let mut item = SingleFieldForward(vec![]);
    let _: &mut [i32] = (&mut item).borrow_mut();
}

```

This generates:

```rust
# #[macro_use] extern crate derive_more;
# #[derive(Borrow)]
# #[borrow(forward)]
# struct SingleFieldForward(Vec<i32>);
impl<__BorrowMutT: ?::core::marker::Sized> ::core::borrow::BorrowMut<__BorrowMutT> for SingleFieldForward
where
    Vec<i32>: ::core::borrow::BorrowMut<__BorrowMutT>,
{
    #[inline]
    fn borrow_mut(&mut self) -> &mut __BorrowMutT {
        <Vec<i32> as ::core::borrow::BorrowMut<__BorrowMutT>>::borrow_mut(&mut self.0)
    }
}
```


# Structs with Multiple Fields

When `BorrowMut` is derived for a struct with more than one field (including tuple
structs), you must also mark one or more fields with the `#[borrow_mut]` attribute.
An implementation will be generated for each indicated field.
You can also exclude a specific field by using `#[borrow_mut(ignore)]`.

```rust
# #[macro_use] extern crate derive_more;
# use core::borrow::Borrow;
# use core::borrow::BorrowMut;
# fn main(){}
#[derive(Borrow, BorrowMut)]
struct MyWrapper {
    #[borrow]
    #[borrow_mut]
    name: String,
    #[borrow]
    #[borrow_mut]
    num: i32,
    valid: bool,
}


```

Generates:

```rust
# #[macro_use] extern crate derive_more;
# use core::borrow::Borrow;
# use core::borrow::BorrowMut;
# #[derive(Borrow)]
# struct MyWrapper {
#     name: String,
#     num: i32,
#     valid: bool,
# }
impl BorrowMut<String> for MyWrapper {
    fn borrow_mut(&mut self) -> &mut String {
        &mut self.name
    }
}

impl BorrowMut<i32> for MyWrapper {
    fn borrow_mut(&mut self) -> &mut i32 {
        &mut self.num
    }
}
```

Note that `BorrowMut<T>` may only be implemented once for any given type `T`. This means any attempt to
mark more than one field of the same type with `#[borrow_mut]` will result in a compilation error.

```compile_fail
# #[macro_use] extern crate derive_more;
# use core::borrow::Borrow;
# use core::borrow::BorrowMut;
# fn main(){}
// Error! Conflicting implementations of BorrowMut<String>
#[derive(BorrowMut)]
struct MyWrapper {
    #[borrow_mut]
    str1: String,
    #[borrow_mut]
    str2: String,
}
```

# Enums

Deriving `BorrowMut` for enums is not supported.
