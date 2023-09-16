//! Definitions used in derived implementations of [`core::convert`] traits.

use core::fmt;

/// Error returned by the derived [`TryInto`] implementation.
///
/// [`TryInto`]: macro@crate::TryInto
#[derive(Clone, Copy, Debug)]
pub struct TryIntoError<T> {
    /// Original input value which failed to convert via the derived
    /// [`TryInto`] implementation.
    ///
    /// [`TryInto`]: macro@crate::TryInto
    pub input: T,
    variant_names: &'static str,
    output_type: &'static str,
}

impl<T> TryIntoError<T> {
    #[doc(hidden)]
    #[must_use]
    #[inline]
    pub const fn new(
        input: T,
        variant_names: &'static str,
        output_type: &'static str,
    ) -> Self {
        Self {
            input,
            variant_names,
            output_type,
        }
    }
}

impl<T> fmt::Display for TryIntoError<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Only {} can be converted to {}",
            self.variant_names, self.output_type,
        )
    }
}

#[cfg(feature = "std")]
impl<T: fmt::Debug> std::error::Error for TryIntoError<T> {}

/// Error returned by the derived [`TryFrom`] implementation on enums to
/// convert from their repr.
///
/// [`TryFrom`]: macro@crate::TryFrom
#[derive(Clone, Copy, Debug)]
pub struct TryFromReprError<T> {
    /// Original input value which failed to convert via the derived
    /// [`TryFrom`] implementation.
    ///
    /// [`TryFrom`]: macro@crate::TryFrom
    pub input: T,
}

impl<T> TryFromReprError<T> {
    #[doc(hidden)]
    #[must_use]
    #[inline]
    pub const fn new(input: T) -> Self {
        Self { input }
    }
}

// `T` should only be an integer type and therefore be debug
impl<T: fmt::Debug> fmt::Display for TryFromReprError<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "`{:?}` does not corespond to a unit variant", self.input)
    }
}

#[cfg(feature = "std")]
// `T` should only be an integer type and therefor be debug
impl<T: fmt::Debug> std::error::Error for TryFromReprError<T> {}
