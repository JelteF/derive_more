#[macro_use]
extern crate derive_more;

use std::error::Error as _;

macro_rules! derive_display {
    (@fmt) => {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            write!(f, "")
        }
    };
    ($type:ident) => {
        impl ::std::fmt::Display for $type {
            derive_display! {@fmt}
        }
    };
    ($type:ident, $($type_parameters:ident),*) => {
        impl<$($type_parameters),*> ::std::fmt::Display for $type<$($type_parameters),*> {
            derive_display! {@fmt}
        }
    };
}

mod derives_for_struct {
    use super::*;

    #[derive(Default, Debug, Error)]
    struct SimpleErr;

    derive_display! {SimpleErr}

    #[test]
    fn unit() {
        assert!(SimpleErr.source().is_none());
    }

    #[test]
    fn named_implicit_no_source() {
        #[derive(Default, Debug, Error)]
        struct TestErr {
            field: i32,
        }

        derive_display! {TestErr}

        assert!(TestErr::default().source().is_none());
    }

    #[test]
    fn named_implicit_source() {
        #[derive(Default, Debug, Error)]
        struct TestErr {
            source: SimpleErr,
            field: i32,
        }

        derive_display! {TestErr}

        assert!(TestErr::default().source().is_some());
        assert!(TestErr::default().source().unwrap().is::<SimpleErr>());
    }

    #[test]
    fn named_explicit_no_source() {
        #[derive(Default, Debug, Error)]
        struct TestErr {
            #[error(not(source))]
            source: SimpleErr,
            field: i32,
        }

        derive_display! {TestErr}

        assert!(TestErr::default().source().is_none());
    }

    #[test]
    fn named_explicit_source() {
        #[derive(Default, Debug, Error)]
        struct TestErr {
            #[error(source)]
            explicit_source: SimpleErr,
            field: i32,
        }

        derive_display! {TestErr}

        assert!(TestErr::default().source().is_some());
        assert!(TestErr::default().source().unwrap().is::<SimpleErr>());
    }

    #[test]
    fn named_explicit_no_source_redundant() {
        #[derive(Default, Debug, Error)]
        struct TestErr {
            #[error(not(source))]
            field: i32,
        }

        derive_display! {TestErr}

        assert!(TestErr::default().source().is_none());
    }

    #[test]
    fn named_explicit_source_redundant() {
        #[derive(Default, Debug, Error)]
        struct TestErr {
            #[error(source)]
            source: SimpleErr,
            field: i32,
        }

        derive_display! {TestErr}

        assert!(TestErr::default().source().is_some());
        assert!(TestErr::default().source().unwrap().is::<SimpleErr>());
    }

    #[test]
    fn named_explicit_suppresses_implicit() {
        #[derive(Default, Debug, Error)]
        struct TestErr {
            source: i32,
            #[error(source)]
            field: SimpleErr,
        }

        derive_display! {TestErr}

        assert!(TestErr::default().source().is_some());
        assert!(TestErr::default().source().unwrap().is::<SimpleErr>());
    }

    #[test]
    fn unnamed_implicit_no_source() {
        #[derive(Default, Debug, Error)]
        struct TestErr(i32, i32);

        derive_display! {TestErr}

        assert!(TestErr::default().source().is_none());
    }

    #[test]
    fn unnamed_implicit_source() {
        #[derive(Default, Debug, Error)]
        struct TestErr(SimpleErr);

        derive_display! {TestErr}

        assert!(TestErr::default().source().is_some());
        assert!(TestErr::default().source().unwrap().is::<SimpleErr>());
    }

    #[test]
    fn unnamed_explicit_no_source() {
        #[derive(Default, Debug, Error)]
        struct TestErr(#[error(not(source))] SimpleErr);

        derive_display! {TestErr}

        assert!(TestErr::default().source().is_none());
    }

    #[test]
    fn unnamed_explicit_source() {
        #[derive(Default, Debug, Error)]
        struct TestErr(#[error(source)] SimpleErr, i32);

        derive_display! {TestErr}

        assert!(TestErr::default().source().is_some());
        assert!(TestErr::default().source().unwrap().is::<SimpleErr>());
    }

    #[test]
    fn unnamed_explicit_no_source_redundant() {
        #[derive(Default, Debug, Error)]
        struct TestErr(#[error(not(source))] i32, #[error(not(source))] i32);

        derive_display! {TestErr}

        assert!(TestErr::default().source().is_none());
    }

    #[test]
    fn unnamed_explicit_source_redundant() {
        #[derive(Default, Debug, Error)]
        struct TestErr(#[error(source)] SimpleErr);

        derive_display! {TestErr}

        assert!(TestErr::default().source().is_some());
        assert!(TestErr::default().source().unwrap().is::<SimpleErr>());
    }

    #[test]
    fn named_ignore() {
        #[derive(Default, Debug, Error)]
        struct TestErr {
            #[error(ignore)]
            source: SimpleErr,
            field: i32,
        }

        derive_display! {TestErr}

        assert!(TestErr::default().source().is_none());
    }

    #[test]
    fn unnamed_ignore() {
        #[derive(Default, Debug, Error)]
        struct TestErr(#[error(ignore)] SimpleErr);

        derive_display! {TestErr}

        assert!(TestErr::default().source().is_none());
    }

    #[test]
    fn named_ignore_redundant() {
        #[derive(Default, Debug, Error)]
        struct TestErr {
            #[error(ignore)]
            field: i32,
        }

        derive_display! {TestErr}

        assert!(TestErr::default().source().is_none());
    }

    #[test]
    fn unnamed_ignore_redundant() {
        #[derive(Default, Debug, Error)]
        struct TestErr(#[error(ignore)] i32, #[error(ignore)] i32);

        derive_display! {TestErr}

        assert!(TestErr::default().source().is_none());
    }

    #[test]
    fn named_struct_ignore() {
        #[derive(Default, Debug, Error)]
        #[error(ignore)]
        struct TestErr {
            source: SimpleErr,
            field: i32,
        }

        derive_display! {TestErr}

        assert!(TestErr::default().source().is_none())
    }

    #[test]
    fn unnamed_struct_ignore() {
        #[derive(Default, Debug, Error)]
        #[error(ignore)]
        struct TestErr(SimpleErr);

        derive_display! {TestErr}

        assert!(TestErr::default().source().is_none())
    }

    #[test]
    fn named_struct_ignore_redundant() {
        #[derive(Default, Debug, Error)]
        #[error(ignore)]
        struct TestErr {
            field: i32,
        }

        derive_display! {TestErr}

        assert!(TestErr::default().source().is_none())
    }

    #[test]
    fn unnamed_struct_ignore_redundant() {
        #[derive(Default, Debug, Error)]
        #[error(ignore)]
        struct TestErr(i32, i32);

        derive_display! {TestErr}

        assert!(TestErr::default().source().is_none())
    }
}

mod derives_for_enum {
    use super::*;

    #[derive(Default, Debug, Error)]
    struct SimpleErr;

    derive_display! {SimpleErr}

    #[derive(Debug, Error)]
    enum TestErr {
        Unit,
        NamedImplicitNoSource {
            field: i32,
        },
        NamedImplicitSource {
            source: SimpleErr,
            field: i32,
        },
        NamedExplicitNoSource {
            #[error(not(source))]
            source: SimpleErr,
            field: i32,
        },
        NamedExplicitSource {
            #[error(source)]
            explicit_source: SimpleErr,
            field: i32,
        },
        NamedExplicitNoSourceRedundant {
            #[error(not(source))]
            field: i32,
        },
        NamedExplicitSourceRedundant {
            #[error(source)]
            source: SimpleErr,
            field: i32,
        },
        NamedExplicitSuppressesImplicit {
            source: i32,
            #[error(source)]
            field: SimpleErr,
        },
        UnnamedImplicitNoSource(i32, i32),
        UnnamedImplicitSource(SimpleErr),
        UnnamedExplicitNoSource(#[error(not(source))] SimpleErr),
        UnnamedExplicitSource(#[error(source)] SimpleErr, i32),
        UnnamedExplicitNoSourceRedundant(
            #[error(not(source))] i32,
            #[error(not(source))] i32,
        ),
        UnnamedExplicitSourceRedundant(#[error(source)] SimpleErr),
        NamedIgnore {
            #[error(ignore)]
            source: SimpleErr,
            field: i32,
        },
        UnnamedIgnore(#[error(ignore)] SimpleErr),
        NamedIgnoreRedundant {
            #[error(ignore)]
            field: i32,
        },
        UnnamedIgnoreRedundant(#[error(ignore)] i32, #[error(ignore)] i32),
        #[error(ignore)]
        NamedVariantIgnore {
            source: SimpleErr,
            field: i32,
        },
        #[error(ignore)]
        UnnamedVariantIgnore(SimpleErr),
        #[error(ignore)]
        NamedVariantIgnoreRedundant {
            field: i32,
        },
        #[error(ignore)]
        UnnamedVariantIgnoreRedundant(i32, i32),
    }

    derive_display! {TestErr}

    #[test]
    fn unit() {
        assert!(TestErr::Unit.source().is_none());
    }

    #[test]
    fn named_implicit_no_source() {
        let err = TestErr::NamedImplicitNoSource { field: 0 };

        assert!(err.source().is_none());
    }

    #[test]
    fn named_implicit_source() {
        let err = TestErr::NamedImplicitSource {
            source: SimpleErr::default(),
            field: 0,
        };

        assert!(err.source().is_some());
        assert!(err.source().unwrap().is::<SimpleErr>());
    }

    #[test]
    fn named_explicit_no_source() {
        let err = TestErr::NamedExplicitNoSource {
            source: SimpleErr::default(),
            field: 0,
        };

        assert!(err.source().is_none());
    }

    #[test]
    fn named_explicit_source() {
        let err = TestErr::NamedExplicitSource {
            explicit_source: SimpleErr::default(),
            field: 0,
        };

        assert!(err.source().is_some());
        assert!(err.source().unwrap().is::<SimpleErr>());
    }

    #[test]
    fn named_explicit_no_source_redundant() {
        let err = TestErr::NamedExplicitNoSourceRedundant { field: 0 };

        assert!(err.source().is_none());
    }

    #[test]
    fn named_explicit_source_redundant() {
        let err = TestErr::NamedExplicitSourceRedundant {
            source: SimpleErr::default(),
            field: 0,
        };

        assert!(err.source().is_some());
        assert!(err.source().unwrap().is::<SimpleErr>());
    }

    #[test]
    fn named_explicit_suppresses_implicit() {
        let err = TestErr::NamedExplicitSuppressesImplicit {
            source: 0,
            field: SimpleErr::default(),
        };

        assert!(err.source().is_some());
        assert!(err.source().unwrap().is::<SimpleErr>());
    }

    #[test]
    fn unnamed_implicit_no_source() {
        assert!(TestErr::UnnamedImplicitNoSource(0, 0).source().is_none());
    }

    #[test]
    fn unnamed_implicit_source() {
        let err = TestErr::UnnamedImplicitSource(SimpleErr::default());

        assert!(err.source().is_some());
        assert!(err.source().unwrap().is::<SimpleErr>());
    }

    #[test]
    fn unnamed_explicit_no_source() {
        let err = TestErr::UnnamedExplicitNoSource(SimpleErr::default());

        assert!(err.source().is_none());
    }

    #[test]
    fn unnamed_explicit_source() {
        let err = TestErr::UnnamedExplicitSource(SimpleErr::default(), 0);

        assert!(err.source().is_some());
        assert!(err.source().unwrap().is::<SimpleErr>());
    }

    #[test]
    fn unnamed_explicit_no_source_redundant() {
        let err = TestErr::UnnamedExplicitNoSourceRedundant(0, 0);

        assert!(err.source().is_none());
    }

    #[test]
    fn unnamed_explicit_source_redundant() {
        let err = TestErr::UnnamedExplicitSourceRedundant(SimpleErr::default());

        assert!(err.source().is_some());
        assert!(err.source().unwrap().is::<SimpleErr>());
    }

    #[test]
    fn named_ignore() {
        let err = TestErr::NamedIgnore {
            source: SimpleErr::default(),
            field: 0,
        };

        assert!(err.source().is_none());
    }

    #[test]
    fn unnamed_ignore() {
        let err = TestErr::UnnamedIgnore(SimpleErr::default());

        assert!(err.source().is_none());
    }

    #[test]
    fn named_ignore_redundant() {
        let err = TestErr::NamedIgnoreRedundant { field: 0 };

        assert!(err.source().is_none());
    }

    #[test]
    fn unnamed_ignore_redundant() {
        let err = TestErr::UnnamedIgnoreRedundant(0, 0);

        assert!(err.source().is_none());
    }

    #[test]
    fn named_variant_ignore() {
        let err = TestErr::NamedVariantIgnore {
            source: SimpleErr::default(),
            field: 0,
        };

        assert!(err.source().is_none());
    }

    #[test]
    fn unnamed_variant_ignore() {
        let err = TestErr::UnnamedVariantIgnore(SimpleErr::default());

        assert!(err.source().is_none())
    }

    #[test]
    fn named_variant_ignore_redundant() {
        let err = TestErr::NamedVariantIgnoreRedundant { field: 0 };

        assert!(err.source().is_none());
    }

    #[test]
    fn unnamed_variant_ignore_redundant() {
        let err = TestErr::UnnamedVariantIgnoreRedundant(0, 0);

        assert!(err.source().is_none())
    }
}

mod derives_for_generic_struct {
    use super::*;

    #[derive(Default, Debug, Error)]
    struct SimpleErr;

    derive_display! {SimpleErr}

    #[test]
    fn named_implicit_no_source() {
        #[derive(Default, Debug, Error)]
        struct TestErr<T> {
            field: T,
        }

        derive_display! {TestErr, T}

        assert!(TestErr::<i32>::default().source().is_none());
    }

    #[test]
    fn named_implicit_source() {
        #[derive(Default, Debug, Error)]
        struct TestErr<E, T> {
            source: E,
            field: T,
        }

        derive_display! {TestErr, E, T}

        let err = TestErr::<SimpleErr, i32>::default();

        assert!(err.source().is_some());
        assert!(err.source().unwrap().is::<SimpleErr>());
    }

    #[test]
    fn named_explicit_no_source() {
        #[derive(Default, Debug, Error)]
        struct TestErr<E, T> {
            #[error(not(source))]
            source: E,
            field: T,
        }

        derive_display! {TestErr, E, T}

        let err = TestErr::<SimpleErr, i32>::default();

        assert!(err.source().is_none());
    }

    #[test]
    fn named_explicit_source() {
        #[derive(Default, Debug, Error)]
        struct TestErr<E, T> {
            #[error(source)]
            explicit_source: E,
            field: T,
        }

        derive_display! {TestErr, E, T}

        let err = TestErr::<SimpleErr, i32>::default();

        assert!(err.source().is_some());
        assert!(err.source().unwrap().is::<SimpleErr>());
    }

    #[test]
    fn named_explicit_no_source_redundant() {
        #[derive(Default, Debug, Error)]
        struct TestErr<T> {
            #[error(not(source))]
            field: T,
        }

        derive_display! {TestErr, T}

        assert!(TestErr::<i32>::default().source().is_none());
    }

    #[test]
    fn named_explicit_source_redundant() {
        #[derive(Default, Debug, Error)]
        struct TestErr<E, T> {
            #[error(source)]
            source: E,
            field: T,
        }

        derive_display! {TestErr, E, T}

        let err = TestErr::<SimpleErr, i32>::default();

        assert!(err.source().is_some());
        assert!(err.source().unwrap().is::<SimpleErr>());
    }

    #[test]
    fn named_explicit_suppresses_implicit() {
        #[derive(Default, Debug, Error)]
        struct TestErr<E, T> {
            source: E,
            #[error(source)]
            field: T,
        }

        derive_display! {TestErr, E, T}

        let err = TestErr::<i32, SimpleErr>::default();

        assert!(err.source().is_some());
        assert!(err.source().unwrap().is::<SimpleErr>());
    }

    #[test]
    fn unnamed_implicit_no_source() {
        #[derive(Default, Debug, Error)]
        struct TestErr<T>(T, T);

        derive_display! {TestErr, T}

        assert!(TestErr::<i32>::default().source().is_none());
    }

    #[test]
    fn unnamed_implicit_source() {
        #[derive(Default, Debug, Error)]
        struct TestErr<E>(E);

        derive_display! {TestErr, E}

        let err = TestErr::<SimpleErr>::default();

        assert!(err.source().is_some());
        assert!(err.source().unwrap().is::<SimpleErr>());
    }

    #[test]
    fn unnamed_explicit_no_source() {
        #[derive(Default, Debug, Error)]
        struct TestErr<E>(#[error(not(source))] E);

        derive_display! {TestErr, E}

        assert!(TestErr::<SimpleErr>::default().source().is_none());
    }

    #[test]
    fn unnamed_explicit_source() {
        #[derive(Default, Debug, Error)]
        struct TestErr<E, T>(#[error(source)] E, T);

        derive_display! {TestErr, E, T}

        let err = TestErr::<SimpleErr, i32>::default();

        assert!(err.source().is_some());
        assert!(err.source().unwrap().is::<SimpleErr>());
    }

    #[test]
    fn unnamed_explicit_no_source_redundant() {
        #[derive(Default, Debug, Error)]
        struct TestErr<T>(#[error(not(source))] T, #[error(not(source))] T);

        derive_display! {TestErr, T}

        assert!(TestErr::<i32>::default().source().is_none());
    }

    #[test]
    fn unnamed_explicit_source_redundant() {
        #[derive(Default, Debug, Error)]
        struct TestErr<E>(#[error(source)] E);

        derive_display! {TestErr, E}

        let err = TestErr::<SimpleErr>::default();

        assert!(err.source().is_some());
        assert!(err.source().unwrap().is::<SimpleErr>());
    }

    #[test]
    fn named_ignore() {
        #[derive(Default, Debug, Error)]
        struct TestErr<E, T> {
            #[error(ignore)]
            source: E,
            field: T,
        }

        derive_display! {TestErr, E, T}

        assert!(TestErr::<SimpleErr, i32>::default().source().is_none());
    }

    #[test]
    fn unnamed_ignore() {
        #[derive(Default, Debug, Error)]
        struct TestErr<E>(#[error(ignore)] E);

        derive_display! {TestErr, E}

        assert!(TestErr::<SimpleErr>::default().source().is_none());
    }

    #[test]
    fn named_ignore_redundant() {
        #[derive(Default, Debug, Error)]
        struct TestErr<T> {
            #[error(ignore)]
            field: T,
        }

        derive_display! {TestErr, T}

        assert!(TestErr::<i32>::default().source().is_none());
    }

    #[test]
    fn unnamed_ignore_redundant() {
        #[derive(Default, Debug, Error)]
        struct TestErr<T>(#[error(ignore)] T, #[error(ignore)] T);

        derive_display! {TestErr, T}

        assert!(TestErr::<i32>::default().source().is_none());
    }

    #[test]
    fn named_struct_ignore() {
        #[derive(Default, Debug, Error)]
        #[error(ignore)]
        struct TestErr<E, T> {
            source: E,
            field: T,
        }

        derive_display! {TestErr, E, T}

        assert!(TestErr::<SimpleErr, i32>::default().source().is_none())
    }

    #[test]
    fn unnamed_struct_ignore() {
        #[derive(Default, Debug, Error)]
        #[error(ignore)]
        struct TestErr<E>(E);

        derive_display! {TestErr, E}

        assert!(TestErr::<SimpleErr>::default().source().is_none())
    }

    #[test]
    fn named_struct_ignore_redundant() {
        #[derive(Default, Debug, Error)]
        #[error(ignore)]
        struct TestErr<T> {
            field: T,
        }

        derive_display! {TestErr, T}

        assert!(TestErr::<i32>::default().source().is_none())
    }

    #[test]
    fn unnamed_struct_ignore_redundant() {
        #[derive(Default, Debug, Error)]
        #[error(ignore)]
        struct TestErr<T>(T, T);

        derive_display! {TestErr, T}

        assert!(TestErr::<i32>::default().source().is_none())
    }
}

mod derives_for_generic_enum {
    use super::*;

    #[derive(Default, Debug, Error)]
    struct SimpleErr;

    derive_display! {SimpleErr}

    #[derive(Debug, Error)]
    enum TestErr<E, T> {
        Unit,
        NamedImplicitNoSource {
            field: T,
        },
        NamedImplicitSource {
            source: E,
            field: T,
        },
        NamedExplicitNoSource {
            #[error(not(source))]
            source: E,
            field: T,
        },
        NamedExplicitSource {
            #[error(source)]
            explicit_source: E,
            field: T,
        },
        NamedExplicitNoSourceRedundant {
            #[error(not(source))]
            field: T,
        },
        NamedExplicitSourceRedundant {
            #[error(source)]
            source: E,
            field: T,
        },
        NamedExplicitSuppressesImplicit {
            source: T,
            #[error(source)]
            field: E,
        },
        UnnamedImplicitNoSource(T, T),
        UnnamedImplicitSource(E),
        UnnamedExplicitNoSource(#[error(not(source))] E),
        UnnamedExplicitSource(#[error(source)] E, T),
        UnnamedExplicitNoSourceRedundant(
            #[error(not(source))] T,
            #[error(not(source))] T,
        ),
        UnnamedExplicitSourceRedundant(#[error(source)] E),
        NamedIgnore {
            #[error(ignore)]
            source: E,
            field: T,
        },
        UnnamedIgnore(#[error(ignore)] E),
        NamedIgnoreRedundant {
            #[error(ignore)]
            field: T,
        },
        UnnamedIgnoreRedundant(#[error(ignore)] T, #[error(ignore)] T),
        #[error(ignore)]
        NamedVariantIgnore {
            source: E,
            field: T,
        },
        #[error(ignore)]
        UnnamedVariantIgnore(E),
        #[error(ignore)]
        NamedVariantIgnoreRedundant {
            field: T,
        },
        #[error(ignore)]
        UnnamedVariantIgnoreRedundant(T, T),
    }

    derive_display! {TestErr, T, E}

    #[test]
    fn unit() {
        assert!(TestErr::<SimpleErr, i32>::Unit.source().is_none());
    }

    #[test]
    fn named_implicit_no_source() {
        let err = TestErr::<SimpleErr, _>::NamedImplicitNoSource { field: 0 };

        assert!(err.source().is_none());
    }

    #[test]
    fn named_implicit_source() {
        let err = TestErr::NamedImplicitSource {
            source: SimpleErr::default(),
            field: 0,
        };

        assert!(err.source().is_some());
        assert!(err.source().unwrap().is::<SimpleErr>());
    }

    #[test]
    fn named_explicit_no_source() {
        let err = TestErr::NamedExplicitNoSource {
            source: SimpleErr::default(),
            field: 0,
        };

        assert!(err.source().is_none());
    }

    #[test]
    fn named_explicit_source() {
        let err = TestErr::NamedExplicitSource {
            explicit_source: SimpleErr::default(),
            field: 0,
        };

        assert!(err.source().is_some());
        assert!(err.source().unwrap().is::<SimpleErr>());
    }

    #[test]
    fn named_explicit_no_source_redundant() {
        let err = TestErr::<SimpleErr, _>::NamedExplicitNoSourceRedundant { field: 0 };

        assert!(err.source().is_none());
    }

    #[test]
    fn named_explicit_source_redundant() {
        let err = TestErr::NamedExplicitSourceRedundant {
            source: SimpleErr::default(),
            field: 0,
        };

        assert!(err.source().is_some());
        assert!(err.source().unwrap().is::<SimpleErr>());
    }

    #[test]
    fn named_explicit_suppresses_implicit() {
        let err = TestErr::NamedExplicitSuppressesImplicit {
            source: 0,
            field: SimpleErr::default(),
        };

        assert!(err.source().is_some());
        assert!(err.source().unwrap().is::<SimpleErr>());
    }

    #[test]
    fn unnamed_implicit_no_source() {
        let err = TestErr::<SimpleErr, _>::UnnamedImplicitNoSource(0, 0);

        assert!(err.source().is_none());
    }

    #[test]
    fn unnamed_implicit_source() {
        let err = TestErr::<_, i32>::UnnamedImplicitSource(SimpleErr::default());

        assert!(err.source().is_some());
        assert!(err.source().unwrap().is::<SimpleErr>());
    }

    #[test]
    fn unnamed_explicit_no_source() {
        let err = TestErr::<_, i32>::UnnamedExplicitNoSource(SimpleErr::default());

        assert!(err.source().is_none());
    }

    #[test]
    fn unnamed_explicit_source() {
        let err = TestErr::UnnamedExplicitSource(SimpleErr::default(), 0);

        assert!(err.source().is_some());
        assert!(err.source().unwrap().is::<SimpleErr>());
    }

    #[test]
    fn unnamed_explicit_no_source_redundant() {
        let err = TestErr::<SimpleErr, _>::UnnamedExplicitNoSourceRedundant(0, 0);

        assert!(err.source().is_none());
    }

    #[test]
    fn unnamed_explicit_source_redundant() {
        let err =
            TestErr::<_, i32>::UnnamedExplicitSourceRedundant(SimpleErr::default());

        assert!(err.source().is_some());
        assert!(err.source().unwrap().is::<SimpleErr>());
    }

    #[test]
    fn named_ignore() {
        let err = TestErr::NamedIgnore {
            source: SimpleErr::default(),
            field: 0,
        };

        assert!(err.source().is_none());
    }

    #[test]
    fn unnamed_ignore() {
        let err = TestErr::<_, i32>::UnnamedIgnore(SimpleErr::default());

        assert!(err.source().is_none());
    }

    #[test]
    fn named_ignore_redundant() {
        let err = TestErr::<SimpleErr, _>::NamedIgnoreRedundant { field: 0 };

        assert!(err.source().is_none());
    }

    #[test]
    fn unnamed_ignore_redundant() {
        let err = TestErr::<SimpleErr, _>::UnnamedIgnoreRedundant(0, 0);

        assert!(err.source().is_none());
    }

    #[test]
    fn named_variant_ignore() {
        let err = TestErr::NamedVariantIgnore {
            source: SimpleErr::default(),
            field: 0,
        };

        assert!(err.source().is_none());
    }

    #[test]
    fn unnamed_variant_ignore() {
        let err = TestErr::<_, i32>::UnnamedVariantIgnore(SimpleErr::default());

        assert!(err.source().is_none())
    }

    #[test]
    fn named_variant_ignore_redundant() {
        let err = TestErr::<SimpleErr, _>::NamedVariantIgnoreRedundant { field: 0 };

        assert!(err.source().is_none());
    }

    #[test]
    fn unnamed_variant_ignore_redundant() {
        let err = TestErr::<SimpleErr, _>::UnnamedVariantIgnoreRedundant(0, 0);

        assert!(err.source().is_none())
    }
}
