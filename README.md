C compiler written in Rust based on Sandler Nora's book "Writing a C Compiler". The project is just the the preprocessed C to ASM compiler. Preprocessor and Linker comes from gcc. 

Very much a WIP.

TODO:
- Abstract the three asm passes into one.
- Mutate the asm ast rather than create new ones for each pass.
- Write the tacky to asm codegen

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
exp = Constant(int) | Unary(unary_operator, exp)
unary_operator = Complement | Negate
```

## Formal Grammar
```
<program> ::= <function>
<function> ::= "int" <identifier> "(" "void" ")" "{" <statement> "}"
<statement> ::= "return" <exp> ";"
<exp> ::= <int> | <unop> <exp> | "(" <exp> ")"
<unop> ::= "-" | "~"
<identifier> ::= ? An identifier token ?
<int> ::= ? A constant token ?
```

## TACKY Grammar
```
program = Program(function_definition)
function_definition = Function(identifier, instruction* body)
instruction = Return(val) | Unary(unary_operator, val src, val dst)
val = Constant(int) | Var(identifier)
unary_operator = Complement | Negate
```

## ASM Grammar
```
program = Program(function_definition)
function_definition = Function(identifier name, instruction* instructions)
instruction = Mov(operand src, operand dst)
| Unary(unary_operator, operand)
| AllocateStack(int)
| Ret
unary_operator = Neg | Not
operand = Imm(int) | Reg(reg) | Pseudo(identifier) | Stack(int)
reg = AX | R10
```

3 passes: 

1. Convert tacky to asm (refers to temp vars directly with Pseudo(identifier))
2. Replace pseudoregisters with concrete addresses in the stack with Stack(int) 
3. Fix up instructions so that src and dst operands are not both memory addresses
