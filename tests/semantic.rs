mod common;
mod driver;

crate::invalid_tests!(
    "semantic",
    driver::semantic,
    "compound_statements",
    "if_statements",
    "local_variables",
    "longs",
    "loops",
    "doubles",
    "functions",
    "linkage",
    "unsigned_integers",
    "pointers"
);

crate::valid_tests!(
    "semantic",
    driver::semantic,
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
    "doubles",
    "pointers"
);
