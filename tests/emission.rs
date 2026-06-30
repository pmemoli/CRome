use anyhow::Result;
use walkdir::WalkDir;
mod driver;
use std::panic;

// TODO: Modularize the emission code pass (looking at the exit code + panic)

pub fn emission_test_pass_valid(folder_path: &str, pass: fn(&str) -> Result<String>) {
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

#[test]
#[cfg(feature = "emission")]
fn test_emission_float_valid() {
    let empty_libs: Vec<String> = Vec::new();
    let exit_code = driver::compile_exit_code("./tests/source/float_valid.c", &empty_libs);
    let expected_status_code = 3;

    match exit_code {
        Ok(actual_status_code) => {
            assert!(
                actual_status_code == expected_status_code,
                "Expected status code: {}, but got: {}",
                expected_status_code,
                actual_status_code
            );
        }
        Err(e) => {
            panic!("Error occurred while compiling: {}", e);
        }
    }
}
