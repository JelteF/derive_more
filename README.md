# derive_more

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

#[derive(Debug, Eq, PartialEq, From, Mul)]
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
    assert_eq!(MyEnum::Int(15), (MyEnum::Int(8) + 7.into()).unwrap())
}


```

## The newly derivable traits

Obviously not all traits should be derived to the same code, because they are different
different traits after all.
However, some of the semantics of the traits overlap a lot, so they have been grouped in the
following way:

1. `From`, only contains the [`From`].
2. `Not`-like, contains [`Not`] and [`Neg`].
3. `Add`-like, contains [`Add`], [`Sub`], [`BitAnd`], [`BitOr`] and [`BitXor`].
4. `AddAssign`-like, contains [`AddAssign`], [`SubAssign`], [`BitAndAssign`], [`BitOrAssign`]
   and [`BitXorAssign`].
5. `Mul`-like, contains [`Mul`], [`Div`], [`Rem`], [`Shr`] and [`Shl`].


## Generated code

It is important to understand what code gets generated when using one of the derives from this
crate.
That is why the links below explain what code gets generated for a trait for each group from
before.

1. [`#[derive(From)]`](https://jeltef.github.io/derive_more/derive_more/from.html)
2. [`#[derive(Not)]`](https://jeltef.github.io/derive_more/derive_more/not.html)
3. [`#[derive(Add)]`](https://jeltef.github.io/derive_more/derive_more/add.html)
4. [`#[derive(AddAssign)]`](https://jeltef.github.io/derive_more/derive_more/add_assign.html)
5. [`#[derive(Mul)]`](https://jeltef.github.io/derive_more/derive_more/mul.html)

If you want to be sure what code is generated for your specific trait I recommend using the
[`cargo-expand`] utility.
This will show you your code with all macros and derives expanded.

## Installation

This library heavily uses Macros 1.1, which is to stabilized in Rust 1.15 (the next Rust
release).
To use it before that time you have to install the nightly or beta channel.

After doing this, add this to `Cargo.toml`:

```toml
[dependencies]
derive_more = "0.4.0"
```

And this to the top of your Rust file:

```rust
#[macro_use]
extern crate derive_more;
```

[`cargo-expand`]: https://github.com/dtolnay/cargo-expand
[`From`]: https://doc.rust-lang.org/std/convert/trait.From.html
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
[`AddAssign`]: https://doc.rust-lang.org/std/ops/trait.AddAssign.html
[`SubAssign`]: https://doc.rust-lang.org/std/ops/trait.SubAssign.html
[`BitAndAssign`]: https://doc.rust-lang.org/std/ops/trait.BitAndAssign.html
[`BitOrAssign`]: https://doc.rust-lang.org/std/ops/trait.BitOrAssign.html
[`BitXorAssign`]: https://doc.rust-lang.org/std/ops/trait.BitXorAssign.html

