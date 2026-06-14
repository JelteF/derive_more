# What `#[derive(Constructor)]` generates

A common pattern in Rust is to create a static constructor method called
`new`. This method can then be used to create an instance of a struct. You
can now derive this method by using `#[derive(Constructor)]`, even though
`Constructor` is not an actual trait. The generated `new` method is very
similar to the `from` method when deriving `From`, except that it takes multiple
arguments instead of a tuple.




## Tuple structs

When deriving `Constructor` for a tuple struct with two fields like this:

```rust
# use derive_more::Constructor;
#
#[derive(Constructor)]
struct MyInts(i32, i32);
```

Code like this will be generated:

```rust
# struct MyInts(i32, i32);
impl MyInts {
    pub const fn new(__0: i32, __1: i32) -> MyInts {
        MyInts(__0, __1)
    }
}
```

The generated code is similar for more or less fields.




## Regular structs

For regular structs almost the same code is generated as for tuple structs
except that it assigns the fields differently.

```rust
# use derive_more::Constructor;
#
#[derive(Constructor)]
struct Point2D {
    x: i32,
    y: i32,
}
```

Code like this will be generated:

```rust
# struct Point2D {
#     x: i32,
#     y: i32,
# }
impl Point2D {
    pub const fn new(x: i32, y: i32) -> Point2D {
        Point2D { x: x, y: y }
    }
}
```

The generated code is similar for more or less fields.




## `Into` arguments

By default the generated `new` method takes each field by its exact type. Adding
`#[constructor(into)]` to a field makes `new` accept any type that implements
[`Into`] that field's type, and calls [`Into::into`] to convert it.

```rust
# use derive_more::Constructor;
#
#[derive(Constructor)]
struct Foo {
    id: i32,
    #[constructor(into)]
    name: String,
}
```

Code like this will be generated:

```rust
# struct Foo {
#     id: i32,
#     name: String,
# }
impl Foo {
    pub fn new(id: i32, name: impl ::core::convert::Into<String>) -> Foo {
        Foo { id: id, name: ::core::convert::Into::into(name) }
    }
}
```

So a `&str` can be passed where a `String` is expected:

```rust
# use derive_more::Constructor;
#
# #[derive(Constructor)]
# struct Foo {
#     id: i32,
#     #[constructor(into)]
#     name: String,
# }
let foo = Foo::new(1, "hello");
```

Note that, unlike the default, a `new` method with any `#[constructor(into)]` field
cannot be `const`, because trait methods like [`Into::into`] are not yet callable in
`const` contexts.

[`Into`]: https://doc.rust-lang.org/core/convert/trait.Into.html
[`Into::into`]: https://doc.rust-lang.org/core/convert/trait.Into.html#tymethod.into




## Enums

Currently `Constructor` cannot be derived for enums. This is because the `new`
method might then need to have a different number of arguments. This is
currently not supported by Rust. So this functionality will not be added until
this [RFC](https://github.com/rust-lang/rfcs/issues/376) (or a similar one) is
accepted and implemented.
