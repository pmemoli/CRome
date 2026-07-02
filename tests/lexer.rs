mod common;
mod driver;

crate::invalid_tests!(
    "lex",
    driver::lexer,
    "minimal_compiler",
    "if_statements",
    "longs",
    "doubles",
    "unsigned_integers",
    "pointers"
);

crate::valid_tests!(
    "lex",
    driver::lexer,
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
