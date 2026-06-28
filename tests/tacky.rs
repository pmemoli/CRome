mod common;

#[cfg(feature = "tacky")]
mod tacky_tests {
    use crate::common::{Stage, compile_up_to};
    use std::fs;

    #[test]
    fn test_tacky_float_valid() {
        let content = fs::read_to_string("./tests/source/float_valid.c").unwrap();
        compile_up_to(&content, Stage::Tacky);
    }
}
