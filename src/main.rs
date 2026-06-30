use anyhow::Result;
use clap::Parser;
use std::fs;

use crome::driver;

#[derive(Parser)]
#[command(name = "crab")]
struct Args {
    #[arg(help = "Source c file to compile")]
    source_files: Vec<String>,

    #[arg(short = 'o', help = "output file name")]
    o: String,

    #[arg(short = 'c', help = "Only output relocatable object file")]
    c: bool,

    #[arg(short = 'l', help = "Link with library")]
    l: Vec<String>,

    #[arg(short = 'g', help = "Generate debug information for gdb")]
    g: bool,

    #[arg(short = 'p', help = "Print the compiler output to stdout")]
    p: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let source_files = &args.source_files;
    let mut relocatable_objects: Vec<Vec<u8>> = Vec::new();

    for source_file in source_files {
        // Run preprocessor (cpp)
        let content = fs::read_to_string(source_file)?;
        let preprocessed = driver::preprocessor::preprocessor(&content)?;

        // Run compiler
        let compiler_output = driver::compiler::compiler(&preprocessed)?;

        if args.p {
            println!("{}", compiler_output);
        }

        #[cfg(not(feature = "emission"))]
        return Ok(());

        // Run assembler
        let stem = args.o.clone();

        // if debug flag is set dump the assembly code to a file for gdb to reference
        if args.g {
            fs::write(format!("{}.s", stem), &compiler_output)?;
        }

        let relocatable_object = driver::assembler::assembler(&compiler_output, args.g)?;
        relocatable_objects.push(relocatable_object.clone());

        // if -c flag is set just dump the relocatable object to a file and exit
        if args.c {
            fs::write(format!("{}.o", stem), &relocatable_object)?;
        }
    }

    // if -c flag is set just dump the relocatable object to a file and exit
    if args.c {
        return Ok(());
    }

    // Run linker
    let libs: Vec<&str> = args.l.iter().map(|s| s.as_str()).collect();
    driver::linker::linker(&relocatable_objects, libs, &args.o)?;

    Ok(())
}
