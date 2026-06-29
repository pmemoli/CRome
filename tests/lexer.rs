mod driver;

#[test]
#[cfg(feature = "lex")]
fn test_lexer_valid() {
    driver::lexer("./tests/source/float_valid.c").unwrap();
}
