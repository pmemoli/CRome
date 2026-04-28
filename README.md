# CRome

C compiler written in Rust based on Sandler Nora's book "Writing a C Compiler". The project is just the the preprocessed C to ASM compiler. Preprocessor and Linker comes from gcc.

After implementing the book's subset of C, the language will be extended with cool stuff like ADTs and pattern matching. The final goal is compiling an simple xv6-like OS.  

Very much a WIP, currently in chapter 9 out of 20.

TODO:

- Chapter 9 codegen.

Backlog:

- Potentially flatten some tacky to asm passes into one function rather than a gazillion.
- Make lexer and parser work with Result's rather than just panicking for errors.

## Lexer

## AST Specification
```
program = Program(function_declaration*)
declaration = FunDecl(function_declaration) | VarDecl(variable_declaration)
variable_declaration = (identifier name, exp? init)
function_declaration = (identifier name, identifier* params, block? body)
block = Block(block_item*)
block_item = S(statement) | D(declaration)
for_init = InitDecl(variable_declaration) | InitExp(exp?)
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
    | FunctionCall(identifier, exp* args)
unary_operator = Complement | Negate | Not
binary_operator = Add | Subtract | Multiply | Divide | Remainder | And | Or
    | Equal | NotEqual | LessThan | LessOrEqual
    | GreaterThan | GreaterOrEqual
```

Loop related statements are annotated in the semantic analysis pass.

## Formal Grammar
```
<program> ::= { <function-declaration> }
<declaration> ::= <variable-declaration> | <function-declaration>
<variable-declaration> ::= "int" <identifier> [ "=" <exp> ] ";"
<function-declaration> ::= "int" <identifier> "(" <param-list> ")" ( <block> | ";")
<param-list> ::= "void" | "int" <identifier> { "," "int" <identifier> }
<block> ::= "{" { <block-item> } "}"
<block-item> ::= <statement> | <declaration>
<declaration> ::= "int" <identifier> [ "=" <exp> ] ";"
<for-init> ::= <variable-declaration> | [ <exp> ] ";"
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
    | <identifier> "(" [ <argument-list> ] ")"
<argument-list> ::= <exp> { "," <exp> }
<unop> ::= "-" | "~" | "!"
<binop> ::= "-" | "+" | "*" | "/" | "%" | "&&" | "||"
    | "==" | "!=" | "<" | "<=" | ">" | ">=" | "="
<identifier> ::= ? An identifier token ?
<int> ::= ? A constant token ?
```

## Semantic Analysis

### First pass (Identifier Resolution):

#### Variables:

1. Map each variable name to a unique value and adds to symbol table 
2. Check that variable assignments have valid left expressions (Var(String))
3. Check that all variables are defined in their scope
4. Check that variable declarations are not repeated in their scope

#### Functions (external linkage):

1. Check that all function calls refer to declared functions in their scope
2. Check that functions and variables are not declared with SAME name in the SAME scope
3. Check that definitions of functions do not live within other functions.

### Second pass (Loop Annotation):

1. Annotate loop nodes in the ast with a unique identifier for each corresponding loop 
2. Check that break and continue statements live within loops 

### Third pass (Type Checking):

1. Check that function declarations are consistent everywhere, and adds name to symbol table
2. A function can't be called with the wrong number of arguments
3. A function can't be defined more than once (not really type checking but easy to implement here)

## TACKY Grammar
```
program = Program(function_definition*)
function_definition = Function(identifier, identifier* params, instruction* body)
instruction = Return(val)
    | Unary(unary_operator, val src, val dst)
    | Binary(binary_operator, val src1, val src2, val dst)
    | Copy(val src, val dst)
    | Jump(identifier target)
    | JumpIfZero(val condition, identifier target)
    | JumpIfNotZero(val condition, identifier target)
    | Label(identifier)
    | FunCall(identifier fun_name, val* args, val dst)
val = Constant(int) | Var(identifier)
unary_operator = Complement | Negate | Not
binary_operator = Add | Subtract | Multiply | Divide | Remainder | Equal | NotEqual
    | LessThan | LessOrEqual | GreaterThan | GreaterOrEqual
```

## ASM Grammar
```
program = Program(function_definition*)
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
    | DeallocateStack(int)
    | Push(operand)
    | Call(identifier)
    | Ret
unary_operator = Neg | Not
binary_operator = Add | Sub | Mult
operand = Imm(int) | Reg(reg) | Pseudo(identifier) | Stack(int)
cond_code = E | NE | G | GE | L | LE
reg = AX | CX | DX | DI | SI | R8 | R9 | R10 | R11
```

### First pass (Tacky to ASM)

Convert Tacky to ASM, without register allocation (using Pseudo(identifier) for variables).

. System V 64 bit calling ABI is implemented in this pass:
    - input/output regs
    - callee/caller saved regs
    - caller handles arg cleanup
    - 16 byte aligned before call.

. Copies arguments to pseudo variables at the start of each function, rather than using the corresponding registers/stack.

### Second pass (Register allocation)

Replace Pseudo(identifier) with Stack(int) for variables, and Reg(reg) for temps. Allocates memory so that each identifier gets its own place in the stack.

### Third pass (Instruction fix up)

Fix up instructions so that src and dst operands are not both memory addresses

