use std::collections::HashMap;
use std::path::Path;
use walkdir::WalkDir;
mod driver;
use std::panic;

macro_rules! valid_tests {
    ($(($feature:literal, [$($lib:literal),*])),* $(,)?) => {
        $(
            paste::paste! {
                #[test]
                #[cfg(feature = "emission")]
                fn [<test_emission_valid_ $feature>]() {
                    let folder_path = format!("./tests/source/{}/valid", $feature);
                    let libs: Vec<String> = vec![$($lib.to_string()),*];
                    emission_test_pass_valid(&folder_path, &libs);
                }
            }
        )*
    };
}

valid_tests!(
    ("minimal_compiler", []),
    ("unary_operators", []),
    ("binary_operators", []),
    ("logical_operators", []),
    ("local_variables", []),
    ("compound_statements", []),
    ("loops", []),
    ("if_statements", []),
    ("functions", []),
    ("linkage", []),
    ("longs", []),
    ("unsigned_integers", []),
    ("doubles", ["m"]),
    ("floats", ["m"]),
);

pub fn emission_test_pass_valid(folder_path: &str, libs: &Vec<String>) {
    let expected = load_expected(folder_path);

    for entry in WalkDir::new(folder_path).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();

        if is_leaf_directory(path) {
            let test_name = path.file_name().unwrap().to_str().unwrap();

            let compilation_units: Vec<String> = std::fs::read_dir(path)
                .unwrap()
                .filter_map(|e| e.ok())
                .map(|e| e.path().to_str().unwrap().to_string())
                .collect();

            let expected_exit_code = expected
                .get(test_name)
                .unwrap_or_else(|| panic!("Expected exit code not found for test: {}", test_name));

            run_and_check(&compilation_units, libs, &test_name, *expected_exit_code);
        }
    }
}

fn run_and_check(
    compilation_units: &Vec<String>,
    libs: &Vec<String>,
    test_name: &str,
    expected: i32,
) {
    let result = panic::catch_unwind(|| driver::compile_and_run(compilation_units, libs));

    match result.unwrap() {
        Ok(actual) => assert!(
            actual == expected,
            "Expected exit code {}, got {} for {:?}",
            expected,
            actual,
            test_name
        ),
        Err(e) => panic!("Error compiling {:?}: {}", test_name, e),
    }
}

// parses the expected.result file in each feature folder
fn load_expected(folder_path: &str) -> HashMap<String, i32> {
    let result_file = format!("{}/expected.result", folder_path);
    let content = std::fs::read_to_string(&result_file).unwrap();

    content
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| {
            let mut parts = l.rsplitn(2, ' ');
            let code: i32 = parts.next().unwrap().trim().parse().unwrap();
            let path = parts.next().unwrap().trim().to_string();
            (path, code)
        })
        .collect()
}

pub fn is_leaf_directory(path: &Path) -> bool {
    if !path.is_dir() {
        return false;
    }

    for entry in std::fs::read_dir(path).unwrap() {
        let entry = entry.unwrap();
        let entry_path = entry.path();
        if entry_path.is_dir() {
            return false;
        }
    }

    true
}
