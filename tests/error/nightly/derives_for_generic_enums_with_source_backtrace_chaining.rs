use super::*;

derive_display!(TestErr, E, T);
#[derive(Debug, Error)]
enum TestErr<E, T> {
    NamedNoBacktraceNoSource {
        field: T,
    },
    NamedNoBacktraceSourceWithoutBacktrace {
        source: E,
        field: T,
    },
    NamedNoBacktraceSourceWithBacktraceExplicitlyDisabled {
        #[error(not(backtrace))]
        source: E,
        field: T,
    },
    NamedNoBacktraceSourceWithBacktrace {
        source: E,
        field: T,
    },
    NamedBacktraceNoSource {
        backtrace: Backtrace,
        field: T,
    },
    NamedBacktraceSourceWithoutBacktrace {
        source: E,
        backtrace: Backtrace,
        field: T,
    },
    NamedBacktraceSourceWithBacktraceExplicitlyDisabled {
        #[error(not(backtrace))]
        source: E,
        backtrace: Backtrace,
        field: T,
    },
    NamedBacktraceSourceWithBacktrace {
        source: E,
        backtrace: Backtrace,
        field: T,
    },
    UnnamedNoBacktraceNoSource(T, T),
    UnnamedNoBacktraceSourceWithoutBacktrace(#[error(source)] E, T),
    UnnamedNoBacktraceSourceWithBacktraceExplicitlyDisabled(
        #[error(source, not(backtrace))] E,
        T,
    ),
    UnnamedNoBacktraceSourceWithBacktrace(#[error(source)] E, T),
    UnnamedBacktraceNoSource(Backtrace, T, T),
    UnnamedBacktraceSourceWithoutBacktrace(#[error(source)] E, Backtrace, T),
    UnnamedBacktraceSourceWithBacktraceExplictilyDisabled(
        #[error(source, not(backtrace))] E,
        Backtrace,
        T,
    ),
    UnnamedBacktraceSourceWithBacktrace(#[error(source)] E, Backtrace, T),
    UnnamedBacktraceImplicitSourceWithoutBacktrace(E, Backtrace),
    UnnamedBacktraceImplicitSourceWithBacktraceExplicitlyDisabled(
        #[error(not(backtrace))] E,
        Backtrace,
    ),
    UnnamedBacktraceImplicitSourceWithBacktrace(E, Backtrace),
}

impl<E, T> TestErr<E, T> {
    fn get_stored_backtrace(&self) -> &Backtrace {
        match self {
            Self::NamedBacktraceSourceWithoutBacktrace { backtrace, .. } => backtrace,
            Self::NamedBacktraceSourceWithBacktraceExplicitlyDisabled {
                backtrace,
                ..
            } => backtrace,
            Self::NamedBacktraceSourceWithBacktrace { backtrace, .. } => backtrace,
            Self::UnnamedBacktraceSourceWithoutBacktrace(_, backtrace, _) => backtrace,
            Self::UnnamedBacktraceSourceWithBacktraceExplictilyDisabled(
                _,
                backtrace,
                _,
            ) => backtrace,
            Self::UnnamedBacktraceSourceWithBacktrace(_, backtrace, _) => backtrace,
            Self::UnnamedBacktraceImplicitSourceWithoutBacktrace(_, backtrace) => {
                backtrace
            }
            Self::UnnamedBacktraceImplicitSourceWithBacktraceExplicitlyDisabled(
                _,
                backtrace,
            ) => backtrace,
            Self::UnnamedBacktraceImplicitSourceWithBacktrace(_, backtrace) => {
                backtrace
            }
            _ => panic!("ERROR IN TEST IMPLEMENTATION"),
        }
    }
}

#[test]
fn named_no_backtrace_no_source() {
    let err = TestErr::<BacktraceErr, _>::NamedNoBacktraceNoSource { field: 0 };

    assert!(err.backtrace().is_none())
}

#[test]
fn named_no_backtrace_source_without_backtrace() {
    let err = TestErr::NamedNoBacktraceSourceWithoutBacktrace {
        source: SimpleErr,
        field: 0,
    };

    assert!(err.backtrace().is_none())
}

#[test]
fn named_no_backtrace_source_with_backtrace_explicitly_disabled() {
    let err = TestErr::NamedNoBacktraceSourceWithBacktraceExplicitlyDisabled {
        source: BacktraceErr::default(),
        field: 0,
    };

    assert!(err.backtrace().is_none());
}

#[test]
fn named_no_backtrace_source_with_backtrace() {
    let err = TestErr::NamedNoBacktraceSourceWithBacktrace {
        source: BacktraceErr::default(),
        field: 0,
    };

    assert!(err.backtrace().is_some());
}

#[test]
fn named_backtrace_no_source() {
    let err = TestErr::<BacktraceErr, _>::NamedBacktraceNoSource {
        backtrace: Backtrace::force_capture(),
        field: 0,
    };

    assert!(err.backtrace().is_some());
}

#[test]
fn named_backtrace_source_without_backtrace() {
    let err = TestErr::NamedBacktraceSourceWithoutBacktrace {
        source: SimpleErr,
        backtrace: Backtrace::force_capture(),
        field: 0,
    };

    assert!(err.backtrace().is_some());
    assert_bt!(==, err, .get_stored_backtrace);
}

#[test]
fn named_backtrace_source_with_backtrace_explicitly_disabled() {
    let err = TestErr::NamedBacktraceSourceWithBacktraceExplicitlyDisabled {
        source: BacktraceErr::default(),
        backtrace: Backtrace::force_capture(),
        field: 0,
    };

    assert!(err.backtrace().is_some());
    assert_bt!(==, err, .get_stored_backtrace);
}

#[test]
fn named_backtrace_source_with_backtrace() {
    let err = TestErr::NamedBacktraceSourceWithBacktrace {
        source: BacktraceErr::default(),
        backtrace: Backtrace::force_capture(),
        field: 0,
    };

    assert!(err.backtrace().is_some());
    assert_bt!(!=, err, .get_stored_backtrace);
}

#[test]
fn unnamed_no_backtrace_no_source() {
    let err = TestErr::<BacktraceErr, _>::UnnamedNoBacktraceNoSource(0, 0);

    assert!(err.backtrace().is_none())
}

#[test]
fn unnamed_no_backtrace_source_without_backtrace() {
    let err = TestErr::UnnamedNoBacktraceSourceWithoutBacktrace(SimpleErr, 0);

    assert!(err.backtrace().is_none())
}

#[test]
fn unnamed_no_backtrace_source_with_backtrace_explicitly_disabled() {
    let err = TestErr::UnnamedNoBacktraceSourceWithBacktraceExplicitlyDisabled(
        BacktraceErr::default(),
        0,
    );

    assert!(err.backtrace().is_none());
}

#[test]
fn unnamed_no_backtrace_source_with_backtrace() {
    let err =
        TestErr::UnnamedNoBacktraceSourceWithBacktrace(BacktraceErr::default(), 0);

    assert!(err.backtrace().is_some());
}

#[test]
fn unnamed_backtrace_no_source() {
    let err = TestErr::<BacktraceErr, _>::UnnamedBacktraceNoSource(
        Backtrace::force_capture(),
        0,
        0,
    );

    assert!(err.backtrace().is_some());
}

#[test]
fn unnamed_backtrace_source_without_backtrace() {
    let err = TestErr::UnnamedBacktraceSourceWithoutBacktrace(
        SimpleErr,
        Backtrace::force_capture(),
        0,
    );

    assert!(err.backtrace().is_some());
    assert_bt!(==, err, .get_stored_backtrace);
}

#[test]
fn unnamed_backtrace_source_with_backtrace_explictily_disabled() {
    let err = TestErr::UnnamedBacktraceSourceWithBacktraceExplictilyDisabled(
        BacktraceErr::default(),
        Backtrace::force_capture(),
        0,
    );

    assert!(err.backtrace().is_some());
    assert_bt!(==, err, .get_stored_backtrace);
}

#[test]
fn unnamed_backtrace_source_with_backtrace() {
    let err = TestErr::UnnamedBacktraceSourceWithBacktrace(
        BacktraceErr::default(),
        Backtrace::force_capture(),
        0,
    );

    assert!(err.backtrace().is_some());
    assert_bt!(!=, err, .get_stored_backtrace);
}

#[test]
fn unnamed_backtrace_implicit_source_without_backtrace() {
    let err = TestErr::<_, i32>::UnnamedBacktraceImplicitSourceWithoutBacktrace(
        SimpleErr,
        Backtrace::force_capture(),
    );

    assert!(err.backtrace().is_some());
    assert_bt!(==, err, .get_stored_backtrace);
}

#[test]
fn unnamed_backtrace_implicit_source_with_backtrace_explicitly_disabled() {
    let err = TestErr::<_, i32>::UnnamedBacktraceImplicitSourceWithBacktraceExplicitlyDisabled(
        BacktraceErr::default(),
        Backtrace::force_capture(),
    );

    assert!(err.backtrace().is_some());
    assert_bt!(==, err, .get_stored_backtrace);
}

#[test]
fn unnamed_backtrace_implicit_source_with_backtrace() {
    let err = TestErr::<_, i32>::UnnamedBacktraceImplicitSourceWithBacktrace(
        BacktraceErr::default(),
        Backtrace::force_capture(),
    );

    assert!(err.backtrace().is_some());
    assert_bt!(!=, err, .get_stored_backtrace);
}
