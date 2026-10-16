#![deny(deprecated)]
#![allow(dead_code)] // some code is tested for type checking only

use super::*;

#[derive(Debug, Error)]
#[deprecated(note = "struct")]
struct Tuple<T>(#[deprecated(note = "field")] T);

#[allow(deprecated)]
impl<T> core::fmt::Display for Tuple<T> {
    derive_display!(@fmt);
}

#[derive(Debug, Error)]
#[deprecated(note = "struct")]
struct Struct {
    #[deprecated(note = "field")]
    source: SimpleErr,
}

#[allow(deprecated)]
impl core::fmt::Display for Struct {
    derive_display!(@fmt);
}

#[derive(Debug, Error)]
#[deprecated(note = "enum")]
enum Enum {
    Unit,
}

#[allow(deprecated)]
impl core::fmt::Display for Enum {
    derive_display!(@fmt);
}

#[derive(Debug, Error)]
enum Variants {
    #[deprecated(note = "variant")]
    Tuple(SimpleErr),
    Struct {
        #[deprecated(note = "field")]
        source: SimpleErr,
    },
}

derive_display!(Variants);
