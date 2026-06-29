use anyhow::{Result, bail};
use crome::driver;

#[cfg(feature = "lex")]
pub fn lexer(path: &str) -> Result<String> {
    let content = std::fs::read_to_string(path).unwrap();
    let preprocessed_content = driver::preprocessor::preprocessor(&content)?;

    let tokens = crome::lexer::lexical_analysis(&preprocessed_content);
    return Ok(format!("{:#?}", tokens));
}

#[cfg(feature = "parser")]
pub fn parser(path: &str) -> Result<String> {
    let content = std::fs::read_to_string(path).unwrap();
    let preprocessed_content = driver::preprocessor::preprocessor(&content)?;

    let tokens = crome::lexer::lexical_analysis(&preprocessed_content);
    let ast = crome::parser::parse_program(&mut tokens.clone());
    return Ok(format!("{:#?}", ast));
}

#[cfg(feature = "validate")]
pub fn validate(path: &str) -> Result<String> {
    let content = std::fs::read_to_string(path).unwrap();
    let preprocessed_content = driver::preprocessor::preprocessor(&content)?;

    let tokens = crome::lexer::lexical_analysis(&preprocessed_content);
    let ast = crome::parser::parse_program(&mut tokens.clone());
    let mut symbol_table = crome::symbol::SymbolTable::new();
    let resolved_ast = crome::semantic::semantic_analysis(&ast, &mut symbol_table);
    return Ok(format!("{:#?}", resolved_ast));
}

#[cfg(feature = "tacky")]
pub fn tacky(path: &str) -> Result<String> {
    let content = std::fs::read_to_string(path).unwrap();
    let preprocessed_content = driver::preprocessor::preprocessor(&content)?;

    let tokens = crome::lexer::lexical_analysis(&preprocessed_content);
    let ast = crome::parser::parse_program(&mut tokens.clone());
    let mut symbol_table = crome::symbol::SymbolTable::new();
    let resolved_ast = crome::semantic::semantic_analysis(&ast, &mut symbol_table);
    let tacky_ast = crome::tacky::ast_program_to_tacky(&resolved_ast, &mut symbol_table);
    return Ok(format!("{:#?}", tacky_ast));
}

#[cfg(feature = "codegen")]
pub fn codegen(path: &str) -> Result<String> {
    let content = std::fs::read_to_string(path).unwrap();
    let preprocessed_content = driver::preprocessor::preprocessor(&content)?;

    let tokens = crome::lexer::lexical_analysis(&preprocessed_content);
    let ast = crome::parser::parse_program(&mut tokens.clone());
    let mut symbol_table = crome::symbol::SymbolTable::new();
    let resolved_ast = crome::semantic::semantic_analysis(&ast, &mut symbol_table);
    let tacky_ast = crome::tacky::ast_program_to_tacky(&resolved_ast, &mut symbol_table);
    let asm_ast = crome::codegen::codegen_program(&tacky_ast, &mut symbol_table);

    return Ok(format!("{:#?}", asm_ast));
}

#[cfg(feature = "emission")]
pub fn emission(path: &str) -> Result<String> {
    let content = std::fs::read_to_string(path).unwrap();
    let preprocessed_content = driver::preprocessor::preprocessor(&content)?;

    let tokens = crome::lexer::lexical_analysis(&preprocessed_content);
    let ast = crome::parser::parse_program(&mut tokens.clone());
    let mut symbol_table = crome::symbol::SymbolTable::new();
    let resolved_ast = crome::semantic::semantic_analysis(&ast, &mut symbol_table);
    let tacky_ast = crome::tacky::ast_program_to_tacky(&resolved_ast, &mut symbol_table);
    let asm_ast = crome::codegen::codegen_program(&tacky_ast, &mut symbol_table);
    let asm_str = crome::emission::emission_program(&asm_ast, &symbol_table);

    return Ok(asm_str);
}

#[cfg(feature = "emission")]
pub fn compile_exit_code(path: &str, libs: &Vec<String>) -> Result<i32> {
    let result = emission(path).unwrap();
    let reloc_elf = driver::assembler::assembler(&result, false).unwrap();

    let output_file = tempfile::NamedTempFile::new().unwrap().into_temp_path();
    let output_path = output_file.to_str().unwrap().to_string();

    let libs_str: Vec<&str> = libs.iter().map(|s| s.as_str()).collect();
    driver::linker::linker(&reloc_elf, libs_str, &output_path).unwrap();

    let status = std::process::Command::new(&output_path).status()?;
    let exit_code = status.code();

    let Some(actual) = exit_code else {
        bail!("No exit code returned.")
    };

    return Ok(actual);
}
