use super::*;

derive_display!(Inner);
#[derive(Debug, Error)]
struct Inner {
    source: SimpleErr,
}

derive_display!(StructAttr);
#[derive(Debug, Error)]
#[error(forward)]
struct StructAttr {
    source: Inner,
}

#[test]
fn struct_attr() {
    let err = StructAttr {
        source: Inner { source: SimpleErr },
    };

    assert!(err.source().is_some());
    assert!(err.source().unwrap().is::<SimpleErr>());
}

derive_display!(EnumAttr);
#[derive(Debug, Error)]
#[error(forward)]
enum EnumAttr {
    A { source: Inner },
    B { source: Inner },
}

#[test]
fn enum_attr() {
    let err_a = EnumAttr::A {
        source: Inner { source: SimpleErr },
    };

    let err_b = EnumAttr::B {
        source: Inner { source: SimpleErr },
    };

    assert!(err_a.source().is_some());
    assert!(err_a.source().unwrap().is::<SimpleErr>());

    assert!(err_b.source().is_some());
    assert!(err_b.source().unwrap().is::<SimpleErr>());
}

derive_display!(VariantAttr);
#[derive(Debug, Error)]
enum VariantAttr {
    #[error(forward)]
    A {
        source: Inner,
    },
    B {
        source: Inner,
    },
}

#[test]
fn variant_attr() {
    let err_a = VariantAttr::A {
        source: Inner { source: SimpleErr },
    };

    let err_b = VariantAttr::B {
        source: Inner { source: SimpleErr },
    };

    assert!(err_a.source().is_some());
    assert!(err_a.source().unwrap().is::<SimpleErr>());

    assert!(err_b.source().is_some());
    assert!(err_b.source().unwrap().is::<Inner>());
}

derive_display!(FieldAttr);
#[derive(Debug, Error)]
enum FieldAttr {
    A {
        #[error(forward, source)]
        field: Inner,
    },
    B {
        #[error(source)]
        field: Inner,
    },
}

#[test]
fn field_attr() {
    let err_a = FieldAttr::A {
        field: Inner { source: SimpleErr },
    };

    let err_b = FieldAttr::B {
        field: Inner { source: SimpleErr },
    };

    assert!(err_a.source().is_some());
    assert!(err_a.source().unwrap().is::<SimpleErr>());

    assert!(err_b.source().is_some());
    assert!(err_b.source().unwrap().is::<Inner>());
}
