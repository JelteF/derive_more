# `derive_more`

[![Build Status](https://api.travis-ci.org/JelteF/derive_more.svg?branch=master)](https://travis-ci.org/JelteF/derive_more)
[![Latest Version](https://img.shields.io/crates/v/derive_more.svg)](https://crates.io/crates/derive_more)
[![Rust Documentation](https://img.shields.io/badge/api-rustdoc-blue.svg)](https://jeltef.github.io/derive_more/derive_more/)
[![GitHub license](https://img.shields.io/badge/license-MIT-blue.svg)](https://raw.githubusercontent.com/JelteF/derive_more/master/LICENSE)

Rust has lots of builtin traits that are implemented for its basic types, such as [`Add`],
[`Not`] or [`From`].
However, when wrapping these types inside your own structs or enums you lose the
implementations of these traits and are required to recreate them.
This is especially annoying when your own structures are very simple, such as when using the
commonly advised newtype pattern (e.g. `MyInt(i32)`).

This library tries to remove these annoyances and the corresponding boilerplate code.
It does this by allowing you to derive lots of commonly used traits for both structs and enums.

## Example code

By using this library the following code just works:


```rust
#[derive(Debug, Eq, PartialEq, From, Add)]
struct MyInt(i32);

#[derive(Debug, Eq, PartialEq, From, Into, Constructor, Mul)]
struct Point2D {
    x: i32,
    y: i32,
}

#[derive(Debug, Eq, PartialEq, From, Add)]
enum MyEnum {
    Int(i32),
    UnsignedInt(u32),
    Nothing,
}

fn main() {
    let my_11 = MyInt(5) + 6.into();
    assert_eq!(MyInt(11), MyInt(5) + 6.into());
    assert_eq!(Point2D { x: 5, y: 6 } * 10, (50, 60).into());
    assert_eq!((5, 6), Point2D { x: 5, y: 6 }.into());
    assert_eq!(Point2D { x: 5, y: 6 }, Point2D::new(5, 6));
    assert_eq!(MyEnum::Int(15), (MyEnum::Int(8) + 7.into()).unwrap())
}
```

## The derivable traits

Below are all the traits that you can derive using this library.
Some trait derivations are so similar that the further documentation will only show a single one
of them.
You can recognize these by the "-like" suffix in their name.
The trait name before that will be the only one that is used throughout the further
documentation.

**NOTE**: You still have to derive each trait separately. So `#[derive(Mul)]` doesn't
automatically derive `Div` as well. To derive both you should do `#[derive(Mul, Div)]`

### Conversion traits
These are traits that are used to convert automatically between types.

1. [`From`]
2. [`Into`]
3. [`FromStr`]

### Formatting traits
These traits are used for converting a struct to a string in different ways.

1. `Display`-like, contains [`Display`], [`Binary`], [`Octal`], [`LowerHex`], [`UpperHex`],
   [`LowerExp`], [`UpperExp`], [`Pointer`]

### Operators
These are traits that can be used for operator overloading.

1. [`Index`]
2. `Not`-like, contains [`Not`] and [`Neg`]
3. `Add`-like, contains [`Add`], [`Sub`], [`BitAnd`], [`BitOr`] and [`BitXor`]
4. `Mul`-like, contains [`Mul`], [`Div`], [`Rem`], [`Shr`] and [`Shl`]
5. [`IndexMut`]
6. `AddAssign`-like, contains [`AddAssign`], [`SubAssign`], [`BitAndAssign`], [`BitOrAssign`]
   and [`BitXorAssign`]
7. `MulAssign`-like, contains [`MulAssign`], [`DivAssign`], [`RemAssign`], [`ShrAssign`] and
   [`ShlAssign`]

### Static methods
These don't derive traits, but derive static methods instead.

1. `Constructor`, this derives a `new` method that can be used as a constructor. This is very
   basic if you need more customization for your constructor, check out the [`derive-new`] crate.


## Generated code

It is important to understand what code gets generated when using one of the derives from this
crate.
That is why the links below explain what code gets generated for a trait for each group from
before.

1. [`#[derive(From)]`](https://jeltef.github.io/derive_more/derive_more/from.html)
2. [`#[derive(Into)]`](https://jeltef.github.io/derive_more/derive_more/into.html)
3. [`#[derive(FromStr)]`](https://jeltef.github.io/derive_more/derive_more/from_str.html)
4. [`#[derive(Display)]`](https://jeltef.github.io/derive_more/derive_more/display.html)
5. [`#[derive(Index)]`](https://jeltef.github.io/derive_more/derive_more/index_op.html)
6. [`#[derive(Not)]`](https://jeltef.github.io/derive_more/derive_more/not.html)
7. [`#[derive(Add)]`](https://jeltef.github.io/derive_more/derive_more/add.html)
8. [`#[derive(Mul)]`](https://jeltef.github.io/derive_more/derive_more/mul.html)
9. [`#[derive(IndexMut)]`](https://jeltef.github.io/derive_more/derive_more/index_mut.html)
10. [`#[derive(AddAssign)]`](https://jeltef.github.io/derive_more/derive_more/add_assign.html)
11. [`#[derive(MulAssign)]`](https://jeltef.github.io/derive_more/derive_more/mul_assign.html)
12. [`#[derive(Constructor)]`](https://jeltef.github.io/derive_more/derive_more/constructor.html)

If you want to be sure what code is generated for your specific type I recommend using the
[`cargo-expand`] utility.
This will show you your code with all macros and derives expanded.

## Installation

This library requires Rust 1.15 or higher, so this needs to be installed.
Then add the following to `Cargo.toml`:

```toml
[dependencies]
derive_more = "0.9.0"
```

And this to the top of your Rust file:

```rust
#[macro_use]
extern crate derive_more;
```

[`cargo-expand`]: https://github.com/dtolnay/cargo-expand
[`derive-new`]: https://github.com/nrc/derive-new
[`From`]: https://doc.rust-lang.org/core/convert/trait.From.html
[`Into`]: https://doc.rust-lang.org/core/convert/trait.Into.html
[`FromStr`]: https://doc.rust-lang.org/std/str/trait.FromStr.html
[`Display`]: https://doc.rust-lang.org/std/fmt/trait.Display.html
[`Binary`]: https://doc.rust-lang.org/std/fmt/trait.Binary.html
[`Octal`]: https://doc.rust-lang.org/std/fmt/trait.Octal.html
[`LowerHex`]: https://doc.rust-lang.org/std/fmt/trait.LowerHex.html
[`UpperHex`]: https://doc.rust-lang.org/std/fmt/trait.UpperHex.html
[`LowerExp`]: https://doc.rust-lang.org/std/fmt/trait.LowerExp.html
[`UpperExp`]: https://doc.rust-lang.org/std/fmt/trait.UpperExp.html
[`Pointer`]: https://doc.rust-lang.org/std/fmt/trait.Pointer.html
[`Index`]: https://doc.rust-lang.org/std/ops/trait.Index.html
[`Not`]: https://doc.rust-lang.org/std/ops/trait.Not.html
[`Neg`]: https://doc.rust-lang.org/std/ops/trait.Neg.html
[`Add`]: https://doc.rust-lang.org/std/ops/trait.Add.html
[`Sub`]: https://doc.rust-lang.org/std/ops/trait.Sub.html
[`BitAnd`]: https://doc.rust-lang.org/std/ops/trait.BitAnd.html
[`BitOr`]: https://doc.rust-lang.org/std/ops/trait.BitOr.html
[`BitXor`]: https://doc.rust-lang.org/std/ops/trait.BitXor.html
[`Mul`]: https://doc.rust-lang.org/std/ops/trait.Mul.html
[`Div`]: https://doc.rust-lang.org/std/ops/trait.Div.html
[`Rem`]: https://doc.rust-lang.org/std/ops/trait.Rem.html
[`Shr`]: https://doc.rust-lang.org/std/ops/trait.Shr.html
[`Shl`]: https://doc.rust-lang.org/std/ops/trait.Shl.html
[`IndexMut`]: https://doc.rust-lang.org/std/ops/trait.IndexMut.html
[`AddAssign`]: https://doc.rust-lang.org/std/ops/trait.AddAssign.html
[`SubAssign`]: https://doc.rust-lang.org/std/ops/trait.SubAssign.html
[`BitAndAssign`]: https://doc.rust-lang.org/std/ops/trait.BitAndAssign.html
[`BitOrAssign`]: https://doc.rust-lang.org/std/ops/trait.BitOrAssign.html
[`BitXorAssign`]: https://doc.rust-lang.org/std/ops/trait.BitXorAssign.html
[`MulAssign`]: https://doc.rust-lang.org/std/ops/trait.MulAssign.html
[`DivAssign`]: https://doc.rust-lang.org/std/ops/trait.DivAssign.html
[`RemAssign`]: https://doc.rust-lang.org/std/ops/trait.RemAssign.html
[`ShrAssign`]: https://doc.rust-lang.org/std/ops/trait.ShrAssign.html
[`ShlAssign`]: https://doc.rust-lang.org/std/ops/trait.ShlAssign.html
