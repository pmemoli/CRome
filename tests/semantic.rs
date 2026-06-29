mod driver;

#[test]
#[cfg(feature = "validate")]
fn test_semantic_float_valid() {
    driver::validate("./tests/source/float_valid.c").unwrap();
}
