#[cfg(feature = "validate")]
mod semantic_tests {
    use crome::lexer::lexical_analysis;
    use crome::parser::parse_program;
    use crome::semantic::semantic_analysis;
    use crome::symbol;
    use std::fs;

    #[test]
    fn test_semantic_float_valid() {
        let mut symbol_table = symbol::SymbolTable::new();

        let content = fs::read_to_string("./tests/source/float_valid.c").unwrap();
        let tokens = lexical_analysis(&content);
        let ast = parse_program(&mut tokens.clone());
        semantic_analysis(&ast, &mut symbol_table);
    }
}
