mod common;

#[cfg(feature = "validate")]
mod semantic_tests {
    use crate::common::{Stage, compile_up_to};
    use std::fs;

    #[test]
    fn test_semantic_float_valid() {
        let content = fs::read_to_string("./tests/source/float_valid.c").unwrap();
        compile_up_to(&content, Stage::Semantic);
    }
}
