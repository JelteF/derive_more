/// Error returned by the derived [`TryIntoVariant`] implementation.
///
/// [`TryIntoVariant`]: macro@crate::TryIntoVariant
#[derive(Clone, Copy, Debug, PartialEq)]
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

impl<T> core::fmt::Display for TryIntoVariantError<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
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
impl<T: core::fmt::Debug> std::error::Error for TryIntoVariantError<T> {}
