% What #[derive(Error)] generates

Deriving `Error` will generate an `Error` implementation, with a `source`
method that matches `self` and each of its variants. In the case of a struct, only
a single variant is available. In the case of an enum, each of its variants is matched.

For each matched variant, a *source-field* is returned, if present. This field can
either be inferred, or explicitly specified via `#[error(source)]` attribute.

**For struct-variants** any field named `source` is inferred as a source-field.

**For tuple-variants** source-field is inferred only if its the only field in the variant.

Any field can be explicitly specified as a source-field via `#[error(source)]` attribute.
And any field, that would have been inferred as a source-field otherwise, can be
explicitly specified as a non-source-field via `#[error(not(source))]` attribute.

# Example usage

```rust
# #[macro_use] extern crate derive_more;

# use std::error::Error as _;

// std::error::Error requires std::fmt::Debug and std::fmt::Display,
// so we can also use derive_more::Display for fully declarative
// error-type definitions.

#[derive(Default, Debug, Display, Error)]
struct Simple;

#[derive(Default, Debug, Display, Error)]
struct WithSource {
    source: Simple,
}

#[derive(Default, Debug, Display, Error)]
struct WithExplicitSource {
    #[error(source)]
    explicit_source: Simple,
}

#[derive(Default, Debug, Display, Error)]
struct Tuple(Simple);

#[derive(Default, Debug, Display, Error)]
struct WithoutSource(#[error(not(source))] i32);

// derive_more::From fits nicely into this pattern as well
#[derive(Debug, Display, Error, From)]
enum CompoundError {
    Simple,
    WithSource {
        source: Simple,
    },
    WithExplicitSource {
        #[error(source)]
        explicit_source: WithSource,
    },
    Tuple(WithExplicitSource),
    WithoutSource(#[error(not(source))] Tuple),
}

fn main() {
    assert!(Simple.source().is_none());
    assert!(WithSource::default().source().is_some());
    assert!(WithExplicitSource::default().source().is_some());
    assert!(Tuple::default().source().is_some());
    assert!(WithoutSource::default().source().is_none());

    assert!(CompoundError::Simple.source().is_none());
    assert!(CompoundError::from(Simple).source().is_some());
    assert!(CompoundError::from(WithSource::default()).source().is_some());
    assert!(CompoundError::from(WithExplicitSource::default()).source().is_some());
    assert!(CompoundError::from(Tuple::default()).source().is_none());
}
```
