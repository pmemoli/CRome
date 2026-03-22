C compiler written in Rust based on Sandler Nora's book "Writing a C Compiler". The project is just the the preprocessed C to ASM compiler. Preprocessor and Linker comes from gcc. 

Very much a WIP.

TODO:
- Chapter 4 parser

Backlog:
- Potentially flatten some tacky to asm passes into one function rather than a gazillion.
- Make lexer and parser work with Result's rather than just panicking for errors.

# Compiler passes

## Lexer

## AST Specification
```
program = Program(function_definition)
function_definition = Function(identifier name, statement body)
statement = Return(exp)
exp = Constant(int)
    | Unary(unary_operator, exp)
    | Binary(binary_operator, exp, exp)
unary_operator = Complement | Negate | Not
binary_operator = Add | Subtract | Multiply | Divide | Remainder | And | Or
    | Equal | NotEqual | LessThan | LessOrEqual
    | GreaterThan | GreaterOrEqual
```

## Formal Grammar
```
<program> ::= <function>
<function> ::= "int" <identifier> "(" "void" ")" "{" <statement> "}"
<statement> ::= "return" <exp> ";"
<exp> ::= <factor> | <exp> <binop> <exp>
<factor> ::= <int> | <unop> <factor> | "(" <exp> ")"
<unop> ::= "-" | "~" | "!"
<binop> ::= "-" | "+" | "*" | "/" | "%" | "&&" | "||"
    | "==" | "!=" | "<" | "<=" | ">" | ">="
<identifier> ::= ? An identifier token ?
<int> ::= ? A constant token ?
```

## TACKY Grammar
```
program = Program(function_definition)
function_definition = Function(identifier, instruction* body)
instruction = Return(val)
    | Unary(unary_operator, val src, val dst)
    | Binary(binary_operator, val src1, val src2, val dst)
    | Copy(val src, val dst)
    | Jump(identifier target)
    | JumpIfZero(val condition, identifier target)
    | JumpIfNotZero(val condition, identifier target)
    | Label(identifier)
val = Constant(int) | Var(identifier)
unary_operator = Complement | Negate | Not
binary_operator = Add | Subtract | Multiply | Divide | Remainder | Equal | NotEqual
    | LessThan | LessOrEqual | GreaterThan | GreaterOrEqual
```

## ASM Grammar
```
program = Program(function_definition)
function_definition = Function(identifier name, instruction* instructions)
instruction = Mov(operand src, operand dst)
| Unary(unary_operator, operand)
| Binary(binary_operator, operand, operand)
| Idiv(operand)
| Cdq
| AllocateStack(int)
| Ret
unary_operator = Neg | Not
binary_operator = Add | Sub | Mult
operand = Imm(int) | Reg(reg) | Pseudo(identifier) | Stack(int)
reg = AX | DX | R10 | R11
```

3 passes:

1. Convert tacky to asm (refers to temp vars directly with Pseudo(identifier))
2. Replace pseudoregisters with concrete addresses in the stack with Stack(int)
3. Fix up instructions so that src and dst operands are not both memory addresses
