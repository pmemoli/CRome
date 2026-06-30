use std::process::Command;

use anyhow::{Ok, Result, bail};

use std::io::Write;

pub fn linker(contents: &[Vec<u8>], libs: Vec<&str>, output_file_path: &str) -> Result<()> {
    let mut temp_files = Vec::new();
    for content in contents {
        let mut linker_file = tempfile::NamedTempFile::new()?;
        linker_file.write_all(content)?;
        temp_files.push(linker_file);
    }

    let mut command = Command::new("gcc");
    for temp_file in &temp_files {
        command.arg(temp_file.path());
    }

    let status = command
        .arg("-o")
        .arg(output_file_path)
        .args(libs.iter().map(|lib| format!("-l{}", lib)))
        .status()?;

    if !status.success() {
        bail!("Linking failed at runtime.");
    }
    Ok(())
}
