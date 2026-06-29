use std::process::Command;

use anyhow::{Ok, Result, bail};

pub fn linker(content: &[u8], libs: Vec<&str>, output_file_path: &str) -> Result<()> {
    let linker_file = tempfile::NamedTempFile::new()?;
    let linker_file_path = linker_file.path();
    std::fs::write(linker_file_path, content)?;

    // Simplest way to use ld
    let status = Command::new("gcc")
        .arg(linker_file_path)
        .arg("-o")
        .arg(output_file_path)
        .args(libs.iter().map(|lib| format!("-l{}", lib)))
        .status()?;

    if !status.success() {
        bail!("Linking failed at runtime.");
    }

    return Ok(());
}
