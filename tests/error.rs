#[macro_use]
extern crate derive_more;

use std::error::Error as _;

mod derives_for_struct {
    use super::*;

    #[derive(Default, Debug, Display, Error)]
    struct SS;

    #[test]
    fn unit() {
        assert!(SS.source().is_none());
    }

    #[test]
    fn named_implicit_no_source() {
        #[derive(Default, Debug, Display, Error)]
        struct S {
            field: i32,
        }

        assert!(S::default().source().is_none());
    }

    #[test]
    fn named_implicit_source() {
        #[derive(Default, Debug, Display, Error)]
        #[display(fmt = "")]
        struct S {
            source: SS,
            field: i32,
        }

        assert!(S::default().source().is_some());
    }

    #[test]
    fn named_explicit_no_source() {
        #[derive(Default, Debug, Display, Error)]
        #[display(fmt = "")]
        struct S {
            #[error(not(source))]
            source: SS,
            field: i32,
        }

        assert!(S::default().source().is_none());
    }

    #[test]
    fn named_explicit_source() {
        #[derive(Default, Debug, Display, Error)]
        #[display(fmt = "")]
        struct S {
            #[error(source)]
            explicit_source: SS,
            field: i32,
        }

        assert!(S::default().source().is_some());
    }

    #[test]
    fn named_explicit_no_source_redundant() {
        #[derive(Default, Debug, Display, Error)]
        struct S {
            #[error(not(source))]
            field: i32,
        }

        assert!(S::default().source().is_none());
    }

    #[test]
    fn named_explicit_source_redundant() {
        #[derive(Default, Debug, Display, Error)]
        #[display(fmt = "")]
        struct S {
            #[error(source)]
            source: SS,
            field: i32,
        }

        assert!(S::default().source().is_some());
    }

    #[test]
    fn unnamed_implicit_no_source() {
        #[derive(Default, Debug, Display, Error)]
        #[display(fmt = "")]
        struct S(i32, i32);

        assert!(S::default().source().is_none());
    }

    #[test]
    fn unnamed_implicit_source() {
        #[derive(Default, Debug, Display, Error)]
        struct S(SS);

        assert!(S::default().source().is_some());
    }

    #[test]
    fn unnamed_explicit_no_source() {
        #[derive(Default, Debug, Display, Error)]
        struct S(#[error(not(source))] SS);

        assert!(S::default().source().is_none());
    }

    #[test]
    fn unnamed_explicit_source() {
        #[derive(Default, Debug, Display, Error)]
        #[display(fmt = "")]
        struct S(#[error(source)] SS, i32);

        assert!(S::default().source().is_some());
    }

    #[test]
    fn unnamed_explicit_no_source_redundant() {
        #[derive(Default, Debug, Display, Error)]
        #[display(fmt = "")]
        struct S(#[error(not(source))] i32, #[error(not(source))] i32);

        assert!(S::default().source().is_none());
    }

    #[test]
    fn unnamed_explicit_source_redundant() {
        #[derive(Default, Debug, Display, Error)]
        struct S(#[error(source)] SS);

        assert!(S::default().source().is_some());
    }
}

mod derives_for_enum {
    use super::*;

    #[derive(Default, Debug, Display, Error)]
    struct S;

    #[derive(Debug, Display, Error)]
    #[display(fmt = "")]
    enum E {
        Unit,
        NamedImplicitNoSource {
            field: i32,
        },
        NamedImplicitSource {
            source: S,
            field: i32,
        },
        NamedExplicitNoSource {
            #[error(not(source))]
            source: S,
            field: i32,
        },
        NamedExplicitSource {
            #[error(source)]
            explicit_source: S,
            field: i32,
        },
        NamedExplicitNoSourceRedundant {
            #[error(not(source))]
            field: i32,
        },
        NamedExplicitSourceRedundant {
            #[error(source)]
            source: S,
            field: i32,
        },
        UnnamedImplicitNoSource(i32, i32),
        UnnamedImplicitSource(S),
        UnnamedExplicitNoSource(#[error(not(source))] S),
        UnnamedExplicitSource(#[error(source)] S, i32),
        UnnamedExplicitNoSourceRedundant(
            #[error(not(source))] i32,
            #[error(not(source))] i32,
        ),
        UnnamedExplicitSourceRedundant(#[error(source)] S),
    }

    #[test]
    fn unit() {
        assert!(E::Unit.source().is_none());
    }

    #[test]
    fn named_implicit_no_source() {
        assert!(E::NamedImplicitNoSource { field: 0 }.source().is_none());
    }

    #[test]
    fn named_implicit_source() {
        assert!(E::NamedImplicitSource {
            source: S::default(),
            field: 0
        }
        .source()
        .is_some());
    }

    #[test]
    fn named_explicit_no_source() {
        assert!(E::NamedExplicitNoSource {
            source: S::default(),
            field: 0
        }
        .source()
        .is_none());
    }

    #[test]
    fn named_explicit_source() {
        assert!(E::NamedExplicitSource {
            explicit_source: S::default(),
            field: 0
        }
        .source()
        .is_some());
    }

    #[test]
    fn named_explicit_no_source_redundant() {
        assert!(E::NamedExplicitNoSourceRedundant { field: 0 }
            .source()
            .is_none());
    }

    #[test]
    fn named_explicit_source_redundant() {
        assert!(E::NamedExplicitSourceRedundant {
            source: S::default(),
            field: 0
        }
        .source()
        .is_some());
    }

    #[test]
    fn unnamed_implicit_no_source() {
        assert!(E::UnnamedImplicitNoSource(0, 0).source().is_none());
    }

    #[test]
    fn unnamed_implicit_source() {
        assert!(E::UnnamedImplicitSource(S::default()).source().is_some());
    }

    #[test]
    fn unnamed_explicit_no_source() {
        assert!(E::UnnamedExplicitNoSource(S::default()).source().is_none());
    }

    #[test]
    fn unnamed_explicit_source() {
        assert!(E::UnnamedExplicitSource(S::default(), 0).source().is_some());
    }

    #[test]
    fn unnamed_explicit_no_source_redundant() {
        assert!(E::UnnamedExplicitNoSourceRedundant(0, 0).source().is_none());
    }

    #[test]
    fn unnamed_explicit_source_redundant() {
        assert!(E::UnnamedExplicitSourceRedundant(S::default())
            .source()
            .is_some());
    }
}

mod derives_for_generic_struct {
    use super::*;

    #[derive(Default, Debug, Display, Error)]
    struct SS;

    #[test]
    fn named_implicit_no_source() {
        #[derive(Default, Debug, Display, Error)]
        struct S<T> {
            field: T,
        }

        assert!(S::<i32>::default().source().is_none());
    }

    #[test]
    fn named_implicit_source() {
        #[derive(Default, Debug, Display, Error)]
        #[display(fmt = "")]
        struct S<SS, T> {
            source: SS,
            field: T,
        }

        assert!(S::<SS, i32>::default().source().is_some());
    }

    #[test]
    fn named_explicit_no_source() {
        #[derive(Default, Debug, Display, Error)]
        #[display(fmt = "")]
        struct S<SS, T> {
            #[error(not(source))]
            source: SS,
            field: T,
        }

        assert!(S::<SS, i32>::default().source().is_none());
    }

    #[test]
    fn named_explicit_source() {
        #[derive(Default, Debug, Display, Error)]
        #[display(fmt = "")]
        struct S<SS, T> {
            #[error(source)]
            explicit_source: SS,
            field: T,
        }

        assert!(S::<SS, i32>::default().source().is_some());
    }

    #[test]
    fn named_explicit_no_source_redundant() {
        #[derive(Default, Debug, Display, Error)]
        struct S<T> {
            #[error(not(source))]
            field: T,
        }

        assert!(S::<i32>::default().source().is_none());
    }

    #[test]
    fn named_explicit_source_redundant() {
        #[derive(Default, Debug, Display, Error)]
        #[display(fmt = "")]
        struct S<SS, T> {
            #[error(source)]
            source: SS,
            field: T,
        }

        assert!(S::<SS, i32>::default().source().is_some());
    }

    #[test]
    fn unnamed_implicit_no_source() {
        #[derive(Default, Debug, Display, Error)]
        #[display(fmt = "")]
        struct S<T>(T, T);

        assert!(S::<i32>::default().source().is_none());
    }

    #[test]
    fn unnamed_implicit_source() {
        #[derive(Default, Debug, Display, Error)]
        struct S<SS>(SS);

        assert!(S::<SS>::default().source().is_some());
    }

    #[test]
    fn unnamed_explicit_no_source() {
        #[derive(Default, Debug, Display, Error)]
        struct S<SS>(#[error(not(source))] SS);

        assert!(S::<SS>::default().source().is_none());
    }

    #[test]
    fn unnamed_explicit_source() {
        #[derive(Default, Debug, Display, Error)]
        #[display(fmt = "")]
        struct S<SS, T>(#[error(source)] SS, T);

        assert!(S::<SS, i32>::default().source().is_some());
    }

    #[test]
    fn unnamed_explicit_no_source_redundant() {
        #[derive(Default, Debug, Display, Error)]
        #[display(fmt = "")]
        struct S<T>(#[error(not(source))] T, #[error(not(source))] T);

        assert!(S::<i32>::default().source().is_none());
    }

    #[test]
    fn unnamed_explicit_source_redundant() {
        #[derive(Default, Debug, Display, Error)]
        struct S<SS>(#[error(source)] SS);

        assert!(S::<SS>::default().source().is_some());
    }
}

mod derives_for_generic_enum {
    use super::*;

    #[derive(Default, Debug, Display, Error)]
    struct S;

    #[derive(Debug, Display, Error)]
    #[display(fmt = "")]
    enum E<S, T> {
        Unit,
        NamedImplicitNoSource {
            field: T,
        },
        NamedImplicitSource {
            source: S,
            field: T,
        },
        NamedExplicitNoSource {
            #[error(not(source))]
            source: S,
            field: T,
        },
        NamedExplicitSource {
            #[error(source)]
            explicit_source: S,
            field: T,
        },
        NamedExplicitNoSourceRedundant {
            #[error(not(source))]
            field: T,
        },
        NamedExplicitSourceRedundant {
            #[error(source)]
            source: S,
            field: T,
        },
        UnnamedImplicitNoSource(T, T),
        UnnamedImplicitSource(S),
        UnnamedExplicitNoSource(#[error(not(source))] S),
        UnnamedExplicitSource(#[error(source)] S, T),
        UnnamedExplicitNoSourceRedundant(
            #[error(not(source))] T,
            #[error(not(source))] T,
        ),
        UnnamedExplicitSourceRedundant(#[error(source)] S),
    }

    #[test]
    fn unit() {
        assert!(E::<S, i32>::Unit.source().is_none());
    }

    #[test]
    fn named_implicit_no_source() {
        assert!(E::<S, _>::NamedImplicitNoSource { field: 0 }
            .source()
            .is_none());
    }

    #[test]
    fn named_implicit_source() {
        assert!(E::NamedImplicitSource {
            source: S::default(),
            field: 0
        }
        .source()
        .is_some());
    }

    #[test]
    fn named_explicit_no_source() {
        assert!(E::NamedExplicitNoSource {
            source: S::default(),
            field: 0
        }
        .source()
        .is_none());
    }

    #[test]
    fn named_explicit_source() {
        assert!(E::NamedExplicitSource {
            explicit_source: S::default(),
            field: 0
        }
        .source()
        .is_some());
    }

    #[test]
    fn named_explicit_no_source_redundant() {
        assert!(E::<S, _>::NamedExplicitNoSourceRedundant { field: 0 }
            .source()
            .is_none());
    }

    #[test]
    fn named_explicit_source_redundant() {
        assert!(E::NamedExplicitSourceRedundant {
            source: S::default(),
            field: 0
        }
        .source()
        .is_some());
    }

    #[test]
    fn unnamed_implicit_no_source() {
        assert!(E::<S, _>::UnnamedImplicitNoSource(0, 0).source().is_none());
    }

    #[test]
    fn unnamed_implicit_source() {
        assert!(E::<_, i32>::UnnamedImplicitSource(S::default())
            .source()
            .is_some());
    }

    #[test]
    fn unnamed_explicit_no_source() {
        assert!(E::<_, i32>::UnnamedExplicitNoSource(S::default())
            .source()
            .is_none());
    }

    #[test]
    fn unnamed_explicit_source() {
        assert!(E::UnnamedExplicitSource(S::default(), 0).source().is_some());
    }

    #[test]
    fn unnamed_explicit_no_source_redundant() {
        assert!(E::<S, _>::UnnamedExplicitNoSourceRedundant(0, 0)
            .source()
            .is_none());
    }

    #[test]
    fn unnamed_explicit_source_redundant() {
        assert!(E::<_, i32>::UnnamedExplicitSourceRedundant(S::default())
            .source()
            .is_some());
    }
}
