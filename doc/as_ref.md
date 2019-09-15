% What #[derive(AsRef)] generates

Deriving `AsRef` generates one or more implementations of `AsRef`, each corresponding to one of the
fields of the decorated type. This allows types which contain some `T` to be passed anywhere that an
`AsRef<T>` is accepted.

# Newtypes and Structs with One Field

When `AsRef` is derived for a newtype or struct with one field, a single implementation is generated
to expose the underlying field.

```rust
#[derive(AsRef)]
struct MyWrapper(String);



// Generates:

impl AsRef<String> for MyWrapper {
    fn as_ref(&self) -> &String {
        &self.0
    }
}
```

# Structs with Multiple Fields

When `AsRef` is derived for a struct with more than one field (including tuple structs), you must
also mark one or more fields with the `#[as_ref]` attribute. An implementation will be generated for
each indicated field.

```rust
#[derive(AsRef)]
struct MyWrapper {
    #[as_ref]
    name: String,
    #[as_ref]
    path: Path,
    valid: bool,
}



// Generates:

impl AsRef<String> for MyWrapper {
    fn as_ref(&self) -> &String {
        &self.name
    }
}

impl AsRef<Path> for MyWrapper {
    fn as_ref(&self) -> &Path {
        &self.path
    }
}
```

Note that `AsRef<T>` may only be implemented once for any given type `T`. This means any attempt to
mark more than one field of the same type with `#[as_ref]` will result in a compilation error.

```rust
// Error! Conflicting implementations of AsRef<String>
#[derive(AsRef)]
struct MyWrapper {
    #[as_ref]
    str1: String,
    #[as_ref]
    str2: String,
}
```

# Enums

Deriving `AsRef` for enums is not supported.
