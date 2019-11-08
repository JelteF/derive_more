#[macro_use]
extern crate derive_more;

use std::error::Error as _;

mod derives_for_struct {
    use super::*;

    #[derive(Default, Debug, Display, Error)]
    struct SE; // SE - Simple Error

    #[test]
    fn unit() {
        assert!(SE.source().is_none());
    }

    #[test]
    fn named_implicit_no_source() {
        #[derive(Default, Debug, Display, Error)]
        struct Test {
            field: i32,
        }

        assert!(Test::default().source().is_none());
    }

    #[test]
    fn named_implicit_source() {
        #[derive(Default, Debug, Display, Error)]
        #[display(fmt = "")]
        struct Test {
            source: SE,
            field: i32,
        }

        assert!(Test::default().source().is_some());
    }

    #[test]
    fn named_explicit_no_source() {
        #[derive(Default, Debug, Display, Error)]
        #[display(fmt = "")]
        struct Test {
            #[error(not(source))]
            source: SE,
            field: i32,
        }

        assert!(Test::default().source().is_none());
    }

    #[test]
    fn named_explicit_source() {
        #[derive(Default, Debug, Display, Error)]
        #[display(fmt = "")]
        struct Test {
            #[error(source)]
            explicit_source: SE,
            field: i32,
        }

        assert!(Test::default().source().is_some());
    }

    #[test]
    fn named_explicit_no_source_redundant() {
        #[derive(Default, Debug, Display, Error)]
        struct Test {
            #[error(not(source))]
            field: i32,
        }

        assert!(Test::default().source().is_none());
    }

    #[test]
    fn named_explicit_source_redundant() {
        #[derive(Default, Debug, Display, Error)]
        #[display(fmt = "")]
        struct Test {
            #[error(source)]
            source: SE,
            field: i32,
        }

        assert!(Test::default().source().is_some());
    }

    #[test]
    fn unnamed_implicit_no_source() {
        #[derive(Default, Debug, Display, Error)]
        #[display(fmt = "")]
        struct Test(i32, i32);

        assert!(Test::default().source().is_none());
    }

    #[test]
    fn unnamed_implicit_source() {
        #[derive(Default, Debug, Display, Error)]
        struct Test(SE);

        assert!(Test::default().source().is_some());
    }

    #[test]
    fn unnamed_explicit_no_source() {
        #[derive(Default, Debug, Display, Error)]
        struct Test(#[error(not(source))] SE);

        assert!(Test::default().source().is_none());
    }

    #[test]
    fn unnamed_explicit_source() {
        #[derive(Default, Debug, Display, Error)]
        #[display(fmt = "")]
        struct Test(#[error(source)] SE, i32);

        assert!(Test::default().source().is_some());
    }

    #[test]
    fn unnamed_explicit_no_source_redundant() {
        #[derive(Default, Debug, Display, Error)]
        #[display(fmt = "")]
        struct Test(#[error(not(source))] i32, #[error(not(source))] i32);

        assert!(Test::default().source().is_none());
    }

    #[test]
    fn unnamed_explicit_source_redundant() {
        #[derive(Default, Debug, Display, Error)]
        struct Test(#[error(source)] SE);

        assert!(Test::default().source().is_some());
    }
}

mod derives_for_enum {
    use super::*;

    #[derive(Default, Debug, Display, Error)]
    struct SE; // SE - Simple Error

    #[derive(Debug, Display, Error)]
    #[display(fmt = "")]
    enum Test {
        Unit,
        NamedImplicitNoSource {
            field: i32,
        },
        NamedImplicitSource {
            source: SE,
            field: i32,
        },
        NamedExplicitNoSource {
            #[error(not(source))]
            source: SE,
            field: i32,
        },
        NamedExplicitSource {
            #[error(source)]
            explicit_source: SE,
            field: i32,
        },
        NamedExplicitNoSourceRedundant {
            #[error(not(source))]
            field: i32,
        },
        NamedExplicitSourceRedundant {
            #[error(source)]
            source: SE,
            field: i32,
        },
        UnnamedImplicitNoSource(i32, i32),
        UnnamedImplicitSource(SE),
        UnnamedExplicitNoSource(#[error(not(source))] SE),
        UnnamedExplicitSource(#[error(source)] SE, i32),
        UnnamedExplicitNoSourceRedundant(
            #[error(not(source))] i32,
            #[error(not(source))] i32,
        ),
        UnnamedExplicitSourceRedundant(#[error(source)] SE),
    }

    #[test]
    fn unit() {
        assert!(Test::Unit.source().is_none());
    }

    #[test]
    fn named_implicit_no_source() {
        assert!(Test::NamedImplicitNoSource { field: 0 }.source().is_none());
    }

    #[test]
    fn named_implicit_source() {
        assert!(Test::NamedImplicitSource {
            source: SE::default(),
            field: 0
        }
        .source()
        .is_some());
    }

    #[test]
    fn named_explicit_no_source() {
        assert!(Test::NamedExplicitNoSource {
            source: SE::default(),
            field: 0
        }
        .source()
        .is_none());
    }

    #[test]
    fn named_explicit_source() {
        assert!(Test::NamedExplicitSource {
            explicit_source: SE::default(),
            field: 0
        }
        .source()
        .is_some());
    }

    #[test]
    fn named_explicit_no_source_redundant() {
        assert!(Test::NamedExplicitNoSourceRedundant { field: 0 }
            .source()
            .is_none());
    }

    #[test]
    fn named_explicit_source_redundant() {
        assert!(Test::NamedExplicitSourceRedundant {
            source: SE::default(),
            field: 0
        }
        .source()
        .is_some());
    }

    #[test]
    fn unnamed_implicit_no_source() {
        assert!(Test::UnnamedImplicitNoSource(0, 0).source().is_none());
    }

    #[test]
    fn unnamed_implicit_source() {
        assert!(Test::UnnamedImplicitSource(SE::default())
            .source()
            .is_some());
    }

    #[test]
    fn unnamed_explicit_no_source() {
        assert!(Test::UnnamedExplicitNoSource(SE::default())
            .source()
            .is_none());
    }

    #[test]
    fn unnamed_explicit_source() {
        assert!(Test::UnnamedExplicitSource(SE::default(), 0)
            .source()
            .is_some());
    }

    #[test]
    fn unnamed_explicit_no_source_redundant() {
        assert!(Test::UnnamedExplicitNoSourceRedundant(0, 0)
            .source()
            .is_none());
    }

    #[test]
    fn unnamed_explicit_source_redundant() {
        assert!(Test::UnnamedExplicitSourceRedundant(SE::default())
            .source()
            .is_some());
    }
}

mod derives_for_generic_struct {
    use super::*;

    #[derive(Default, Debug, Display, Error)]
    struct SE; // SE - Simple Error

    #[test]
    fn named_implicit_no_source() {
        #[derive(Default, Debug, Display, Error)]
        struct Test<T> {
            field: T,
        }

        assert!(Test::<i32>::default().source().is_none());
    }

    #[test]
    fn named_implicit_source() {
        #[derive(Default, Debug, Display, Error)]
        #[display(fmt = "")]
        struct Test<E, T> {
            source: E,
            field: T,
        }

        assert!(Test::<SE, i32>::default().source().is_some());
    }

    #[test]
    fn named_explicit_no_source() {
        #[derive(Default, Debug, Display, Error)]
        #[display(fmt = "")]
        struct Test<E, T> {
            #[error(not(source))]
            source: E,
            field: T,
        }

        assert!(Test::<SE, i32>::default().source().is_none());
    }

    #[test]
    fn named_explicit_source() {
        #[derive(Default, Debug, Display, Error)]
        #[display(fmt = "")]
        struct Test<E, T> {
            #[error(source)]
            explicit_source: E,
            field: T,
        }

        assert!(Test::<SE, i32>::default().source().is_some());
    }

    #[test]
    fn named_explicit_no_source_redundant() {
        #[derive(Default, Debug, Display, Error)]
        struct Test<T> {
            #[error(not(source))]
            field: T,
        }

        assert!(Test::<i32>::default().source().is_none());
    }

    #[test]
    fn named_explicit_source_redundant() {
        #[derive(Default, Debug, Display, Error)]
        #[display(fmt = "")]
        struct Test<E, T> {
            #[error(source)]
            source: E,
            field: T,
        }

        assert!(Test::<SE, i32>::default().source().is_some());
    }

    #[test]
    fn unnamed_implicit_no_source() {
        #[derive(Default, Debug, Display, Error)]
        #[display(fmt = "")]
        struct Test<T>(T, T);

        assert!(Test::<i32>::default().source().is_none());
    }

    #[test]
    fn unnamed_implicit_source() {
        #[derive(Default, Debug, Display, Error)]
        struct Test<E>(E);

        assert!(Test::<SE>::default().source().is_some());
    }

    #[test]
    fn unnamed_explicit_no_source() {
        #[derive(Default, Debug, Display, Error)]
        struct Test<E>(#[error(not(source))] E);

        assert!(Test::<SE>::default().source().is_none());
    }

    #[test]
    fn unnamed_explicit_source() {
        #[derive(Default, Debug, Display, Error)]
        #[display(fmt = "")]
        struct Test<E, T>(#[error(source)] E, T);

        assert!(Test::<SE, i32>::default().source().is_some());
    }

    #[test]
    fn unnamed_explicit_no_source_redundant() {
        #[derive(Default, Debug, Display, Error)]
        #[display(fmt = "")]
        struct Test<T>(#[error(not(source))] T, #[error(not(source))] T);

        assert!(Test::<i32>::default().source().is_none());
    }

    #[test]
    fn unnamed_explicit_source_redundant() {
        #[derive(Default, Debug, Display, Error)]
        struct Test<E>(#[error(source)] E);

        assert!(Test::<SE>::default().source().is_some());
    }
}

mod derives_for_generic_enum {
    use super::*;

    #[derive(Default, Debug, Display, Error)]
    struct SE; // SE - Simple Error

    #[derive(Debug, Display, Error)]
    #[display(fmt = "")]
    enum Test<E, T> {
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
        assert!(Test::<SE, i32>::Unit.source().is_none());
    }

    #[test]
    fn named_implicit_no_source() {
        assert!(Test::<SE, _>::NamedImplicitNoSource { field: 0 }
            .source()
            .is_none());
    }

    #[test]
    fn named_implicit_source() {
        assert!(Test::NamedImplicitSource {
            source: SE::default(),
            field: 0
        }
        .source()
        .is_some());
    }

    #[test]
    fn named_explicit_no_source() {
        assert!(Test::NamedExplicitNoSource {
            source: SE::default(),
            field: 0
        }
        .source()
        .is_none());
    }

    #[test]
    fn named_explicit_source() {
        assert!(Test::NamedExplicitSource {
            explicit_source: SE::default(),
            field: 0
        }
        .source()
        .is_some());
    }

    #[test]
    fn named_explicit_no_source_redundant() {
        assert!(Test::<SE, _>::NamedExplicitNoSourceRedundant { field: 0 }
            .source()
            .is_none());
    }

    #[test]
    fn named_explicit_source_redundant() {
        assert!(Test::NamedExplicitSourceRedundant {
            source: SE::default(),
            field: 0
        }
        .source()
        .is_some());
    }

    #[test]
    fn unnamed_implicit_no_source() {
        assert!(Test::<SE, _>::UnnamedImplicitNoSource(0, 0)
            .source()
            .is_none());
    }

    #[test]
    fn unnamed_implicit_source() {
        assert!(Test::<_, i32>::UnnamedImplicitSource(SE::default())
            .source()
            .is_some());
    }

    #[test]
    fn unnamed_explicit_no_source() {
        assert!(Test::<_, i32>::UnnamedExplicitNoSource(SE::default())
            .source()
            .is_none());
    }

    #[test]
    fn unnamed_explicit_source() {
        assert!(Test::UnnamedExplicitSource(SE::default(), 0)
            .source()
            .is_some());
    }

    #[test]
    fn unnamed_explicit_no_source_redundant() {
        assert!(Test::<SE, _>::UnnamedExplicitNoSourceRedundant(0, 0)
            .source()
            .is_none());
    }

    #[test]
    fn unnamed_explicit_source_redundant() {
        assert!(
            Test::<_, i32>::UnnamedExplicitSourceRedundant(SE::default())
                .source()
                .is_some()
        );
    }
}
