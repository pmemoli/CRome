mod driver;

#[test]
#[cfg(feature = "tacky")]
fn test_tacky_float_valid() {
    driver::tacky("./tests/source/float_valid.c").unwrap();
}
