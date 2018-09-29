% What #[derive(Display)] generates

Deriving `Display` will generate a `Display` implementation, with a `fmt`
method that matches `self` and each of its variants. In the case of a struct or union,
only a single variant is available, and it is thus equivalent to a simple `let` statement.
In the case of an enum, each of its variants is matched.

For each matched variant, a `write!` expression will be generated with
the supplied format, or an automatically inferred one.

You specify the format on each variant by writing e.g. `#[display(fmt = "my val: {}", "some_val * 2")]`.
For enums, you can either specify it on each variant, or on the enum as a whole.

For variants that don't have a format specified, it will simply defer to the format of the
inner variable. If there is no such variable, or there is more than 1, an error is generated.

# The format of the format

You supply a format by attaching an attribute of the syntax: `#[display(fmt = "...", args...)]`.
The format supplied is passed verbatim to `write!`. The arguments supplied handled specially,
due to constraints in the syntax of attributes. In the case of an argument being a simple
identifier, it is passed verbatim. If an argument is a string, it is **parsed as an expression**,
and then passed to `write!`.

The variables available in the arguments is `self` and each member of the variant,
with members of tuple structs being named with a leading underscore and their index,
i.e. `_0`, `_1`, `_2`, etc.

## Other formatting traits

The syntax does not change, but the name of the attribute is the snake case version of the trait.
E.g. `Octal` -> `octal`, `Pointer` -> `pointer`, `UpperHex` -> `upper_hex`.

# Example usage

```rust
# #[macro_use] extern crate derive_more;
#[derive(Display)]
struct MyInt(i32);

#[derive(Display)]
#[display(fmt = "({}, {})", x, y)]
struct Point1D {
    x: i32,
    y: i32,
}

#[derive(Display)]
enum E {
    A(u32),
    #[display(fmt = "I am B {:b}", i)]
    B {
        i: i8,
    },
    #[cfg(feature = "nightly")] // this will be in stable releases soon
    #[display(fmt = "I am C {}", "_0.display()")]
    C(PathBuf),
}

#[derive(Display)]
#[display(fmt = "Hello there!")]
union U {
    i: u32,
}

#[derive(Octal)]
#[octal(fmt = "8")]
struct S;
```
