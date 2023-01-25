//! Definitions used in derived implementations of [`core::ops`] traits.

use core::fmt;

/// Error returned by the derived implementations when an arithmetic or logic
/// operation is invoked on a unit-like variant of an enum.
#[derive(Clone, Copy, Debug)]
pub struct UnitError {
    operation_name: &'static str,
}

impl UnitError {
    #[doc(hidden)]
    #[inline]
    pub const fn new(
        operation_name: &'static str,
    ) -> Self {
        Self {
            operation_name
        }
    }
}

impl fmt::Display for UnitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Cannot {}() unit variants", self.operation_name)
    }
}

#[cfg(feature = "std")]
impl std::error::Error for UnitError {}

#[cfg(feature = "add")]
/// Error returned by the derived implementations when an arithmetic or logic
/// operation is invoked on mismatched enum variants.
pub struct WrongVariantError {
    operation_name: &'static str,
}

#[cfg(feature = "add")]
impl WrongVariantError {
    #[doc(hidden)]
    #[inline]
    pub const fn new(
        operation_name: &'static str,
    ) -> Self {
        Self {
            operation_name
        }
    }
}

#[cfg(feature = "add")]
impl fmt::Display for WrongVariantError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Trying to {}() mismatched enum variants", self.operation_name)
    }
}

#[cfg(all(feature = "add", feature = "std"))]
impl std::error::Error for WrongVariantError {}
