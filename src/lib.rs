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

// Not public, but exported API. For macro expansion internals only.
#[doc(hidden)]
pub mod __private {
    #[cfg(feature = "debug")]
    pub use crate::fmt::{debug_tuple, DebugTuple};

    #[cfg(feature = "error")]
    pub use crate::vendor::thiserror::aserror::AsDynError;
}

/// Module containing macro definitions only, without corresponding traits.
///
/// Use it in your import paths, if you don't want to import traits, but only macros.
pub mod macros {
    #[doc(inline)]
    pub use derive_more_impl::*;
}

#[cfg(any(feature = "add", feature = "not"))]
pub mod ops;
#[cfg(feature = "add")]
mod gimmick_add {
    pub use crate::macros::{Add, BitAnd, BitOr, BitXor, Sub};
    pub use core::ops::*;
}
#[cfg(feature = "add")]
#[doc(inline)]
pub use self::gimmick_add::{Add, BitAnd, BitOr, BitXor, Sub};

#[cfg(feature = "add_assign")]
mod gimmick_add_assign {
    pub use crate::macros::{
        AddAssign, BitAndAssign, BitOrAssign, BitXorAssign, SubAssign,
    };
    pub use core::ops::*;
}
#[cfg(feature = "add_assign")]
#[doc(inline)]
pub use self::gimmick_add_assign::{
    AddAssign, BitAndAssign, BitOrAssign, BitXorAssign, SubAssign,
};

#[cfg(feature = "as_mut")]
mod gimmick_as_mut {
    pub use crate::macros::AsMut;
    pub use core::convert::*;
}
#[cfg(feature = "as_mut")]
#[doc(inline)]
pub use self::gimmick_as_mut::AsMut;

#[cfg(feature = "as_ref")]
mod gimmick_as_ref {
    pub use crate::macros::AsRef;
    pub use core::convert::*;
}
#[cfg(feature = "as_ref")]
#[doc(inline)]
pub use self::gimmick_as_ref::AsRef;

#[cfg(feature = "constructor")]
#[doc(inline)]
pub use self::macros::Constructor;

#[cfg(feature = "debug")]
mod fmt;
#[cfg(feature = "debug")]
mod gimmick_debug {
    pub use crate::macros::Debug;
    pub use core::fmt::*;
}
#[cfg(feature = "debug")]
#[doc(inline)]
pub use self::gimmick_debug::Debug;

#[cfg(feature = "deref")]
mod gimmick_deref {
    pub use crate::macros::Deref;
    pub use core::ops::*;
}
#[cfg(feature = "deref")]
#[doc(inline)]
pub use self::gimmick_deref::Deref;

#[cfg(feature = "deref_mut")]
mod gimmick_deref_mut {
    pub use crate::macros::DerefMut;
    pub use core::ops::*;
}
#[cfg(feature = "deref_mut")]
#[doc(inline)]
pub use self::gimmick_deref_mut::DerefMut;

#[cfg(feature = "display")]
mod gimmick_display {
    pub use crate::macros::{
        Binary, Display, LowerExp, LowerHex, Octal, Pointer, UpperExp, UpperHex,
    };
    pub use core::fmt::*;
}
#[cfg(feature = "display")]
#[doc(inline)]
pub use self::gimmick_display::{
    Binary, Display, LowerExp, LowerHex, Octal, Pointer, UpperExp, UpperHex,
};

#[cfg(feature = "error")]
mod vendor;
#[cfg(feature = "error")]
mod gimmick_error {
    pub use crate::macros::Error;
    #[cfg(not(feature = "std"))]
    pub use core::error::*;
    #[cfg(feature = "std")]
    pub use std::error::*;
}
#[cfg(feature = "error")]
pub use self::gimmick_error::Error;

#[cfg(feature = "from")]
mod gimmick_from {
    pub use crate::macros::From;
    pub use core::convert::*;
}
#[cfg(feature = "from")]
#[doc(inline)]
pub use self::gimmick_from::From;

#[cfg(feature = "from_str")]
mod r#str;
#[cfg(feature = "from_str")]
mod gimmick_from_str {
    pub use crate::{macros::FromStr, r#str::FromStrError};
    pub use core::str::*;
}
#[cfg(feature = "from_str")]
#[doc(hidden)]
pub use self::gimmick_from_str::{FromStr, FromStrError};

#[cfg(feature = "index")]
mod gimmick_index {
    pub use crate::macros::Index;
    pub use core::ops::*;
}
#[cfg(feature = "index")]
#[doc(inline)]
pub use self::gimmick_index::Index;

#[cfg(feature = "index_mut")]
mod gimmick_index_mut {
    pub use crate::macros::IndexMut;
    pub use core::ops::*;
}
#[cfg(feature = "index_mut")]
#[doc(inline)]
pub use self::gimmick_index_mut::IndexMut;

#[cfg(feature = "into")]
mod gimmick_into {
    pub use crate::macros::Into;
    pub use core::convert::*;
}
#[cfg(feature = "into")]
#[doc(inline)]
pub use self::gimmick_into::Into;

#[cfg(feature = "into_iterator")]
mod gimmick_into_iterator {
    pub use crate::macros::IntoIterator;
    pub use core::iter::*;
}
#[cfg(feature = "into_iterator")]
#[doc(inline)]
pub use self::gimmick_into_iterator::IntoIterator;

#[cfg(feature = "is_variant")]
#[doc(inline)]
pub use self::macros::IsVariant;

#[cfg(feature = "mul")]
mod gimmick_mul {
    pub use crate::macros::{Div, Mul, Rem, Shl, Shr};
    pub use core::ops::*;
}
#[cfg(feature = "mul")]
#[doc(inline)]
pub use self::gimmick_mul::{Div, Mul, Rem, Shl, Shr};

#[cfg(feature = "mul_assign")]
mod gimmick_mul_assign {
    pub use crate::macros::{DivAssign, MulAssign, RemAssign, ShlAssign, ShrAssign};
    pub use core::ops::*;
}
#[cfg(feature = "mul_assign")]
#[doc(inline)]
pub use self::gimmick_mul_assign::{
    DivAssign, MulAssign, RemAssign, ShlAssign, ShrAssign,
};

#[cfg(feature = "not")]
mod gimmick_not {
    pub use crate::macros::{Neg, Not};
    pub use core::ops::*;
}
#[cfg(feature = "not")]
#[doc(inline)]
pub use self::gimmick_not::{Neg, Not};

#[cfg(feature = "sum")]
mod gimmick_sum {
    pub use crate::macros::{Product, Sum};
    pub use core::iter::*;
}
#[cfg(feature = "sum")]
#[doc(inline)]
pub use self::gimmick_sum::{Product, Sum};

#[cfg(feature = "try_into")]
mod convert;
#[cfg(feature = "try_into")]
mod gimmick_try_into {
    pub use crate::{convert::TryIntoError, macros::TryInto};
    pub use core::convert::*;
}
#[cfg(feature = "try_into")]
#[doc(inline)]
pub use self::gimmick_try_into::{TryInto, TryIntoError};

#[cfg(feature = "try_unwrap")]
mod try_unwrap;
#[cfg(feature = "try_unwrap")]
#[doc(inline)]
pub use self::{macros::TryUnwrap, try_unwrap::TryUnwrapError};

#[cfg(feature = "unwrap")]
#[doc(inline)]
pub use self::macros::Unwrap;

#[cfg(not(any(
    feature = "full",
    feature = "add",
    feature = "add_assign",
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
    feature = "mul",
    feature = "mul_assign",
    feature = "not",
    feature = "sum",
    feature = "try_into",
    feature = "try_unwrap",
    feature = "unwrap",
)))]
compile_error!(
    "at least one derive feature must be enabled (or the \"full\" one enabling all the derives)"
);
