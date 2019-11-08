#[macro_use]
extern crate derive_more;

use std::error::Error as _;

mod derives_for_struct {
    use super::*;

    #[derive(Default, Debug, Display, Error)]
    struct SimpleErr;

    #[test]
    fn unit() {
        assert!(SimpleErr.source().is_none());
    }

    #[test]
    fn named_implicit_no_source() {
        #[derive(Default, Debug, Display, Error)]
        struct TestErr {
            field: i32,
        }

        assert!(TestErr::default().source().is_none());
    }

    #[test]
    fn named_implicit_source() {
        #[derive(Default, Debug, Display, Error)]
        #[display(fmt = "")]
        struct TestErr {
            source: SimpleErr,
            field: i32,
        }

        assert!(TestErr::default().source().is_some());
        assert!(TestErr::default().source().unwrap().is::<SimpleErr>());
    }

    #[test]
    fn named_explicit_no_source() {
        #[derive(Default, Debug, Display, Error)]
        #[display(fmt = "")]
        struct TestErr {
            #[error(not(source))]
            source: SimpleErr,
            field: i32,
        }

        assert!(TestErr::default().source().is_none());
    }

    #[test]
    fn named_explicit_source() {
        #[derive(Default, Debug, Display, Error)]
        #[display(fmt = "")]
        struct TestErr {
            #[error(source)]
            explicit_source: SimpleErr,
            field: i32,
        }

        assert!(TestErr::default().source().is_some());
        assert!(TestErr::default().source().unwrap().is::<SimpleErr>());
    }

    #[test]
    fn named_explicit_no_source_redundant() {
        #[derive(Default, Debug, Display, Error)]
        struct TestErr {
            #[error(not(source))]
            field: i32,
        }

        assert!(TestErr::default().source().is_none());
    }

    #[test]
    fn named_explicit_source_redundant() {
        #[derive(Default, Debug, Display, Error)]
        #[display(fmt = "")]
        struct TestErr {
            #[error(source)]
            source: SimpleErr,
            field: i32,
        }

        assert!(TestErr::default().source().is_some());
        assert!(TestErr::default().source().unwrap().is::<SimpleErr>());
    }

    #[test]
    fn named_explicit_suppresses_implicit() {
        #[derive(Default, Debug, Display, Error)]
        #[display(fmt = "")]
        struct TestErr {
            source: i32,
            #[error(source)]
            field: SimpleErr,
        }

        assert!(TestErr::default().source().is_some());
        assert!(TestErr::default().source().unwrap().is::<SimpleErr>());
    }

    #[test]
    fn unnamed_implicit_no_source() {
        #[derive(Default, Debug, Display, Error)]
        #[display(fmt = "")]
        struct TestErr(i32, i32);

        assert!(TestErr::default().source().is_none());
    }

    #[test]
    fn unnamed_implicit_source() {
        #[derive(Default, Debug, Display, Error)]
        struct TestErr(SimpleErr);

        assert!(TestErr::default().source().is_some());
        assert!(TestErr::default().source().unwrap().is::<SimpleErr>());
    }

    #[test]
    fn unnamed_explicit_no_source() {
        #[derive(Default, Debug, Display, Error)]
        struct TestErr(#[error(not(source))] SimpleErr);

        assert!(TestErr::default().source().is_none());
    }

    #[test]
    fn unnamed_explicit_source() {
        #[derive(Default, Debug, Display, Error)]
        #[display(fmt = "")]
        struct TestErr(#[error(source)] SimpleErr, i32);

        assert!(TestErr::default().source().is_some());
        assert!(TestErr::default().source().unwrap().is::<SimpleErr>());
    }

    #[test]
    fn unnamed_explicit_no_source_redundant() {
        #[derive(Default, Debug, Display, Error)]
        #[display(fmt = "")]
        struct TestErr(#[error(not(source))] i32, #[error(not(source))] i32);

        assert!(TestErr::default().source().is_none());
    }

    #[test]
    fn unnamed_explicit_source_redundant() {
        #[derive(Default, Debug, Display, Error)]
        struct TestErr(#[error(source)] SimpleErr);

        assert!(TestErr::default().source().is_some());
        assert!(TestErr::default().source().unwrap().is::<SimpleErr>());
    }
}

mod derives_for_enum {
    use super::*;

    #[derive(Default, Debug, Display, Error)]
    struct SimpleErr;

    #[derive(Debug, Display, Error)]
    #[display(fmt = "")]
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
    }

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
}

mod derives_for_generic_struct {
    use super::*;

    #[derive(Default, Debug, Display, Error)]
    struct SimpleErr;

    #[test]
    fn named_implicit_no_source() {
        #[derive(Default, Debug, Display, Error)]
        struct TestErr<T> {
            field: T,
        }

        assert!(TestErr::<i32>::default().source().is_none());
    }

    #[test]
    fn named_implicit_source() {
        #[derive(Default, Debug, Display, Error)]
        #[display(fmt = "")]
        struct TestErr<E, T> {
            source: E,
            field: T,
        }

        let err = TestErr::<SimpleErr, i32>::default();

        assert!(err.source().is_some());
        assert!(err.source().unwrap().is::<SimpleErr>());
    }

    #[test]
    fn named_explicit_no_source() {
        #[derive(Default, Debug, Display, Error)]
        #[display(fmt = "")]
        struct TestErr<E, T> {
            #[error(not(source))]
            source: E,
            field: T,
        }

        let err = TestErr::<SimpleErr, i32>::default();

        assert!(err.source().is_none());
    }

    #[test]
    fn named_explicit_source() {
        #[derive(Default, Debug, Display, Error)]
        #[display(fmt = "")]
        struct TestErr<E, T> {
            #[error(source)]
            explicit_source: E,
            field: T,
        }

        let err = TestErr::<SimpleErr, i32>::default();

        assert!(err.source().is_some());
        assert!(err.source().unwrap().is::<SimpleErr>());
    }

    #[test]
    fn named_explicit_no_source_redundant() {
        #[derive(Default, Debug, Display, Error)]
        struct TestErr<T> {
            #[error(not(source))]
            field: T,
        }

        assert!(TestErr::<i32>::default().source().is_none());
    }

    #[test]
    fn named_explicit_source_redundant() {
        #[derive(Default, Debug, Display, Error)]
        #[display(fmt = "")]
        struct TestErr<E, T> {
            #[error(source)]
            source: E,
            field: T,
        }

        let err = TestErr::<SimpleErr, i32>::default();

        assert!(err.source().is_some());
        assert!(err.source().unwrap().is::<SimpleErr>());
    }

    #[test]
    fn named_explicit_suppresses_implicit() {
        #[derive(Default, Debug, Display, Error)]
        #[display(fmt = "")]
        struct TestErr<E, T> {
            source: E,
            #[error(source)]
            field: T,
        }

        let err = TestErr::<i32, SimpleErr>::default();

        assert!(err.source().is_some());
        assert!(err.source().unwrap().is::<SimpleErr>());
    }

    #[test]
    fn unnamed_implicit_no_source() {
        #[derive(Default, Debug, Display, Error)]
        #[display(fmt = "")]
        struct TestErr<T>(T, T);

        assert!(TestErr::<i32>::default().source().is_none());
    }

    #[test]
    fn unnamed_implicit_source() {
        #[derive(Default, Debug, Display, Error)]
        struct TestErr<E>(E);

        let err = TestErr::<SimpleErr>::default();

        assert!(err.source().is_some());
        assert!(err.source().unwrap().is::<SimpleErr>());
    }

    #[test]
    fn unnamed_explicit_no_source() {
        #[derive(Default, Debug, Display, Error)]
        struct TestErr<E>(#[error(not(source))] E);

        assert!(TestErr::<SimpleErr>::default().source().is_none());
    }

    #[test]
    fn unnamed_explicit_source() {
        #[derive(Default, Debug, Display, Error)]
        #[display(fmt = "")]
        struct TestErr<E, T>(#[error(source)] E, T);

        let err = TestErr::<SimpleErr, i32>::default();

        assert!(err.source().is_some());
        assert!(err.source().unwrap().is::<SimpleErr>());
    }

    #[test]
    fn unnamed_explicit_no_source_redundant() {
        #[derive(Default, Debug, Display, Error)]
        #[display(fmt = "")]
        struct TestErr<T>(#[error(not(source))] T, #[error(not(source))] T);

        assert!(TestErr::<i32>::default().source().is_none());
    }

    #[test]
    fn unnamed_explicit_source_redundant() {
        #[derive(Default, Debug, Display, Error)]
        struct TestErr<E>(#[error(source)] E);

        let err = TestErr::<SimpleErr>::default();

        assert!(err.source().is_some());
        assert!(err.source().unwrap().is::<SimpleErr>());
    }
}

mod derives_for_generic_enum {
    use super::*;

    #[derive(Default, Debug, Display, Error)]
    struct SimpleErr;

    #[derive(Debug, Display, Error)]
    #[display(fmt = "")]
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
    }

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
}
