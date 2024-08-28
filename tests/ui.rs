#[test]
fn ui() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/*.rs");
}

#[cfg(feature = "experimental-write")]
#[test]
fn ui_write() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/read-write/*.rs")
}


#[cfg(not(feature = "experimental-write"))]
#[test]
fn ui_read_only() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/read-only/*.rs")
}
