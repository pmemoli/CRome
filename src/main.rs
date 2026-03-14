use anyhow::{Result, bail};
use clap::Parser;
use std::fs;
use std::process::Command;
use tempfile::{Builder, NamedTempFile};

mod codegen;
mod emission;
mod lexer;
mod parser;
mod tacky;

#[derive(Parser)]
#[command(name = "crab")]
struct Args {
    #[arg(help = "Source c file to compile")]
    source_file: String,

    #[arg(long)]
    lex: bool,

    #[arg(long)]
    tacky: bool,

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

    let mut tokens = crate::lexer::lexical_analysis(&content);

    if args.lex {
        return Ok(());
    }

    let ast = crate::parser::parse_program(&mut tokens);
    println!("{:#?}", ast);

    if args.parse {
        return Ok(());
    }

    // tacky generation

    if args.tacky {
        return Ok(());
    }

    let asm_ast = crate::codegen::codegen_program(&ast);

    if args.codegen {
        return Ok(());
    }

    let asm_str = crate::emission::emission_program(&asm_ast);

    let assembly_file = Builder::new().suffix(".s").tempfile()?;
    let assembly_file_path = assembly_file.path();

    fs::write(assembly_file_path, asm_str)?;

    // Runs linker
    let output_file = source_file.strip_suffix(".c").unwrap_or(source_file);
    let linker_status = Command::new("gcc")
        .arg(assembly_file_path)
        .arg("-o")
        .arg(output_file)
        .status()?;

    if !linker_status.success() {
        bail!("Linking failed at runtime.");
    }

    Ok(())
}
