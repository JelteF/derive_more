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

/// Error returned by the derived [`TryIntoVariant`] implementation.
///
/// [`TryIntoVariant`]: macro@crate::TryIntoVariant
#[derive(Clone, Copy, Debug)]
pub struct TryIntoVariantError<T> {
    /// Original input value which failed to convert via the derived
    /// [`TryIntoVariant`] implementation.
    ///
    /// [`TryIntoVariant`]: macro@crate::TryIntoVariant
    pub input: T,
    enum_name: &'static str,
    variant_name: &'static str,
    func_name: &'static str,
}

impl<T> TryIntoVariantError<T> {
    #[doc(hidden)]
    #[must_use]
    #[inline]
    pub const fn new(
        input: T,
        enum_name: &'static str,
        variant_name: &'static str,
        func_name: &'static str,
    ) -> Self {
        Self {
            input,
            enum_name,
            variant_name,
            func_name,
        }
    }
}

impl<T> fmt::Display for TryIntoVariantError<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Attempt to call `{enum_name}::{func_name}()` on a `{enum_name}::{variant_name}` value",
            enum_name = self.enum_name,
            variant_name = self.variant_name,
            func_name = self.func_name,
        )
    }
}

#[cfg(feature = "std")]
impl<T: fmt::Debug> std::error::Error for TryIntoVariantError<T> {}
