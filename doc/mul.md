% What #[derive(Mul)] generates

# Tuple structs

When deriving for a tuple struct with a single field (i.e. a newtype) like this:

```
#[derive(From)]
struct MyInt(i32)
```

Code like this will be generated:

```

```

The behaviour is slightly different for multiple fields, since the type now
needs.
When deriving for a tuple struct with two fields like this:

```
#[derive(Mul)]
struct MyInts(i32, i32)
```

Code like this will be generated:

```
TODO
```

The behaviour is similar with more or less fields.



# Regular structs

TODO

# Enums

TODO
