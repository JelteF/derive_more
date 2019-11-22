use super::*;

#[test]
fn named_no_backtrace_no_source() {
    derive_display!(TestErr, T);
    #[derive(Default, Debug, Error)]
    struct TestErr<T> {
        field: T,
    }

    assert!(TestErr::<i32>::default().backtrace().is_none())
}

#[test]
fn named_no_backtrace_source_without_backtrace() {
    derive_display!(TestErr, E, T);
    #[derive(Default, Debug, Error)]
    struct TestErr<E, T> {
        source: E,
        field: T,
    }

    assert!(TestErr::<SimpleErr, i32>::default().backtrace().is_none())
}

#[test]
fn named_no_backtrace_source_with_backtrace_explicitly_disabled() {
    derive_display!(TestErr, E, T);
    #[derive(Default, Debug, Error)]
    struct TestErr<E, T> {
        #[error(not(backtrace))]
        source: E,
        field: T,
    }

    assert!(TestErr::<BacktraceErr, i32>::default()
        .backtrace()
        .is_none());
}

#[test]
fn named_no_backtrace_source_with_backtrace() {
    derive_display!(TestErr, E, T);
    #[derive(Default, Debug, Error)]
    struct TestErr<E, T> {
        source: E,
        field: T,
    }

    assert!(TestErr::<BacktraceErr, i32>::default()
        .backtrace()
        .is_some());
}

#[test]
fn named_backtrace_no_source() {
    derive_display!(TestErr, T);
    #[derive(Debug, Error)]
    struct TestErr<T> {
        backtrace: Backtrace,
        field: T,
    }

    assert!(TestErr {
        backtrace: Backtrace::force_capture(),
        field: 0
    }
    .backtrace()
    .is_some());
}

#[test]
fn named_backtrace_source_without_backtrace() {
    derive_display!(TestErr, E, T);
    #[derive(Debug, Error)]
    struct TestErr<E, T> {
        source: E,
        backtrace: Backtrace,
        field: T,
    }

    let err = TestErr {
        source: SimpleErr,
        backtrace: Backtrace::force_capture(),
        field: 0,
    };
    assert!(err.backtrace().is_some());
    assert_bt!(==, err);
}

#[test]
fn named_backtrace_source_with_backtrace_explicitly_disabled() {
    derive_display!(TestErr, E, T);
    #[derive(Debug, Error)]
    struct TestErr<E, T> {
        #[error(not(backtrace))]
        source: E,
        backtrace: Backtrace,
        field: T,
    }

    let err = TestErr {
        source: BacktraceErr::default(),
        backtrace: Backtrace::force_capture(),
        field: 0,
    };
    assert!(err.backtrace().is_some());
    assert_bt!(==, err);
}

#[test]
fn named_backtrace_source_with_backtrace() {
    derive_display!(TestErr, E, T);
    #[derive(Debug, Error)]
    struct TestErr<E, T> {
        source: E,
        backtrace: Backtrace,
        field: T,
    }

    let err = TestErr {
        source: BacktraceErr::default(),
        backtrace: Backtrace::force_capture(),
        field: 0,
    };
    assert!(err.backtrace().is_some());
    assert_bt!(!=, err);
}

#[test]
fn unnamed_no_backtrace_no_source() {
    derive_display!(TestErr, T);
    #[derive(Default, Debug, Error)]
    struct TestErr<T>(T, T);

    assert!(TestErr::<i32>::default().backtrace().is_none())
}

#[test]
fn unnamed_no_backtrace_source_without_backtrace() {
    derive_display!(TestErr, E, T);
    #[derive(Default, Debug, Error)]
    struct TestErr<E, T>(#[error(source)] E, T);

    assert!(TestErr::<SimpleErr, i32>::default().backtrace().is_none())
}

#[test]
fn unnamed_no_backtrace_source_with_backtrace_explicitly_disabled() {
    derive_display!(TestErr, E, T);
    #[derive(Default, Debug, Error)]
    struct TestErr<E, T>(#[error(source, not(backtrace))] E, T);

    assert!(TestErr::<BacktraceErr, i32>::default()
        .backtrace()
        .is_none());
}

#[test]
fn unnamed_no_backtrace_source_with_backtrace() {
    derive_display!(TestErr, E, T);
    #[derive(Default, Debug, Error)]
    struct TestErr<E, T>(#[error(source)] E, T);

    assert!(TestErr::<BacktraceErr, i32>::default()
        .backtrace()
        .is_some());
}

#[test]
fn unnamed_backtrace_no_source() {
    derive_display!(TestErr, T);
    #[derive(Debug, Error)]
    struct TestErr<T>(Backtrace, T, T);

    assert!(TestErr(Backtrace::force_capture(), 0, 0)
        .backtrace()
        .is_some());
}

#[test]
fn unnamed_backtrace_source_without_backtrace() {
    derive_display!(TestErr, E, T);
    #[derive(Debug, Error)]
    struct TestErr<E, T>(#[error(source)] E, Backtrace, T);

    let err = TestErr(SimpleErr, Backtrace::force_capture(), 0);
    assert!(err.backtrace().is_some());
    assert_bt!(==, err, 1);
}

#[test]
fn unnamed_backtrace_source_with_backtrace_explictitly_disabled() {
    derive_display!(TestErr, E, T);
    #[derive(Debug, Error)]
    struct TestErr<E, T>(#[error(source, not(backtrace))] E, Backtrace, T);

    let err = TestErr(BacktraceErr::default(), Backtrace::force_capture(), 0);
    assert!(err.backtrace().is_some());
    assert_bt!(==, err, 1);
}

#[test]
fn unnamed_backtrace_source_with_backtrace() {
    derive_display!(TestErr, E, T);
    #[derive(Debug, Error)]
    struct TestErr<E, T>(#[error(source)] E, Backtrace, T);

    let err = TestErr(BacktraceErr::default(), Backtrace::force_capture(), 0);
    assert!(err.backtrace().is_some());
    assert_bt!(!=, err, 1);
}

#[test]
fn unnamed_backtrace_implicit_source_without_backtrace() {
    derive_display!(TestErr, E);
    #[derive(Debug, Error)]
    struct TestErr<E>(E, Backtrace);

    let err = TestErr(SimpleErr, Backtrace::force_capture());
    assert!(err.backtrace().is_some());
    assert_bt!(==, err, 1);
}

#[test]
fn unnamed_backtrace_implicit_source_with_backtrace_explicitly_disabled() {
    derive_display!(TestErr, E);
    #[derive(Debug, Error)]
    struct TestErr<E>(#[error(not(backtrace))] E, Backtrace);

    let err = TestErr(BacktraceErr::default(), Backtrace::force_capture());
    assert!(err.backtrace().is_some());
    assert_bt!(==, err, 1);
}

#[test]
fn unnamed_backtrace_implicit_source_with_backtrace() {
    derive_display!(TestErr, E);
    #[derive(Debug, Error)]
    struct TestErr<E>(E, Backtrace);

    let err = TestErr(BacktraceErr::default(), Backtrace::force_capture());
    assert!(err.backtrace().is_some());
    assert_bt!(!=, err, 1);
}
