#[cfg(feature = "lex")]
mod lexer_tests {
    use crome::lexer::lexical_analysis;
    use std::fs;

    #[test]
    fn test_lexer_valid() {
        let content = fs::read_to_string("./tests/source/lexer_valid.c").unwrap();
        lexical_analysis(&content);
    }
}
