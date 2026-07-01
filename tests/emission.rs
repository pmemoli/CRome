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
);

pub fn emission_test_pass_valid(folder_path: &str, libs: &Vec<String>) {
    let expected = load_expected(folder_path);

    // standalone tests (may include asm helpers)
    for entry in WalkDir::new(folder_path) {
        let entry = entry.unwrap();
        let path = entry.path();
        let path_str = path.to_str().unwrap();

        if path_str.contains("/libraries") || !path.is_file() || !path_str.ends_with(".c") {
            continue;
        }

        let rel = path_str
            .strip_prefix(&format!("{}/", folder_path))
            .unwrap()
            .to_string();

        let &exp = expected
            .get(&rel)
            .unwrap_or_else(|| panic!("No expected exit code for {}", rel));

        let mut compilation_units = vec![path_str.to_string()];
        compilation_units.extend(asm_helpers(path_str));

        run_and_check(&compilation_units, libs, &rel, exp);
    }

    // tests that require linking (foo_client.c + foo.c)
    let lib_dir = format!("{}/libraries", folder_path);
    if Path::new(&lib_dir).exists() {
        for entry in WalkDir::new(&lib_dir) {
            let entry = entry.unwrap();
            let path = entry.path();
            let path_str = path.to_str().unwrap();

            if !path.is_file() || !path_str.ends_with("_client.c") {
                continue;
            }

            let rel = path_str
                .strip_prefix(&format!("{}/", folder_path))
                .unwrap()
                .to_string();

            let &exp = expected
                .get(&rel)
                .unwrap_or_else(|| panic!("No expected exit code for {}", rel));

            let impl_file = path_str.replace("_client.c", ".c");
            let mut compilation_units = vec![path_str.to_string()];
            if Path::new(&impl_file).exists() {
                compilation_units.push(impl_file);
            }

            run_and_check(&compilation_units, libs, &rel, exp);
        }
    }
}

fn run_and_check(compilation_units: &Vec<String>, libs: &Vec<String>, rel: &str, expected: i32) {
    let result = panic::catch_unwind(|| driver::compile_and_run(compilation_units, libs));

    match result.unwrap() {
        Ok(actual) => assert!(
            actual == expected,
            "Expected exit code {}, got {} for {:?}",
            expected,
            actual,
            rel
        ),
        Err(e) => panic!("Error compiling {:?}: {}", rel, e),
    }
}

fn asm_helpers(c_path: &str) -> Vec<String> {
    let dir = Path::new(c_path).parent().unwrap();

    let mut helpers = Vec::new();

    for entry in std::fs::read_dir(dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        let name = path.to_str().unwrap_or("");

        if name.ends_with("_linux.s") || (name.ends_with(".s") && !name.contains("_osx")) {
            helpers.push(name.to_string());
        }
    }

    helpers
}

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
