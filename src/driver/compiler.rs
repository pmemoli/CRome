use anyhow::{Ok, Result};

#[cfg(feature = "validate")]
use crate::symbol;

#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub enum Stage {
    Lex,
    Parse,
    Semantic,
    Tacky,
    Codegen,
}

pub fn compiler(source: &str, stage: Stage) -> Result<String> {
    #[cfg(feature = "validate")]
    let mut symbol_table = symbol::SymbolTable::new();

    #[cfg(feature = "lex")]
    let tokens = crate::lexer::lexical_analysis(source);
    if stage == Stage::Lex {
        return Ok(format!("{:?}", tokens));
    }

    #[cfg(feature = "parser")]
    let ast = crate::parser::parse_program(&mut tokens.clone());
    if stage == Stage::Parse {
        return Ok(format!("{:?}", ast));
    }

    #[cfg(feature = "validate")]
    let resolved_ast = crate::semantic::semantic_analysis(&ast, &mut symbol_table);
    if stage == Stage::Semantic {
        return Ok(format!("{:?}", resolved_ast));
    }

    #[cfg(feature = "tacky")]
    let tacky_ast = crate::tacky::ast_program_to_tacky(&resolved_ast, &mut symbol_table);
    if stage == Stage::Tacky {
        return Ok(format!("{:?}", tacky_ast));
    }

    #[cfg(feature = "codegen")]
    let ast = crate::codegen::codegen_program(&tacky_ast, &mut symbol_table);
    if stage == Stage::Codegen {
        return Ok(format!("{:?}", ast));
    }

    #[cfg(feature = "emission")]
    let asm_str = crate::emission::emission_program(&ast, &symbol_table);
    return Ok(asm_str);
}
