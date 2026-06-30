use anyhow::{Ok, Result};

#[cfg(feature = "semantic")]
use crate::symbol;

// Could probably be refactored to something cleaner, but whatever
pub fn compiler(source: &str) -> Result<String> {
    let asm_str = String::new();

    #[cfg(feature = "semantic")]
    let mut symbol_table = symbol::SymbolTable::new();

    #[cfg(feature = "lex")]
    let tokens = crate::lexer::lexical_analysis(source);
    #[cfg(all(feature = "lex", not(feature = "parser")))]
    return Ok(format!("{:#?}", tokens));

    #[cfg(feature = "parser")]
    let ast = crate::parser::parse_program(&mut tokens.clone());
    #[cfg(all(feature = "parser", not(feature = "semantic")))]
    return Ok(format!("{:#?}", ast));

    #[cfg(feature = "semantic")]
    let resolved_ast = crate::semantic::semantic_analysis(&ast, &mut symbol_table);
    #[cfg(all(feature = "semantic", not(feature = "tacky")))]
    return Ok(format!("{:#?}", resolved_ast));

    #[cfg(feature = "tacky")]
    let tacky_ast = crate::tacky::ast_program_to_tacky(&resolved_ast, &mut symbol_table);
    #[cfg(all(feature = "tacky", not(feature = "codegen")))]
    return Ok(format!("{:#?}", tacky_ast));

    #[cfg(feature = "codegen")]
    let ast = crate::codegen::codegen_program(&tacky_ast, &mut symbol_table);
    #[cfg(all(feature = "codegen", not(feature = "emission")))]
    return Ok(format!("{:#?}", ast));

    #[cfg(feature = "emission")]
    let asm_str = crate::emission::emission_program(&ast, &symbol_table);
    return Ok(asm_str);
}
