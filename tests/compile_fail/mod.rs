#[rustversion::stable]
#[test]
fn compile_fail() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/compile_fail/*/*.rs");
    since_1_41(&t);
}

#[rustversion::since(1.41)]
fn since_1_41(t: &trybuild::TestCases) {
    t.compile_fail("tests/compile_fail/*/since_1_41/*.rs");
}

#[rustversion::before(1.41)]
fn since_1_41(_: &trybuild::TestCases) {}
