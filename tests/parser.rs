mod common;

#[cfg(feature = "parser")]
mod parser_tests {
    use crate::common::{Stage, compile_up_to};
    use std::fs;

    #[test]
    fn test_parser_float_valid() {
        let content = fs::read_to_string("./tests/source/float_valid.c").unwrap();
        compile_up_to(&content, Stage::Parse);
    }
}
