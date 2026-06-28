use anyhow::{Result, bail};
use clap::Parser;
use std::fs;
use std::process::Command;
use tempfile::{Builder, NamedTempFile};

use crome::driver;

#[cfg(feature = "codegen")]
use crome::codegen;
#[cfg(feature = "emission")]
use crome::emission;
#[cfg(feature = "lex")]
use crome::lexer;
#[cfg(feature = "parser")]
use crome::parser;
#[cfg(feature = "validate")]
use crome::semantic;
#[cfg(feature = "validate")]
use crome::symbol;
#[cfg(feature = "tacky")]
use crome::tacky;
#[cfg(feature = "parser")]
use crome::types;

#[derive(Parser)]
#[command(name = "crab")]
struct Args {
    #[arg(help = "Source c file to compile")]
    source_file: String,

    // Only generate relocatable object file
    #[arg(short = 'c')]
    c: bool,

    // Link libraries
    #[arg(short = 'l')]
    l: Vec<String>,

    // Debug info
    #[arg(short = 'g')]
    g: bool,

    #[arg(long)]
    lex: bool,

    #[arg(long)]
    validate: bool,

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

    #[cfg(feature = "validate")]
    let mut symbol_table = symbol::SymbolTable::new();

    #[cfg(feature = "lex")]
    let mut tokens = lexer::lexical_analysis(&content);

    if args.lex {
        return Ok(());
    }

    #[cfg(feature = "parser")]
    let ast = parser::parse_program(&mut tokens);

    if args.parse {
        return Ok(());
    }

    #[cfg(feature = "validate")]
    let resolved_ast = semantic::semantic_analysis(&ast, &mut symbol_table);

    if args.validate {
        return Ok(());
    }

    #[cfg(feature = "tacky")]
    let tacky_ast = tacky::ast_program_to_tacky(&resolved_ast, &mut symbol_table);

    if args.tacky {
        return Ok(());
    }

    #[cfg(feature = "codegen")]
    let asm_ast = codegen::codegen_program(&tacky_ast, &mut symbol_table);

    if args.codegen {
        return Ok(());
    }

    #[cfg(feature = "emission")]
    {
        let asm_str = emission::emission_program(&asm_ast, &symbol_table);

        // Runs assembler and linker
        let assembly_file = Builder::new().suffix(".s").tempfile()?;
        let assembly_file_path = assembly_file.path();
        fs::write(assembly_file_path, asm_str.clone())?;

        let stem = source_file.strip_suffix(".c").unwrap_or(source_file);
        let output_file = if args.c {
            format!("{}.o", stem)
        } else {
            stem.to_string()
        };

        let mut gcc_command = Command::new("gcc");

        if args.c {
            gcc_command.arg("-c"); // do not link, only generate object file
        }

        if args.g {
            let debug_assembly_file = format!("{}.s", stem);
            fs::write(debug_assembly_file, asm_str.clone())?;
            gcc_command.arg("-g"); // generate debug information
        }

        let status = gcc_command
            .arg(assembly_file_path)
            .arg("-o")
            .arg(&output_file)
            .args(args.l.iter().map(|lib| format!("-l{}", lib)))
            .status()?;

        if !status.success() {
            bail!("Object generation and linking failed at runtime.");
        }
    }

    Ok(())
}
