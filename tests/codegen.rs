#[cfg(feature = "codegen")]
mod codegen_tests {
    use crome::lexer::lexical_analysis;
    use crome::parser::parse_program;
    use crome::semantic::semantic_analysis;
    use crome::tacky::ast_program_to_tacky;
    use crome::codegen::codegen_program;
    use crome::symbol;
    use std::fs;

    #[test]
    fn test_codegen_float_valid() {
        let mut symbol_table = symbol::SymbolTable::new();

        let content = fs::read_to_string("./tests/source/float_valid.c").unwrap();
        let tokens = lexical_analysis(&content);
        let ast = parse_program(&mut tokens.clone());
        let resolved_ast = semantic_analysis(&ast, &mut symbol_table);
        let tacky_ast = ast_program_to_tacky(&resolved_ast, &mut symbol_table);
        codegen_program(&tacky_ast, &mut symbol_table);
    }
}
