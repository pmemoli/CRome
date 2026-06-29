use anyhow::{Ok, Result, bail};
use std::{fs, process::Command};
use tempfile::NamedTempFile;

pub fn preprocessor(content: &str) -> Result<String> {
    let cpp_file = NamedTempFile::new()?;
    let cpp_path = cpp_file.path();
    fs::write(cpp_path, content)?;

    let preprocessor_file = NamedTempFile::new()?;
    let preprocessor_file_path = preprocessor_file.path();
    let preprocessor_status = Command::new("cpp")
        .arg("-P")
        .arg(cpp_path)
        .arg(preprocessor_file_path)
        .status()?;

    if !preprocessor_status.success() {
        bail!("Preprocessing failed at runtime.");
    }

    let content = fs::read_to_string(preprocessor_file_path)?;
    Ok(content)
}
