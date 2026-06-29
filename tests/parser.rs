use crome::driver;
use std::fs;

#[test]
#[cfg(feature = "parser")]
fn test_parser_float_valid() {
    let content = fs::read_to_string("./tests/source/float_valid.c").unwrap();
    driver::compiler::compiler(&content).unwrap();
}
