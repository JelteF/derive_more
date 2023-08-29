# What `#[derive(AsMut)]` generates

Deriving `AsMut` generates one or more implementations of `AsMut`, each
corresponding to one of the fields of the decorated type.
This allows types which contain some `T` to be passed anywhere that an
`AsMut<T>` is accepted.




## Newtypes and Structs with One Field

When `AsMut` is derived for a newtype or struct with one field, a single
implementation is generated to expose the underlying field.

```rust
# use derive_more::AsMut;
#
#[derive(AsMut)]
struct MyWrapper(String);
```

Generates:

```rust
# struct MyWrapper(String);
impl AsMut<String> for MyWrapper {
    fn as_mut(&mut self) -> &mut String {
        &mut self.0
    }
}
```

The `#[as_mut(forward)]` attribute can be used to forward
to the `as_mut` implementation of the field. So here `SingleFieldForward`
implements all `AsMut` for all types that `Vec<i32>` implements `AsMut` for.

```rust
# use derive_more::AsMut;
#
#[derive(AsMut)]
#[as_mut(forward)]
struct SingleFieldForward(Vec<i32>);

let mut item = SingleFieldForward(vec![]);
let _: &mut [i32] = (&mut item).as_mut();
```

This generates:

```rust
# struct SingleFieldForward(Vec<i32>);
impl<__AsT: ?::core::marker::Sized> ::core::convert::AsMut<__AsT> for SingleFieldForward
where
    Vec<i32>: ::core::convert::AsMut<__AsT>,
{
    #[inline]
    fn as_mut(&mut self) -> &mut __AsT {
        <Vec<i32> as ::core::convert::AsMut<__AsT>>::as_mut(&mut self.0)
    }
}
```

It's also possible to specify concrete types to derive forwarded
impls for with `#[as_mut(<types>)]`.

```rust
# use derive_more::AsMut;
#
#[derive(AsMut)]
#[as_mut(str, [u8])]
struct Types(String);

let mut item = Types("test".to_owned());
let _: &mut str = item.as_mut();
let _: &mut [u8] = item.as_mut();
```

Generates:

```rust
# struct Types(String);
impl AsMut<str> for Types {
    fn as_mut(&mut self) -> &mut str {
        <String as ::core::convert::AsMut<str>>::as_mut(&mut self.0)
    }
}

impl AsMut<[u8]> for Types {
    fn as_mut(&mut self) -> &mut [u8] {
        <String as ::core::convert::AsMut<[u8]>>::as_mut(&mut self.0)
    }
}
```


## Structs with Multiple Fields

When `AsMut` is derived for a struct with more than one field (including tuple
structs), you must also mark one or more fields with the `#[as_mut]` attribute.
An implementation will be generated for each indicated field.

```rust
# use derive_more::AsMut;
#
#[derive(AsMut)]
struct MyWrapper {
    #[as_mut(str)]
    name: String,
    #[as_mut]
    num: i32,
    valid: bool,
}
```

Generates:

```rust
# struct MyWrapper {
#     name: String,
#     num: i32,
#     valid: bool,
# }
impl AsMut<str> for MyWrapper {
    fn as_mut(&mut self) -> &mut String {
        <String as ::core::convert::AsMut<str>>::as_mut(&mut self.name)
    }
}

impl AsMut<i32> for MyWrapper {
    fn as_mut(&mut self) -> &mut i32 {
        &mut self.num
    }
}
```

Only conversions that use a single field are possible with this derive.
Something like this wouldn't work, due to the nature of the `AsMut` trait:

```rust,compile_fail
# use derive_more::AsRef
#
// Error! `#[as_ref(...)]` attribute can only be placed on structs with exactly one field
#[derive(AsRef)]
#[as_ref((str, [u8]))]
struct MyWrapper(String, Vec<u8>)
```

If you need to convert into multiple references, consider using the
[`Into`](crate::Into) derive with `#[into(ref_mut)]`.

### Skipping

Or vice versa: you can exclude a specific field by using `#[as_mut(skip)]` (or
`#[as_mut(ignore)]`). Then, implementations will be generated for non-indicated fields.

```rust
# use derive_more::AsMut;
#
#[derive(AsMut)]
struct MyWrapper {
    #[as_mut(skip)]
    name: String,
    #[as_mut(ignore)]
    num: i32,
    valid: bool,
}
```

Generates:

```rust
# struct MyWrapper {
#     name: String,
#     num: i32,
#     valid: bool,
# }
impl AsMut<bool> for MyWrapper {
    fn as_mut(&mut self) -> &mut bool {
        &mut self.valid
    }
}
```


### Coherence

Note that `AsMut<T>` may only be implemented once for any given type `T`.
This means any attempt to mark more than one field of the same type with
`#[as_mut]` will result in a compilation error.

```rust,compile_fail
# use derive_more::AsMut;
#
// Error! Conflicting implementations of AsMut<String>
#[derive(AsMut)]
struct MyWrapper {
    #[as_mut]
    str1: String,
    #[as_mut]
    str2: String,
}
```

Similarly, if some field is annotated with `#[as_mut(forward)]`, no other
field can be marked.

```rust,compile_fail
# use derive_more::AsMut;
#
// Error! Conflicting implementations of `AsMut<i32>`
// note: upstream crates may add a new impl of trait `AsMut<i32>`
// for type `String` in future versions
#[derive(AsMut)]
struct ForwardWithOther {
    #[as_mut(forward)]
    str: String,
    #[as_mut]
    number: i32,
}
```

Multiple forwarded impls with concrete types, however, can be used.

```rust
# use derive_more::AsMut;
#
#[derive(AsMut)]
struct Types {
    #[as_mut(str)]
    str: String,
    #[as_mut([u8])]
    vec: Vec<u8>,
}

let mut item = Types {
    str: "test".to_owned(),
    vec: vec![0u8],
};

let _: &mut str = item.as_mut();
let _: &mut [u8] = item.as_mut();
```



## Enums

Deriving `AsMut` for enums is not supported.
