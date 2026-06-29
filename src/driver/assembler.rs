use anyhow::{Ok, Result, bail};
use std::{fs, process::Command};
use tempfile::NamedTempFile;

pub fn assembler(content: &str, debug: bool) -> Result<Vec<u8>> {
    let assembler_file = NamedTempFile::new()?;
    let assembler_file_path = assembler_file.path();
    fs::write(assembler_file_path, content)?;

    let output_file = NamedTempFile::new()?;
    let output_file_path = output_file.path();

    let mut assembler_command = Command::new("as");

    assembler_command
        .arg(assembler_file_path)
        .arg("-o")
        .arg(output_file_path);

    if debug {
        assembler_command.arg("-g");
    }

    let assembler_status = assembler_command.status()?;

    if !assembler_status.success() {
        bail!("Assembly failed at runtime.");
    }

    let object_content = fs::read(output_file_path)?;
    Ok(object_content)
}
