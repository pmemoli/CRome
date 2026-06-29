use crome::driver;
use std::fs;

#[test]
#[cfg(feature = "emission")]
fn test_emission_float_valid() {
    let content = fs::read_to_string("./tests/source/float_valid.c").unwrap();
    driver::compiler::compiler(&content).unwrap();
}
