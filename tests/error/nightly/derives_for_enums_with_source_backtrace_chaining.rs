use super::*;

derive_display!(TestErr);
#[derive(Debug, Error)]
enum TestErr {
    NamedNoBacktraceNoSource {
        field: i32,
    },
    NamedNoBacktraceSourceWithoutBacktrace {
        source: SimpleErr,
        field: i32,
    },
    NamedNoBacktraceSourceWithBacktraceExplicitlyDisabled {
        #[error(not(backtrace))]
        source: BacktraceErr,
        field: i32,
    },
    NamedNoBacktraceSourceWithBacktrace {
        source: BacktraceErr,
        field: i32,
    },
    NamedBacktraceNoSource {
        backtrace: Backtrace,
        field: i32,
    },
    NamedBacktraceSourceWithoutBacktrace {
        source: SimpleErr,
        backtrace: Backtrace,
        field: i32,
    },
    NamedBacktraceSourceWithBacktraceExplicitlyDisabled {
        #[error(not(backtrace))]
        source: BacktraceErr,
        backtrace: Backtrace,
        field: i32,
    },
    NamedBacktraceSourceWithBacktrace {
        source: BacktraceErr,
        backtrace: Backtrace,
        field: i32,
    },
    UnnamedNoBacktraceNoSource(i32, i32),
    UnnamedNoBacktraceSourceWithoutBacktrace(#[error(source)] SimpleErr, i32),
    UnnamedNoBacktraceSourceWithBacktraceExplicitlyDisabled(
        #[error(source, not(backtrace))] BacktraceErr,
        i32,
    ),
    UnnamedNoBacktraceSourceWithBacktrace(#[error(source)] BacktraceErr, i32),
    UnnamedBacktraceNoSource(Backtrace, i32, i32),
    UnnamedBacktraceSourceWithoutBacktrace(#[error(source)] SimpleErr, Backtrace, i32),
    UnnamedBacktraceSourceWithBacktraceExplictilyDisabled(
        #[error(source, not(backtrace))] BacktraceErr,
        Backtrace,
        i32,
    ),
    UnnamedBacktraceSourceWithBacktrace(#[error(source)] BacktraceErr, Backtrace, i32),
    UnnamedBacktraceImplicitSourceWithoutBacktrace(SimpleErr, Backtrace),
    UnnamedBacktraceImplicitSourceWithBacktraceExplicitlyDisabled(
        #[error(not(backtrace))] BacktraceErr,
        Backtrace,
    ),
    UnnamedBacktraceImplicitSourceWithBacktrace(BacktraceErr, Backtrace),
}

impl TestErr {
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
    let err = TestErr::NamedNoBacktraceNoSource { field: 0 };

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
    let err = TestErr::NamedBacktraceNoSource {
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
    let err = TestErr::UnnamedNoBacktraceNoSource(0, 0);

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
    let err = TestErr::UnnamedBacktraceNoSource(Backtrace::force_capture(), 0, 0);

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
    let err = TestErr::UnnamedBacktraceImplicitSourceWithoutBacktrace(
        SimpleErr,
        Backtrace::force_capture(),
    );

    assert!(err.backtrace().is_some());
    assert_bt!(==, err, .get_stored_backtrace);
}

#[test]
fn unnamed_backtrace_implicit_source_with_backtrace_explicitly_disabled() {
    let err = TestErr::UnnamedBacktraceImplicitSourceWithBacktraceExplicitlyDisabled(
        BacktraceErr::default(),
        Backtrace::force_capture(),
    );

    assert!(err.backtrace().is_some());
    assert_bt!(==, err, .get_stored_backtrace);
}

#[test]
fn unnamed_backtrace_implicit_source_with_backtrace() {
    let err = TestErr::UnnamedBacktraceImplicitSourceWithBacktrace(
        BacktraceErr::default(),
        Backtrace::force_capture(),
    );

    assert!(err.backtrace().is_some());
    assert_bt!(!=, err, .get_stored_backtrace);
}
