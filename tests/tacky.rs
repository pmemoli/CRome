use crome::driver;
use std::fs;

#[test]
#[cfg(all(feature = "tacky", not(feature = "codegen")))]
fn test_tacky_float_valid() {
    let content = fs::read_to_string("./tests/source/float_valid.c").unwrap();
    driver::compiler::compiler(&content).unwrap();
}
