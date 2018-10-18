% What #[derive(Display)] generates

**NB: These derives are fully backward-compatible with the ones from the display_derive crate.**

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

use std::path::PathBuf;

#[derive(Display)]
struct MyInt(i32);

#[derive(Display)]
#[display(fmt = "({}, {})", x, y)]
struct Point2D {
    x: i32,
    y: i32,
}

#[derive(Display)]
enum E {
    Uint(u32),
    #[display(fmt = "I am B {:b}", i)]
    Binary {
        i: i8,
    },
    #[cfg(feature = "nightly")]
    #[display(fmt = "I am C {}", "_0.display()")]
    Path(PathBuf),
}

#[derive(Display)]
#[display(fmt = "Java EE")]
enum EE {
    A,
    B,
}

#[derive(Display)]
#[display(fmt = "Hello there!")]
union U {
    i: u32,
}

#[derive(Octal)]
#[octal(fmt = "7")]
struct S;

#[derive(UpperHex)]
#[upper_hex(fmt = "UpperHex")]
struct UH;

#[derive(Display)]
struct Unit;

#[derive(Display)]
struct UnitStruct {}

fn main() {
    assert_eq!(MyInt(-2).to_string(), "-2");
    assert_eq!(Point2D { x: 3, y: 4 }.to_string(), "(3, 4)");
    assert_eq!(E::Uint(2).to_string(), "2");
    assert_eq!(E::Binary { i: -2 }.to_string(), "I am B 11111110");
    #[cfg(feature = "nightly")]
    assert_eq!(E::Path("abc".into()).to_string(), "I am C abc");
    assert_eq!(EE::A.to_string(), "Java EE");
    assert_eq!(EE::B.to_string(), "Java EE");
    assert_eq!(U { i: 2 }.to_string(), "Hello there!");
    assert_eq!(format!("{:o}", S), "7");
    assert_eq!(format!("{:X}", UH), "UpperHex");
    assert_eq!(Unit.to_string(), "Unit");
    assert_eq!(UnitStruct {}.to_string(), "UnitStruct");
}
```
