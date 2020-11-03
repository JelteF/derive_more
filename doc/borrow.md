% What #[derive(Borrow)] generates

Deriving `Borrow` generates one or more implementations of `Borrow`, each
corresponding to one of the fields of the decorated type.
This allows types which contain some `T` to be passed anywhere that an
`Borrow<T>` is accepted.

# Newtypes and Structs with One Field

When `Borrow` is derived for a newtype or struct with one field, a single
implementation is generated to expose the underlying field.

```rust
# use core::borrow::Borrow;
# #[macro_use] extern crate derive_more;
# fn main(){}
#[derive(Borrow)]
struct MyWrapper(String);
```

Generates:

```rust
# use core::borrow::Borrow;
# struct MyWrapper(String);
impl Borrow<String> for MyWrapper {
    fn borrow(&self) -> &String {
        &self.0
    }
}
```

It's also possible to use the `#[borrow(forward)]` attribute to forward
to the `borrow` implementation of the field. So here `SigleFieldForward`
implements all `Borrow` for all types that `Vec<i32>` implements `Borrow` for.

```rust
# use core::borrow::Borrow;
# #[macro_use] extern crate derive_more;
#[derive(Borrow)]
#[borrow(forward)]
struct SingleFieldForward(Vec<i32>);

fn main() {
    let item = SingleFieldForward(vec![]);
    let _: &[i32] = (&item).borrow();
}

```

This generates:

```rust
# struct SingleFieldForward(Vec<i32>);
impl<__BorrowT: ?::core::marker::Sized> ::core::borrow::Borrow<__BorrowT> for SingleFieldForward
where
    Vec<i32>: ::core::borrow::Borrow<__BorrowT>,
{
    #[inline]
    fn borrow(&self) -> &__BorrowT {
        <Vec<i32> as ::core::borrow::Borrow<__BorrowT>>::borrow(&self.0)
    }
}
```

# Structs with Multiple Fields

When `Borrow` is derived for a struct with more than one field (including tuple
structs), you must also mark one or more fields with the `#[borrow]` attribute.
An implementation will be generated for each indicated field.
You can also exclude a specific field by using `#[borrow(ignore)]`.

```rust
# use core::borrow::Borrow;
# #[macro_use] extern crate derive_more;
# fn main(){}
#[derive(Borrow)]
struct MyWrapper {
    #[borrow]
    name: String,
    #[borrow]
    num: i32,
    valid: bool,
}

```

Generates:

```rust
# use core::borrow::Borrow;
# struct MyWrapper {
#     name: String,
#     num: i32,
#     valid: bool,
# }
impl Borrow<String> for MyWrapper {
    fn borrow(&self) -> &String {
        &self.name
    }
}

impl Borrow<i32> for MyWrapper {
    fn borrow(&self) -> &i32 {
        &self.num
    }
}
```

Note that `Borrow<T>` may only be implemented once for any given type `T`.
This means any attempt to mark more than one field of the same type with
`#[borrow]` will result in a compilation error.

```compile_fail
# use core::borrow::Borrow;
# #[macro_use] extern crate derive_more;
# fn main(){}
// Error! Conflicting implementations of Borrow<String>
#[derive(Borrow)]
struct MyWrapper {
    #[borrow]
    str1: String,
    #[borrow]
    str2: String,
}
```

# Enums

Deriving `Borrow` for enums is not supported.
