//! # derive_more
//! Rust has lots of builtin traits that are implemented for its basic types, such as [`Add`],
//! [`Not`] or [`From`].
//! However, when wrapping these types inside your own structs or enums you lose the
//! implementations of these traits and are required to recreate them.
//! This is especially annoying when your own structures are very simple, such as when using the
//! commonly advised newtype pattern (e.g. `MyInt(i32)`).
//!
//! This library tries to remove these annoyances and the corresponding boilerplate code.
//! It does this by allowing you to derive lots of commonly used traits for both structs and enums.
//!
//! ## Example code
//!
//! By using this library the following code just works:
//!
//!
//! ```rust
//! # #[macro_use] extern crate derive_more;
//!
//! #[derive(Debug, Eq, PartialEq, From, Add)]
//! struct MyInt(i32);
//!
//! #[derive(Debug, Eq, PartialEq, From, Into, Constructor, Mul)]
//! struct Point2D {
//!     x: i32,
//!     y: i32,
//! }
//!
//! #[derive(Debug, Eq, PartialEq, From, Add)]
//! enum MyEnum {
//!     Int(i32),
//!     UnsignedInt(u32),
//!     Nothing,
//! }
//!
//! fn main() {
//!     let my_11 = MyInt(5) + 6.into();
//!     assert_eq!(MyInt(11), MyInt(5) + 6.into());
//!     assert_eq!(Point2D { x: 5, y: 6 } * 10, (50, 60).into());
//!     assert_eq!((5, 6), Point2D { x: 5, y: 6 }.into());
//!     assert_eq!(Point2D { x: 5, y: 6 }, Point2D::new(5, 6));
//!     assert_eq!(MyEnum::Int(15), (MyEnum::Int(8) + 7.into()).unwrap())
//! }
//! ```
//!
//! ## The newly derivable traits
//!
//! Obviously not all traits should be derived to the same code, because they are different
//! traits after all.
//! However, some of the semantics of the traits overlap a lot, so they have been grouped in the
//! following way:
//!
//! 1. `From`, only contains [`From`].
//! 2. `Into`, only contains [`Into`].
//! 3. `Constructor`, this doesn't derive a trait, but it derives a `new` method that can be
//!    used as a constructor.
//! 4. Newtype derives (and structs with one field), contains [`FromStr`].
//! 5. `Not`-like, contains [`Not`] and [`Neg`].
//! 6. `Add`-like, contains [`Add`], [`Sub`], [`BitAnd`], [`BitOr`] and [`BitXor`].
//! 7. `AddAssign`-like, contains [`AddAssign`], [`SubAssign`], [`BitAndAssign`], [`BitOrAssign`]
//!    and [`BitXorAssign`].
//! 8. `Mul`-like, contains [`Mul`], [`Div`], [`Rem`], [`Shr`] and [`Shl`].
//! 9. `MulAssign`-like, contains [`MulAssign`], [`DivAssign`], [`RemAssign`], [`ShrAssign`] and [`ShlAssign`].
//!
//!
//! ## Generated code
//!
//! It is important to understand what code gets generated when using one of the derives from this
//! crate.
//! That is why the links below explain what code gets generated for a trait for each group from
//! before.
//!
//! 1. [`#[derive(From)]`](from.html)
//! 2. [`#[derive(Into)]`](into.html)
//! 3. [`#[derive(Constructor)]`](constructor.html)
//! 4. [`#[derive(FromStr)]`](from_str.html)
//! 5. [`#[derive(Not)]`](not.html)
//! 6. [`#[derive(Add)]`](add.html)
//! 7. [`#[derive(AddAssign)]`](add_assign.html)
//! 8. [`#[derive(Mul)]`](mul.html)
//! 9. [`#[derive(MulAssign)]`](mul_assign.html)
//!
//! If you want to be sure what code is generated for your specific type I recommend using the
//! [`cargo-expand`] utility.
//! This will show you your code with all macros and derives expanded.
//!
//! ## Installation
//!
//! This library requires Rust 1.15 or higher, so this needs to be installed.
//! Then add the following to `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! derive_more = "0.6.0"
//! ```
//!
//! And this to the top of your Rust file:
//!
//! ```rust
//! #[macro_use]
//! extern crate derive_more;
//! # fn main () {}
//! ```
//!
//! [`cargo-expand`]: https://github.com/dtolnay/cargo-expand
//! [`From`]: https://doc.rust-lang.org/core/convert/trait.From.html
//! [`Into`]: https://doc.rust-lang.org/core/convert/trait.Into.html
//! [`FromStr`]: https://doc.rust-lang.org/std/str/trait.FromStr.html
//! [`Not`]: https://doc.rust-lang.org/std/ops/trait.Not.html
//! [`Neg`]: https://doc.rust-lang.org/std/ops/trait.Neg.html
//! [`Add`]: https://doc.rust-lang.org/std/ops/trait.Add.html
//! [`Sub`]: https://doc.rust-lang.org/std/ops/trait.Sub.html
//! [`BitAnd`]: https://doc.rust-lang.org/std/ops/trait.BitAnd.html
//! [`BitOr`]: https://doc.rust-lang.org/std/ops/trait.BitOr.html
//! [`BitXor`]: https://doc.rust-lang.org/std/ops/trait.BitXor.html
//! [`AddAssign`]: https://doc.rust-lang.org/std/ops/trait.AddAssign.html
//! [`SubAssign`]: https://doc.rust-lang.org/std/ops/trait.SubAssign.html
//! [`BitAndAssign`]: https://doc.rust-lang.org/std/ops/trait.BitAndAssign.html
//! [`BitOrAssign`]: https://doc.rust-lang.org/std/ops/trait.BitOrAssign.html
//! [`BitXorAssign`]: https://doc.rust-lang.org/std/ops/trait.BitXorAssign.html
//! [`Mul`]: https://doc.rust-lang.org/std/ops/trait.Mul.html
//! [`Div`]: https://doc.rust-lang.org/std/ops/trait.Div.html
//! [`Rem`]: https://doc.rust-lang.org/std/ops/trait.Rem.html
//! [`Shr`]: https://doc.rust-lang.org/std/ops/trait.Shr.html
//! [`Shl`]: https://doc.rust-lang.org/std/ops/trait.Shl.html
//! [`MulAssign`]: https://doc.rust-lang.org/std/ops/trait.MulAssign.html
//! [`DivAssign`]: https://doc.rust-lang.org/std/ops/trait.DivAssign.html
//! [`RemAssign`]: https://doc.rust-lang.org/std/ops/trait.RemAssign.html
//! [`ShrAssign`]: https://doc.rust-lang.org/std/ops/trait.ShrAssign.html
//! [`ShlAssign`]: https://doc.rust-lang.org/std/ops/trait.ShlAssign.html

extern crate proc_macro;
#[macro_use]
extern crate quote;
extern crate syn;

use proc_macro::TokenStream;

mod utils;

mod from;
mod into;
mod constructor;
mod not_like;
mod add_like;
mod add_assign_like;
mod mul_like;
mod mul_assign_like;
mod from_str;
mod display;

macro_rules! create_derive(
    ($mod_:ident, $trait_:ident, $fn_name: ident) => {
        #[proc_macro_derive($trait_)]
        #[doc(hidden)]
        pub fn $fn_name(input: TokenStream) -> TokenStream {
            let ast = syn::parse(input).unwrap();
            $mod_::expand(&ast, stringify!($trait_)).into()
        }
    }
);

create_derive!(from, From, from_derive);

create_derive!(into, Into, into_derive);

create_derive!(constructor, Constructor, constructor_derive);

create_derive!(not_like, Not, not_derive);
create_derive!(not_like, Neg, neg_derive);

create_derive!(add_like, Add, add_derive);
create_derive!(add_like, Sub, sub_derive);
create_derive!(add_like, BitAnd, bit_and_derive);
create_derive!(add_like, BitOr, bit_or_derive);
create_derive!(add_like, BitXor, bit_xor_derive);

create_derive!(mul_like, Mul, mul_derive);
create_derive!(mul_like, Div, div_derive);
create_derive!(mul_like, Rem, rem_derive);
create_derive!(mul_like, Shr, shr_derive);
create_derive!(mul_like, Shl, shl_derive);

create_derive!(add_assign_like, AddAssign, add_assign_derive);
create_derive!(add_assign_like, SubAssign, sub_assign_derive);
create_derive!(add_assign_like, BitAndAssign, bit_and_assign_derive);
create_derive!(add_assign_like, BitOrAssign, bit_or_assign_derive);
create_derive!(add_assign_like, BitXorAssign, bit_xor_assign_derive);

create_derive!(mul_assign_like, MulAssign, mul_assign_derive);
create_derive!(mul_assign_like, DivAssign, div_assign_derive);
create_derive!(mul_assign_like, RemAssign, rem_assign_derive);
create_derive!(mul_assign_like, ShrAssign, shr_assign_derive);
create_derive!(mul_assign_like, ShlAssign, shl_assign_derive);

create_derive!(from_str, FromStr, from_str_derive);
create_derive!(display, Display, display_derive);
