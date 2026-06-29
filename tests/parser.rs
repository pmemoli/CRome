mod driver;

#[test]
#[cfg(feature = "parser")]
fn test_parser_float_valid() {
    driver::parser("./tests/source/float_valid.c").unwrap();
}
