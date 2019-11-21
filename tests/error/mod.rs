use std::{backtrace::Backtrace, error::Error};

/// Derives `std::fmt::Display` for structs/enums.
/// Derived implementation outputs empty string.
/// Useful, as a way to formally satisfy `Display` trait bound.
///
/// ## Syntax:
///
/// For regular structs/enums:
///
/// ```
/// enum MyEnum {
///     ...
/// }
///
/// derive_display!(MyEnum);
/// ```
///
/// For generic structs/enums:
///
/// ```
/// struct MyGenericStruct<T, U> {
///     ...
/// }
///
/// derive_display!(MyGenericStruct, T, U);
/// ```
macro_rules! derive_display {
    (@fmt) => {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            write!(f, "")
        }
    };
    ($type:ident) => {
        impl ::std::fmt::Display for $type {
            derive_display!(@fmt);
        }
    };
    ($type:ident, $($type_parameters:ident),*) => {
        impl<$($type_parameters),*> ::std::fmt::Display for $type<$($type_parameters),*> {
            derive_display!(@fmt);
        }
    };
}

/// Asserts that backtrace returned by `Error::backtrace` method equals/not-equals
/// backtrace stored in object itself.
///
/// Comparison is done by converting backtraces to strings
/// and then comparing these strings.
///
/// ## Syntax
///
/// * Equals: `assert_bt!(==, ...)`
/// * Not-equals: `assert_bt!(!=, ...)`
///
/// ### Backtrace Access
///
/// Shortcut for named-structs with `backtrace` field.
/// Access backtrace as `error.backtrace`.
///
/// ```
/// assert_bt!(==, error);
/// ```
///
/// Full form for named- and tuple-structs.
/// Access backtrace as `error.some_other_field` and `error.1` respectively.
///
/// ```
/// assert_bt!(!=, error, some_other_field);
/// assert_bt!(==, error, 1);
/// ```
///
/// Access as a method call.
/// Useful for enums (i.e., you can define a method that will match on enum variants
/// and return backtrace for each variant).
/// Access backtrace as `error.get_stored_backtrace_method()`.
///
/// ```
/// assert_bt!(!=, error, .get_stored_backtrace_method);
/// ```
macro_rules! assert_bt {
    (@impl $macro:ident, $error:expr, $backtrace:expr) => {
        $macro!($error.backtrace().unwrap().to_string(), $backtrace.to_string());
    };
    (@expand $macro:ident, $error:expr, .$backtrace:ident) => {
        assert_bt!(@impl $macro, $error, $error.$backtrace())
    };
    (@expand $macro:ident, $error:expr, $backtrace:tt) => {
        assert_bt!(@impl $macro, $error, $error.$backtrace)
    };
    (@expand $macro:ident, $error:expr) => {
        assert_bt!(@expand $macro, $error, backtrace)
    };
    (==, $($args:tt)*) => {
        assert_bt!(@expand assert_eq, $($args)*)
    };
    (!=, $($args:tt)*) => {
        assert_bt!(@expand assert_ne, $($args)*)
    };
}

mod derives_for_structs_with_backtrace;
mod derives_for_structs_with_source;
mod derives_for_structs_with_source_backtrace_chaining;

mod derives_for_enums_with_backtrace;
mod derives_for_enums_with_source;
mod derives_for_enums_with_source_backtrace_chaining;

mod derives_for_generic_structs_with_backtrace;
mod derives_for_generic_structs_with_source;
mod derives_for_generic_structs_with_source_backtrace_chaining;

mod derives_for_generic_enums_with_backtrace;
mod derives_for_generic_enums_with_source;
mod derives_for_generic_enums_with_source_backtrace_chaining;

derive_display!(SimpleErr);
#[derive(Default, Debug, Error)]
struct SimpleErr;

derive_display!(BacktraceErr);
#[derive(Debug)]
struct BacktraceErr {
    backtrace: Backtrace,
}

impl Default for BacktraceErr {
    fn default() -> Self {
        Self {
            backtrace: Backtrace::force_capture(),
        }
    }
}

impl Error for BacktraceErr {
    fn backtrace(&self) -> Option<&Backtrace> {
        Some(&self.backtrace)
    }
}
