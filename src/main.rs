use anyhow::{Result, bail};
use clap::Parser;
use std::fs;
use std::process::Command;
use tempfile::NamedTempFile;

mod codegen;
mod emission;
mod lexer;
mod parser;

#[derive(Parser)]
#[command(name = "crab")]
struct Args {
    #[arg(help = "Source c file to compile")]
    source_file: String,

    #[arg(long)]
    lex: bool,

    #[arg(long)]
    parse: bool,

    #[arg(long)]
    codegen: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let source_file = &args.source_file;

    // Runs preprocessor
    let preprocessor_file = NamedTempFile::new()?;
    let preprocessor_file_path = preprocessor_file.path();
    let preprocessor_status = Command::new("gcc")
        .arg("-E") // Run only preprocessor
        .arg("-P") // No linemarkers
        .arg(source_file)
        .arg("-o")
        .arg(preprocessor_file_path)
        .status()?;

    if !preprocessor_status.success() {
        bail!("Preprocessing failed at runtime.");
    }

    // Runs compiler
    let content = fs::read_to_string(preprocessor_file_path)?;
    compile(&content, args.lex, args.parse, args.codegen);

    // let assembly_file = NamedTempFile::new()?;
    // let assembly_file_path = assembly_file.path();
    //
    // // Runs linker
    // let output_file = source_file.strip_suffix(".c").unwrap_or(source_file);
    // let linker_status = Command::new("gcc")
    //     .arg(assembly_file_path)
    //     .arg("-o")
    //     .arg(output_file)
    //     .status()?;
    //
    // if !linker_status.success() {
    //     bail!("Linking failed at runtime.");
    // }

    Ok(())
}

fn compile(content: &str, lex_flag: bool, parse_flag: bool, codegen_flag: bool) {
    let mut tokens = crate::lexer::lexical_analysis(&content);

    if lex_flag {
        return;
    }

    let ast = crate::parser::parse_program(&mut tokens);

    if parse_flag {
        return;
    }

    let asm_ast = crate::codegen::codegen_program(&ast);

    if codegen_flag {
        return;
    }

    println!("{:#?}", asm_ast);
}
