//! # derive_more
//! Rust has lots of builtin traits that are implemented for its basic types, such as [`Add`], [`Not`]
//! or [`From`]. 
//! However, when wrapping these types inside your own structs or enums you lose the
//! implementations of these traits and are required to recreate them.
//! This is especially annoying when your own structures are very simple, such as when using the
//! commonly advised newtype pattern (e.g. `MyInt(i32)`).
//!
//! This library tries to remove these annoyances and the corresponding boilerplate code. It does
//! this by allowing you to derive lots of commonly used traits for both structs and enums.
//!
//! ## The newly derivable traits
//!
//! Obviously not all traits should be derived to the same code, because they are different
//! different traits after all. However, some of the semantics of the traits overlap a lot, so they
//! have been grouped in the following way:
//!
//! 1. `From`, only contains the [`From`].
//! 2. `Not`-like, contains [`Not`] and [`Neg`].
//! 3. `Add`-like, contains [`Add`], [`Sub`], [`BitAnd`], [`BitOr`] and [`BitXor`].
//! 4. `Mul`-like, contains [`Mul`], [`Div`], [`Rem`], [`Shr`] and [`Shl`].
//! 5. `AddAssign`-like, contains [`AddAssign`], [`SubAssign`], [`BitAndAssign`], [`BitOrAssign`]
//!    and [`BitXorAssign`].
//!
//!
//! ## Generated code
//!
//! It is important to understand what code gets generated when using one of the derives from this
//! crate. That is why the links below explain what code gets generated for a trait for each group
//! from before.
//!
//! 1. [`#[derive(From)]`](from.html)
//! 2. [`#[derive(Not)]`](not.html)
//! 3. [`#[derive(Add)]`](add.html)
//! 4. [`#[derive(Mul)]`](mul.html)
//! 5. [`#[derive(AddAssign)]`](add_assign.html)
//!
//! If you want to be sure what code is generated for your specific trait I recommend
//! using the [`cargo-expand`] utility. This will show you your code with all macros and derives
//! expanded.
//!
//! ## Installation
//!
//! This library heavily uses Macros 1.1, which is to stabilized in Rust 1.15 (the next Rust
//! release). To use it before that time you have to install the nightly or beta channel.
//!
//! After doing this, add this to `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! derive_more = "0.4.0"
//! ```
//!
//! And this to the top of your Rust file:
//!
//! ```
//! #[macro_use]
//! extern crate derive_more;
//! ```
//!
//! [`cargo-expand`]: https://github.com/dtolnay/cargo-expand
//! [`From`]: https://doc.rust-lang.org/std/convert/trait.From.html
//! [`Not`]: https://doc.rust-lang.org/std/ops/trait.Not.html
//! [`Neg`]: https://doc.rust-lang.org/std/ops/trait.Neg.html
//! [`Add`]: https://doc.rust-lang.org/std/ops/trait.Add.html
//! [`Sub`]: https://doc.rust-lang.org/std/ops/trait.Sub.html
//! [`BitAnd`]: https://doc.rust-lang.org/std/ops/trait.BitAnd.html
//! [`BitOr`]: https://doc.rust-lang.org/std/ops/trait.BitOr.html
//! [`BitXor`]: https://doc.rust-lang.org/std/ops/trait.BitXor.html
//! [`Mul`]: https://doc.rust-lang.org/std/ops/trait.Mul.html
//! [`Div`]: https://doc.rust-lang.org/std/ops/trait.Div.html
//! [`Rem`]: https://doc.rust-lang.org/std/ops/trait.Rem.html
//! [`Shr`]: https://doc.rust-lang.org/std/ops/trait.Shr.html
//! [`Shl`]: https://doc.rust-lang.org/std/ops/trait.Shl.html
//! [`AddAssign`]: https://doc.rust-lang.org/std/ops/trait.AddAssign.html
//! [`SubAssign`]: https://doc.rust-lang.org/std/ops/trait.SubAssign.html
//! [`BitAndAssign`]: https://doc.rust-lang.org/std/ops/trait.BitAndAssign.html
//! [`BitOrAssign`]: https://doc.rust-lang.org/std/ops/trait.BitOrAssign.html
//! [`BitXorAssign`]: https://doc.rust-lang.org/std/ops/trait.BitXorAssign.html

extern crate proc_macro;
#[macro_use]
extern crate quote;
extern crate syn;

use proc_macro::TokenStream;

mod utils;

mod from;
mod add_like;
mod add_assign_like;
mod mul_like;
mod not_like;

macro_rules! create_derive(
    ($mod_:ident, $trait_:ident, $fn_name: ident) => {
        #[proc_macro_derive($trait_)]
        #[doc(hidden)]
        pub fn $fn_name(input: TokenStream) -> TokenStream {
            let s = input.to_string();
            let ast = syn::parse_macro_input(&s).unwrap();
            $mod_::expand(&ast, stringify!($trait_)).parse().unwrap()
        }
    }
);

create_derive!(from, From, from_derive);

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

create_derive!(not_like, Not, not_derive);
create_derive!(not_like, Neg, neg_derive);

create_derive!(add_assign_like, AddAssign,    add_assign_derive);
create_derive!(add_assign_like, SubAssign,    sub_assign_derive);
create_derive!(add_assign_like, BitAndAssign, bit_and_assign_derive);
create_derive!(add_assign_like, BitOrAssign,  bit_or_assign_derive);
create_derive!(add_assign_like, BitXorAssign, bit_xor_assign_derive);
