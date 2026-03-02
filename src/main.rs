use std::env;
use std::fs;
use std::io::{Error, ErrorKind};
use std::process::Command;

fn main() -> Result<(), std::io::Error> {
    let args: Vec<String> = env::args().collect();

    let mut lex_flag = false;
    let mut parse_flag = false;
    let mut codegen_flag = false;
    let mut file: Option<&str> = None;

    for arg in args.iter().skip(1) {
        match arg.as_str() {
            "--lex" => lex_flag = true,
            "--parse" => parse_flag = true,
            "--codegen" => codegen_flag = true,
            _ => file = Some(&arg),
        }
    }

    let Some(file_name) = file else {
        return Err(Error::new(
            ErrorKind::InvalidInput,
            "No source file provided.",
        ));
    };

    // Runs preprocessor
    let preprocessor_file_path = "./temp.i";
    let preprocessor_status = Command::new("gcc")
        .arg("-E") // Run only preprocessor
        .arg("-P") // No linemarkers
        .arg(file_name)
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
    let output_file = file_name.strip_suffix(".c").unwrap_or(file_name);
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
