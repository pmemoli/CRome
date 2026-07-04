mod common;
mod driver;

crate::invalid_tests!(
    "parser",
    driver::parser,
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

crate::valid_tests!(
    "parser",
    driver::parser,
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
