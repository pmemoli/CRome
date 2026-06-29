use crome::driver;
use std::fs;

#[test]
#[cfg(all(feature = "codegen", not(feature = "emission")))]
fn test_codegen_float_valid() {
    let content = fs::read_to_string("./tests/source/float_valid.c").unwrap();
    driver::compiler::compiler(&content).unwrap();
}
