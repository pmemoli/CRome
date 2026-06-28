mod common;

#[cfg(feature = "lex")]
mod lexer_tests {
    use crate::common::{Stage, compile_up_to};
    use std::fs;

    #[test]
    fn test_lexer_valid() {
        let content = fs::read_to_string("./tests/source/lexer_valid.c").unwrap();
        compile_up_to(&content, Stage::Lex);
    }
}
