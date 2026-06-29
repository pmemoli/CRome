mod driver;

#[test]
#[cfg(feature = "codegen")]
fn test_codegen_float_valid() {
    driver::codegen("./tests/source/float_valid.c").unwrap();
}
