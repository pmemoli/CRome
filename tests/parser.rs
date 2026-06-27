#[cfg(feature = "parser")]
mod parser_tests {
    use crome::lexer::lexical_analysis;
    use crome::parser::parse_program;
    use std::fs;

    #[test]
    fn test_parser_float_valid() {
        let content = fs::read_to_string("./tests/source/float_valid.c").unwrap();
        let tokens = lexical_analysis(&content);
        parse_program(&mut tokens.clone());
    }
}
