use std::panic;
use walkdir::WalkDir;

use anyhow::Result;

pub fn test_pass_invalid(folder_path: &str, pass: fn(&str) -> Result<String>) {
    for entry in WalkDir::new(folder_path) {
        let entry = entry.unwrap();
        let path = entry.path();

        if path.is_file() && path.to_str().unwrap().ends_with(".c") {
            let result = panic::catch_unwind(|| pass(path.to_str().unwrap()));
            assert!(result.is_err(), "Expected panic for file: {:?}", path);
        }
    }
}

pub fn test_pass_valid(folder_path: &str, pass: fn(&str) -> Result<String>) {
    for entry in WalkDir::new(folder_path) {
        let entry = entry.unwrap();
        let path = entry.path();

        if path.is_file() && path.to_str().unwrap().ends_with(".c") {
            let result = panic::catch_unwind(|| pass(path.to_str().unwrap()));
            assert!(
                !result.is_err(),
                "File {:?} panicked when it shouldn't have done",
                path
            );
        }
    }
}

#[macro_export]
macro_rules! invalid_tests {
    ($stage:literal, $pass_func:path, $($feature:literal),*) => {
        $(
            paste::paste! {
                #[test]
                #[cfg(feature = $stage)]
                fn [<test_ $stage _invalid_ $feature>]() {
                    let folder_path = format!("./tests/source/{}/invalid/{}", $feature, $stage);
                    common::test_pass_invalid(&folder_path, $pass_func);
                }
            }
        )*
    };
}

#[macro_export]
macro_rules! valid_tests {
    ($stage:literal, $pass_func:path, $($feature:literal),*) => {
        $(
            paste::paste! {
                #[test]
                #[cfg(feature = $stage)]
                fn [<test_ $stage _valid_ $feature>]() {
                    let folder_path = format!("./tests/source/{}/valid", $feature);
                    common::test_pass_valid(&folder_path, $pass_func);
                }
            }
        )*
    };
}
