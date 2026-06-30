mod common;
mod driver;

crate::valid_tests!(
    "tacky",
    driver::tacky,
    "minimal_compiler",
    "unary_operators",
    "binary_operators",
    "logical_operators",
    "local_variables",
    "compound_statements",
    "loops",
    "if_statements",
    "functions",
    "linkage",
    "longs",
    "unsigned_integers",
    "doubles"
);
