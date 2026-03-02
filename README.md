C compiler written in Rust based on Sandler Nora's book "Writing a C Compiler". 

Preprocessor and Linker comes from gcc.

Very much a WIP.

TODO:

- Implement file cleanup on error for the temp files from the compiler driver.
- Handle --lex, --parse, and --codegen flags with clap rather than whatever i'm doing.
