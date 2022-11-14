use core::fmt;

#[derive(Debug)]
pub struct TryIntoError<T> {
    pub input: T,
    variant_names: &'static str,
    output_type: &'static str,
}

impl<T> TryIntoError<T> {
    pub fn new(
        input: T,
        variant_names: &'static str,
        output_type: &'static str,
    ) -> TryIntoError<T> {
        TryIntoError {
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
            self.variant_names, self.output_type
        )
    }
}
