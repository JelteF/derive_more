use core::{error::Error, fmt};

/// Error returned by the derived [`TryUnwrap`] implementation.
///
/// [`TryUnwrap`]: macro@crate::TryUnwrap
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TryUnwrapError<T> {
    /// Original input value which failed to convert via the derived
    /// [`TryUnwrap`] implementation.
    ///
    /// [`TryUnwrap`]: macro@crate::TryUnwrap
    pub input: T,
    enum_name: &'static str,
    variant_name: &'static str,
    func_name: &'static str,
}

impl<T> TryUnwrapError<T> {
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

impl<T> fmt::Display for TryUnwrapError<T> {
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

impl<T: fmt::Debug> Error for TryUnwrapError<T> {}
