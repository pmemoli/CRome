use std::env;
use std::fs;
use std::io::{Error, ErrorKind};
use std::process::Command;

fn main() -> Result<(), std::io::Error> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        return Err(Error::new(
            ErrorKind::InvalidInput,
            "Need a source file to compile.",
        ));
    }

    let source_file = &args[1];

    // Runs preprocessor
    let preprocessor_file_path = "./temp.i";
    let preprocessor_status = Command::new("gcc")
        .arg("-E") // Run only preprocessor
        .arg("-P") // No linemarkers
        .arg(source_file)
        .arg("-o")
        .arg(preprocessor_file_path)
        .status()?;

    if !preprocessor_status.success() {
        return Err(Error::new(
            ErrorKind::Other,
            "Preprocessor failed at runtime.",
        ));
    }

    // Runs compiler (TODO)
    let assembly_file_path = "./temp.s";
    fs::remove_file(preprocessor_file_path)?;

    // Runs linker
    let output_file = source_file.strip_suffix(".c").unwrap_or(source_file);
    let linker_status = Command::new("gcc")
        .arg(assembly_file_path)
        .arg("-o")
        .arg(output_file)
        .status()?;

    if !linker_status.success() {
        return Err(Error::new(ErrorKind::Other, "Linking failed at runtime."));
    }

    fs::remove_file(assembly_file_path)?;

    Ok(())
}
