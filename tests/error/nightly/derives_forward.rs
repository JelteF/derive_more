use core::error::{request_ref, request_value};

use super::*;

derive_display!(Inner);
#[derive(Debug, Error)]
struct Inner {
    #[error(backtrace)]
    source: BacktraceErr,
}

derive_display!(StructAttr);
#[derive(Debug, Error)]
#[error(forward)]
struct StructAttr {
    #[error(backtrace)]
    source: Inner,
}

impl StructAttr {
    fn get_source_backtrace(&self) -> &Backtrace {
        request_ref(&self.source.source).unwrap()
    }
}

#[test]
fn struct_attr() {
    let err = StructAttr {
        source: Inner {
            source: BacktraceErr {
                backtrace: Backtrace::force_capture(),
            },
        },
    };

    assert!(err.source().is_some());
    assert!(err.source().unwrap().is::<BacktraceErr>());
    assert!(request_ref::<Backtrace>(&err).is_some());
    assert_eq!(request_value::<i32>(&err), Some(42));
    assert_bt!(==, err, .get_source_backtrace);
}

derive_display!(EnumAttr);
#[derive(Debug, Error)]
#[error(forward)]
enum EnumAttr {
    A {
        #[error(backtrace)]
        source: Inner,
    },
    B {
        #[error(source, backtrace)]
        explicit_source: Inner,
    },
}

impl EnumAttr {
    fn get_source_backtrace(&self) -> &Backtrace {
        request_ref(match self {
            Self::A { source } => &source.source,
            Self::B { explicit_source } => &explicit_source.source,
        })
        .unwrap()
    }
}

#[test]
fn enum_attr() {
    let err_a = EnumAttr::A {
        source: Inner {
            source: BacktraceErr {
                backtrace: Backtrace::force_capture(),
            },
        },
    };
    let err_b = EnumAttr::B {
        explicit_source: Inner {
            source: BacktraceErr {
                backtrace: Backtrace::force_capture(),
            },
        },
    };

    assert!(err_a.source().is_some());
    assert!(err_a.source().unwrap().is::<BacktraceErr>());
    assert!(request_ref::<Backtrace>(&err_a).is_some());
    assert_eq!(request_value::<i32>(&err_a), Some(42));
    assert_bt!(==, err_a, .get_source_backtrace);

    assert!(err_b.source().is_some());
    assert!(err_b.source().unwrap().is::<BacktraceErr>());
    assert!(request_ref::<Backtrace>(&err_b).is_some());
    assert_eq!(request_value::<i32>(&err_b), Some(42));
    assert_bt!(==, err_b, .get_source_backtrace);
}

derive_display!(VariantAttr);
#[derive(Debug, Error)]
enum VariantAttr {
    #[error(forward)]
    A {
        #[error(backtrace)]
        source: Inner,
    },
    B {
        #[error(backtrace)]
        source: Inner,
    },
}

impl VariantAttr {
    fn get_source_backtrace(&self) -> &Backtrace {
        request_ref(match self {
            Self::A { source } => &source.source,
            Self::B { source } => &source.source,
        })
        .unwrap()
    }
}

#[test]
fn variant_attr() {
    let err_a = VariantAttr::A {
        source: Inner {
            source: BacktraceErr {
                backtrace: Backtrace::force_capture(),
            },
        },
    };
    let err_b = VariantAttr::B {
        source: Inner {
            source: BacktraceErr {
                backtrace: Backtrace::force_capture(),
            },
        },
    };

    assert!(err_a.source().is_some());
    assert!(err_a.source().unwrap().is::<BacktraceErr>());
    assert!(request_ref::<Backtrace>(&err_a).is_some());
    assert_eq!(request_value::<i32>(&err_a), Some(42));
    assert_bt!(==, err_a, .get_source_backtrace);

    assert!(err_b.source().is_some());
    assert!(err_b.source().unwrap().is::<Inner>());
    assert!(request_ref::<Backtrace>(&err_b).is_some());
    assert_eq!(request_value::<i32>(&err_b), Some(42));
    assert_bt!(==, err_b, .get_source_backtrace);
}
