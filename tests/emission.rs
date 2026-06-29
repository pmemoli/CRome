mod driver;

// testing through exit status code, ugly as hell but its the simplest without linking stdio.h

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
