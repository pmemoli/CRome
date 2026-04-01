C compiler written in Rust based on Sandler Nora's book "Writing a C Compiler". The project is just the the preprocessed C to ASM compiler. Preprocessor and Linker comes from gcc. 

Very much a WIP, currently in chapter 9 out of 20.

TODO:

- Chapter 9.
- Refactor the codegen into three files, its kinda big now.
- The label counter should be local to the codegen pass, not part of symbol table.

Backlog:

- Potentially flatten some tacky to asm passes into one function rather than a gazillion.
- Make lexer and parser work with Result's rather than just panicking for errors.

# Compiler passes

## Lexer

## AST Specification
```
program = Program(function_definition)
function_definition = Function(identifier name, block body)
block = Block(block_item*)
block_item = S(statement) | D(declaration)
declaration = Declaration(identifier name, exp? init)
for_init = InitDecl(declaration) | InitExp(exp?)
statement = Return(exp)
    | Expression(exp)
    | If(exp condition, statement then, statement? else)
    | Compound(block)
    | Break(identifier label)
    | Continue(identifier label)
    | While(exp condition, statement body, identifier label)
    | DoWhile(statement body, exp condition, identifier label)
    | For(for_init init, exp? condition, exp? post, statement body, identifier label)
    | Null
exp = Constant(int)
    | Var(identifier)
    | Unary(unary_operator, exp)
    | Binary(binary_operator, exp, exp)
    | Assignment(exp, exp)
    | Conditional(exp condition, exp, exp)
unary_operator = Complement | Negate | Not
binary_operator = Add | Subtract | Multiply | Divide | Remainder | And | Or
    | Equal | NotEqual | LessThan | LessOrEqual
    | GreaterThan | GreaterOrEqual
```

Loop related statements are annotated in the semantic analysis pass.

## Formal Grammar
```
<program> ::= <function>
<function> ::= "int" <identifier> "(" "void" ")" <block>
<block> ::= "{" { <block-item> } "}"
<block-item> ::= <statement> | <declaration>
<declaration> ::= "int" <identifier> [ "=" <exp> ] ";"
<for-init> ::= <declaration> | [ <exp> ] ";"
<statement> ::= "return" <exp> ";"
    | <exp> ";"
    | "if" "(" <exp> ")" <statement> [ "else" <statement> ]
    | <block>
    | "break" ";"
    | "continue" ";"
    | "while" "(" <exp> ")" <statement>
    | "do" <statement> "while" "(" <exp> ")" ";"
    | "for" "(" <for-init> [ <exp> ] ";" [ <exp> ] ")" <statement>
    | ";"
<exp> ::= <factor> | <exp> <binop> <exp> | <exp> "?" <exp> ":" <exp>
<factor> ::= <int> | <identifier> | <unop> <factor> | "(" <exp> ")"
<unop> ::= "-" | "~" | "!"
<binop> ::= "-" | "+" | "*" | "/" | "%" | "&&" | "||"
    | "==" | "!=" | "<" | "<=" | ">" | ">=" | "="
<identifier> ::= ? An identifier token ?
<int> ::= ? A constant token ?
```

## Semantic Analysis

First pass (Variable Resolution):
1. Map each variable name to a unique value
2. Check that assignments have valid left expressions (Var(String))
3. Check that all variables are defined in their scope
4. Check that variable declarations are not repeated in their scope

Second pass (Loop Annotation):
1. Annotate loop nodes in the ast with a unique identifier for each corresponding loop 
2. Check that break and continue statements live within loops 

## TACKY Grammar
```
program = Program(function_definition)
function_definition = Function(identifier name, instruction* instructions)
instruction = Mov(operand src, operand dst)
    | Unary(unary_operator, operand)
    | Binary(binary_operator, operand, operand)
    | Cmp(operand, operand)
    | Idiv(operand)
    | Cdq
    | Jmp(identifier)
    | JmpCC(cond_code, identifier)
    | SetCC(cond_code, operand)
    | Label(identifier)
    | AllocateStack(int)
    | Ret
unary_operator = Neg | Not
binary_operator = Add | Sub | Mult
operand = Imm(int) | Reg(reg) | Pseudo(identifier) | Stack(int)
cond_code = E | NE | G | GE | L | LE
reg = AX | DX | R10 | R11
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
