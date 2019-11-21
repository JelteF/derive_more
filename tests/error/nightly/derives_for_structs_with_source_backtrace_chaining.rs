use super::*;

#[test]
fn named_no_backtrace_no_source() {
    derive_display!(TestErr);
    #[derive(Default, Debug, Error)]
    struct TestErr {
        field: i32,
    }

    assert!(TestErr::default().backtrace().is_none())
}

#[test]
fn named_no_backtrace_source_without_backtrace() {
    derive_display!(TestErr);
    #[derive(Default, Debug, Error)]
    struct TestErr {
        source: SimpleErr,
        field: i32,
    }

    assert!(TestErr::default().backtrace().is_none())
}

#[test]
fn named_no_backtrace_source_with_backtrace_explicitly_disabled() {
    derive_display!(TestErr);
    #[derive(Default, Debug, Error)]
    struct TestErr {
        #[error(not(backtrace))]
        source: BacktraceErr,
        field: i32,
    }

    assert!(TestErr::default().backtrace().is_none());
}

#[test]
fn named_no_backtrace_source_with_backtrace() {
    derive_display!(TestErr);
    #[derive(Default, Debug, Error)]
    struct TestErr {
        source: BacktraceErr,
        field: i32,
    }

    assert!(TestErr::default().backtrace().is_some());
}

#[test]
fn named_backtrace_no_source() {
    derive_display!(TestErr);
    #[derive(Debug, Error)]
    struct TestErr {
        backtrace: Backtrace,
        field: i32,
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
    derive_display!(TestErr);
    #[derive(Debug, Error)]
    struct TestErr {
        source: SimpleErr,
        backtrace: Backtrace,
        field: i32,
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
    derive_display!(TestErr);
    #[derive(Debug, Error)]
    struct TestErr {
        #[error(not(backtrace))]
        source: BacktraceErr,
        backtrace: Backtrace,
        field: i32,
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
    derive_display!(TestErr);
    #[derive(Debug, Error)]
    struct TestErr {
        source: BacktraceErr,
        backtrace: Backtrace,
        field: i32,
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
    derive_display!(TestErr);
    #[derive(Default, Debug, Error)]
    struct TestErr(i32, i32);

    assert!(TestErr::default().backtrace().is_none())
}

#[test]
fn unnamed_no_backtrace_source_without_backtrace() {
    derive_display!(TestErr);
    #[derive(Default, Debug, Error)]
    struct TestErr(#[error(source)] SimpleErr, i32);

    assert!(TestErr::default().backtrace().is_none())
}

#[test]
fn unnamed_no_backtrace_source_with_backtrace_explicitly_disabled() {
    derive_display!(TestErr);
    #[derive(Default, Debug, Error)]
    struct TestErr(#[error(source, not(backtrace))] BacktraceErr, i32);

    assert!(TestErr::default().backtrace().is_none());
}

#[test]
fn unnamed_no_backtrace_source_with_backtrace() {
    derive_display!(TestErr);
    #[derive(Default, Debug, Error)]
    struct TestErr(#[error(source)] BacktraceErr, i32);

    assert!(TestErr::default().backtrace().is_some());
}

#[test]
fn unnamed_backtrace_no_source() {
    derive_display!(TestErr);
    #[derive(Debug, Error)]
    struct TestErr(Backtrace, i32);

    assert!(TestErr(Backtrace::force_capture(), 0).backtrace().is_some());
}

#[test]
fn unnamed_backtrace_source_without_backtrace() {
    derive_display!(TestErr);
    #[derive(Debug, Error)]
    struct TestErr(#[error(source)] SimpleErr, Backtrace, i32);

    let err = TestErr(SimpleErr, Backtrace::force_capture(), 0);
    assert!(err.backtrace().is_some());
    assert_bt!(==, err, 1);
}

#[test]
fn unnamed_backtrace_source_with_backtrace_explictily_disabled() {
    derive_display!(TestErr);
    #[derive(Debug, Error)]
    struct TestErr(
        #[error(source, not(backtrace))] BacktraceErr,
        Backtrace,
        i32,
    );

    let err = TestErr(BacktraceErr::default(), Backtrace::force_capture(), 0);
    assert!(err.backtrace().is_some());
    assert_bt!(==, err, 1);
}

#[test]
fn unnamed_backtrace_source_with_backtrace() {
    derive_display!(TestErr);
    #[derive(Debug, Error)]
    struct TestErr(#[error(source)] BacktraceErr, Backtrace, i32);

    let err = TestErr(BacktraceErr::default(), Backtrace::force_capture(), 0);
    assert!(err.backtrace().is_some());
    assert_bt!(!=, err, 1);
}
