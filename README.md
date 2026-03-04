C compiler written in Rust based on Sandler Nora's book "Writing a C Compiler". The project is just the the preprocessed C to ASM compiler. Preprocessor and Linker comes from gcc. 

Very much a WIP.

TODO:

- Test the lexer
- Write the "compile" wrapper to implement the flag functionality for testing
- Make the lexer return Result<Vec<Token>> rather than just Vec<Token>
