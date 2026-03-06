C compiler written in Rust based on Sandler Nora's book "Writing a C Compiler". The project is just the the preprocessed C to ASM compiler. Preprocessor and Linker comes from gcc. 

Very much a WIP.

TODO:
- Make the lexer return Result<Vec<Token>> rather than just Vec<Token> to process lexical errors
- Make the lexer work with exclusively an enum token. The rule vector should be of tuples (enum, generator function)
