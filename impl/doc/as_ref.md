# What `#[derive(AsRef)]` generates

Deriving `AsRef` generates one or more implementations of `AsRef`, each
corresponding to one of the fields of the decorated type.
This allows types which contain some `T` to be passed anywhere that an
`AsRef<T>` is accepted.




## Newtypes and Structs with One Field

When `AsRef` is derived for a newtype or struct with one field, a single
implementation is generated to expose the underlying field.

```rust
# use derive_more::AsRef;
#
#[derive(AsRef)]
struct MyWrapper(String);
```

Generates:

```rust
# struct MyWrapper(String);
impl AsRef<String> for MyWrapper {
    fn as_ref(&self) -> &String {
        &self.0
    }
}
```

The `#[as_ref(forward)]` attribute can be used to forward
to the `as_ref` implementation of the field. So here `SingleFieldForward`
implements all `AsRef` for all types that `Vec<i32>` implements `AsRef` for.

```rust
# use derive_more::AsRef;
#
#[derive(AsRef)]
#[as_ref(forward)]
struct SingleFieldForward(Vec<i32>);

let item = SingleFieldForward(vec![]);
let _: &[i32] = (&item).as_ref();
```

This generates:

```rust
# struct SingleFieldForward(Vec<i32>);
impl<__AsT: ?::core::marker::Sized> ::core::convert::AsRef<__AsT> for SingleFieldForward
where
    Vec<i32>: ::core::convert::AsRef<__AsT>,
{
    #[inline]
    fn as_ref(&self) -> &__AsT {
        <Vec<i32> as ::core::convert::AsRef<__AsT>>::as_ref(&self.0)
    }
}
```

It's also possible to specify concrete types to derive forwarded
impls for with `#[as_ref(<types>)]`.

```rust
# use derive_more::AsRef;
#
#[derive(AsRef)]
#[as_ref(str, [u8])]
struct Types(String);

let item = Types("test".to_owned());
let _: &str = item.as_ref();
let _: &[u8] = item.as_ref();
```

Generates:

```rust
# struct Types(String);
impl AsRef<str> for Types {
    fn as_ref(&self) -> &str {
        <String as ::core::convert::AsRef<str>>::as_ref(&self.0)
    }
}

impl AsRef<[u8]> for Types {
    fn as_ref(&self) -> &[u8] {
        <String as ::core::convert::AsRef<[u8]>>::as_ref(&self.0)
    }
}
```



## Structs with Multiple Fields

When `AsRef` is derived for a struct with more than one field (including tuple
structs), you must also mark one or more fields with the `#[as_ref]` attribute.
An implementation will be generated for each indicated field.
You can also exclude a specific field by using `#[as_ref(skip)]` (or `#[as_ref(ignore)]`).

```rust
# use derive_more::AsRef;
#
#[derive(AsRef)]
struct MyWrapper {
    #[as_ref]
    name: String,
    #[as_ref]
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
impl AsRef<String> for MyWrapper {
    fn as_ref(&self) -> &String {
        &self.name
    }
}

impl AsRef<i32> for MyWrapper {
    fn as_ref(&self) -> &i32 {
        &self.num
    }
}
```

Note that `AsRef<T>` may only be implemented once for any given type `T`.
This means any attempt to mark more than one field of the same type with
`#[as_ref]` will result in a compilation error.

```rust,compile_fail
# use derive_more::AsRef;
#
// Error! Conflicting implementations of AsRef<String>
#[derive(AsRef)]
struct MyWrapper {
    #[as_ref]
    str1: String,
    #[as_ref]
    str2: String,
}
```

Similarly, if some field is annotated with `#[as_ref(forward)]`, no other
field can be marked.

```rust,compile_fail
# use derive_more::AsRef;
#
// Error! Conflicting implementations of `AsRef<i32>`
// note: upstream crates may add a new impl of trait `AsRef<i32>`
// for type `String` in future versions
#[derive(AsRef)]
struct ForwardWithOther {
    #[as_ref(forward)]
    str: String,
    #[as_ref]
    number: i32,
}
```

Multiple forwarded impls with concrete types, however, can be used.

```rust
# use derive_more::AsRef;
#
#[derive(AsRef)]
struct Types {
    #[as_ref(str)]
    str: String,
    #[as_ref([u8])]
    vec: Vec<u8>,
}

let item = Types {
    str: "test".to_owned(),
    vec: vec![0u8],
};

let _: &str = item.as_ref();
let _: &[u8] = item.as_ref();
```


## Enums

Deriving `AsRef` for enums is not supported.
