C compiler written in Rust based on Sandler Nora's book "Writing a C Compiler". The project is just the the preprocessed C to ASM compiler. Preprocessor and Linker comes from gcc. 

Very much a WIP.

TODO:
- Write parser for unary expressions.
- Make lexer and parser work with Result's rather than just panicking for errors.

## AST Specification
program = Program(function_definition)
function_definition = Function(identifier name, statement body)
statement = Return(exp)
exp = Constant(int) | Unary(unary_operator, exp)
unary_operator = Complement | Negate

## Formal Grammar
<program> ::= <function>
<function> ::= "int" <identifier> "(" "void" ")" "{" <statement> "}"
<statement> ::= "return" <exp> ";"
<exp> ::= <int> | <unop> <exp> | "(" <exp> ")"
<unop> ::= "-" | "~"
<identifier> ::= ? An identifier token ?
<int> ::= ? A constant token ?
