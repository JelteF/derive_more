#![cfg_attr(not(feature = "std"), no_std)]
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

#[cfg(feature = "debug")]
pub mod fmt;

#[cfg(any(feature = "add", feature = "not"))]
pub mod ops;

#[cfg(feature = "from_str")]
mod r#str;
#[cfg(feature = "from_str")]
pub use self::r#str::FromStrError;

mod vendor;

// Not public API.
#[doc(hidden)]
pub mod __private {
    pub use crate::vendor::thiserror::aserror::AsDynError;
}

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
    feature = "unwrap"
)))]
compile_error!(
    "at least one derive feature must be enabled (or the \"full\" one enabling all the derives)"
);
