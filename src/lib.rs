// These links overwrite the ones in `README.md`
// to become proper intra-doc links in Rust docs.
//! [`From`]: crate::From
//! [`Into`]: crate::Into
//! [`FromStr`]: crate::FromStr
//! [`TryInto`]: crate::TryInto
//! [`IntoIterator`]: crate::IntoIterator
//! [`AsRef`]: crate::AsRef
//!
//! [`Debug`]: crate::Debug
//! [`Display`-like]: crate::Display
//!
//! [`Error`]: crate::Error
//!
//! [`Index`]: crate::Index
//! [`Deref`]: crate::Deref
//! [`Not`-like]: crate::Not
//! [`Add`-like]: crate::Add
//! [`Mul`-like]: crate::Mul
//! [`Sum`-like]: crate::Sum
//! [`IndexMut`]: crate::IndexMut
//! [`DerefMut`]: crate::DerefMut
//! [`AddAssign`-like]: crate::AddAssign
//! [`MulAssign`-like]: crate::MulAssign
//!
//! [`Constructor`]: crate::Constructor
//! [`IsVariant`]: crate::IsVariant
//! [`Unwrap`]: crate::Unwrap
//! [`TryUnwrap`]: crate::TryUnwrap

// The README includes doctests requiring these features. To make sure that
// tests pass when not all features are provided we exclude it when the
// required features are not available.
#![cfg_attr(
    all(
        feature = "add",
        feature = "display",
        feature = "from",
        feature = "into"
    ),
    doc = include_str!("../README.md")
)]
#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(all(not(feature = "std"), feature = "error"), feature(error_in_core))]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![deny(rustdoc::broken_intra_doc_links, rustdoc::private_intra_doc_links)]
#![forbid(non_ascii_idents, unsafe_code)]
#![warn(clippy::nonstandard_macro_braces)]

#[doc(inline)]
pub use derive_more_impl::*;

#[cfg(feature = "try_into")]
mod convert;
#[cfg(feature = "try_into")]
pub use self::convert::TryIntoError;

#[cfg(feature = "try_unwrap")]
mod try_unwrap;
#[cfg(feature = "try_unwrap")]
pub use self::try_unwrap::TryUnwrapError;

#[cfg(feature = "debug")]
mod fmt;

#[cfg(any(feature = "add", feature = "not"))]
pub mod ops;

#[cfg(feature = "from_str")]
mod r#str;
#[cfg(feature = "from_str")]
pub use self::r#str::FromStrError;

#[cfg(feature = "error")]
mod vendor;

// Not public API.
#[doc(hidden)]
pub mod __private {
    #[cfg(feature = "error")]
    pub use crate::vendor::thiserror::aserror::AsDynError;

    #[cfg(feature = "debug")]
    pub use crate::fmt::{debug_tuple, DebugTuple};
}

// re-export all the traits for easy usage. But hide their docs because otherwise they clutter
// the actual docs
#[cfg(feature = "add")]
#[doc(hidden)]
pub use core::ops::{Add, BitAnd, BitOr, BitXor, Sub};

#[cfg(feature = "add_assign")]
#[doc(hidden)]
pub use core::ops::{AddAssign, BitAndAssign, BitOrAssign, BitXorAssign, SubAssign};

#[cfg(feature = "as_ref")]
#[doc(hidden)]
pub use core::convert::AsRef;

#[cfg(feature = "as_mut")]
#[doc(hidden)]
pub use core::convert::AsMut;

// XXX: Uncommenting this causes our own derive to not be visible anymore, because the derive
// from std takes precedence somehow.
// #[cfg(feature = "debug")]
// #[doc(hidden)]
// pub use core::fmt::Debug;

#[cfg(feature = "deref")]
#[doc(hidden)]
pub use core::ops::Deref;

#[cfg(feature = "deref_mut")]
#[doc(hidden)]
pub use core::ops::DerefMut;

#[cfg(feature = "display")]
#[doc(hidden)]
pub use core::fmt::{
    Binary, Display, LowerExp, LowerHex, Octal, Pointer, UpperExp, UpperHex,
};

#[cfg(all(feature = "error", not(feature = "std")))]
#[doc(hidden)]
pub use core::error::Error;
#[cfg(all(feature = "error", feature = "std"))]
#[doc(hidden)]
pub use std::error::Error;

#[cfg(feature = "from")]
#[doc(hidden)]
pub use core::convert::From;

#[cfg(feature = "from_str")]
#[doc(hidden)]
pub use core::str::FromStr;

#[cfg(feature = "index")]
#[doc(hidden)]
pub use core::ops::Index;

#[cfg(feature = "index_mut")]
#[doc(hidden)]
pub use core::ops::IndexMut;

#[cfg(feature = "into")]
#[doc(hidden)]
pub use core::convert::Into;

#[cfg(feature = "into_iterator")]
#[doc(hidden)]
pub use core::iter::IntoIterator;

#[cfg(feature = "iterator")]
#[doc(hidden)]
pub use core::iter::Iterator;

#[cfg(feature = "mul")]
#[doc(hidden)]
pub use core::ops::{Div, Mul, Rem, Shl, Shr};

#[cfg(feature = "mul_assign")]
#[doc(hidden)]
pub use core::ops::{DivAssign, MulAssign, RemAssign, ShlAssign, ShrAssign};

#[cfg(feature = "not")]
#[doc(hidden)]
pub use core::ops::{Neg, Not};

#[cfg(feature = "sum")]
#[doc(hidden)]
pub use core::iter::{Product, Sum};

#[cfg(feature = "try_into")]
#[doc(hidden)]
pub use core::convert::TryInto;

#[cfg(not(any(
    feature = "full",
    feature = "add_assign",
    feature = "add",
    feature = "as_mut",
    feature = "as_ref",
    feature = "constructor",
    feature = "debug",
    feature = "deref",
    feature = "deref_mut",
    feature = "display",
    feature = "error",
    feature = "from",
    feature = "from_str",
    feature = "index",
    feature = "index_mut",
    feature = "into",
    feature = "into_iterator",
    feature = "is_variant",
    feature = "iterator",
    feature = "mul_assign",
    feature = "mul",
    feature = "not",
    feature = "sum",
    feature = "try_into",
    feature = "unwrap",
    feature = "try_unwrap",
)))]
compile_error!(
    "at least one derive feature must be enabled (or the \"full\" one enabling all the derives)"
);
