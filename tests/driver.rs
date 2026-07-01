use anyhow::{Result, bail};
use crome::driver;

use std::process::{Command, Stdio};

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

#[cfg(feature = "semantic")]
pub fn semantic(path: &str) -> Result<String> {
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
pub fn compile_and_run(paths: &Vec<String>, libs: &Vec<String>) -> Result<i32> {
    let mut reloc_elfs = Vec::new();

    for path in paths {
        let asm = if path.ends_with(".s") {
            std::fs::read_to_string(path)?
        } else {
            emission(path)?
        };
        let reloc_elf = driver::assembler::assembler(&asm, false)?;
        reloc_elfs.push(reloc_elf);
    }

    let libs_str: Vec<&str> = libs.iter().map(|s| s.as_str()).collect();

    let output_file = tempfile::NamedTempFile::new().unwrap().into_temp_path();
    let output_path = output_file.to_str().unwrap().to_string();

    driver::linker::linker(&reloc_elfs, libs_str, &output_path)?;

    // don't want the output messing with the test output.
    let status = Command::new(&output_path)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()?;
    let exit_code = status.code();

    let Some(actual) = exit_code else {
        bail!("No exit code returned.")
    };

    return Ok(actual);
}
