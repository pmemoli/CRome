# CRome

C compiler written in Rust based on Sandler Nora's book "Writing a C Compiler". The project is just the preprocessed C to ASM compiler. Preprocessor and Linker comes from gcc.

After implementing the book's subset of C, the language will be extended with cool stuff like:

- ADTs and pattern matching
- Go-like methods for structs
- A simple rust like module system
- SSA for optimizations

And a bunch of extra optimizations.

The final goal is writing and compiling a simple xv6-like OS (RomeOS).  

Currently in chapter 11 / 20, finished part 1.

TODO:

- Chapter 11 parser.

Backlog:

- Proper error reporting system, currently just panics with a message and a backtrace.

## Lexer

## AST Specification
```
program = Program(declaration*)
declaration = FunDecl(function_declaration) | VarDecl(variable_declaration)
variable_declaration = (identifier name, exp? init, type var_type, storage_class?)
function_declaration = (identifier name, identifier* params, block? body, type fun_type, storage_class?)
type = Int | Long | FunType(type* params, type ret)
storage_class = Static | Extern
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
exp = Constant(const)
    | Var(identifier)
    | Cast(type target_type, exp)
    | Unary(unary_operator, exp)
    | Binary(binary_operator, exp, exp)
    | Assignment(exp, exp)
    | Conditional(exp condition, exp, exp)
    | FunctionCall(identifier, exp* args)
unary_operator = Complement | Negate | Not
binary_operator = Add | Subtract | Multiply | Divide | Remainder | And | Or
    | Equal | NotEqual | LessThan | LessOrEqual
    | GreaterThan | GreaterOrEqual
const = ConstInt(int) | ConstLong(int)
```

Loop related statements are annotated in the semantic analysis pass.

## Formal Grammar
```
<program> ::= { <declaration> }
<declaration> ::= <variable-declaration> | <function-declaration>
<variable-declaration> ::= { <specifier> }+ <identifier> [ "=" <exp> ] ";"
<function-declaration> ::= { <specifier> }+ <identifier> "(" <param-list> ")" ( <block> | ";")
<param-list> ::= "void"
    | { <type-specifier> }+ <identifier> { "," { <type-specifier> }+ <identifier> }
<type-specifier> ::= "int" | "long"
<specifier> ::= <type-specifier> | "static" | "extern"
<block> ::= "{" { <block-item> } "}"
<block-item> ::= <statement> | <declaration>
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
<factor> ::= <const> | <identifier>
    | "(" { <type-specifier> }+ ")" <factor>
    | <unop> <factor> | "(" <exp> ")"
    | <identifier> "(" [ <argument-list> ] ")"
<argument-list> ::= <exp> { "," <exp> }
<unop> ::= "-" | "~" | "!"
<binop> ::= "-" | "+" | "*" | "/" | "%" | "&&" | "||"
    | "==" | "!=" | "<" | "<=" | ">" | ">=" | "="
<const> ::= <int> | <long>
<identifier> ::= ? An identifier token ?
<int> ::= ? An int token ?
<long> ::= ? An int or long token ?
```

## Semantic Analysis

### Linkage and Storage rules for variables and functions:

Tables from Writing a C Compiler, Pages 216-217.

#### Variable Declarations

| Scope | Specifier | Linkage | Storage Duration | With Initializer | Without Initializer |
|-------|-----------|---------|------------------|------------------|---------------------|
| File scope | None | External | Static | Yes | Tentative |
| File scope | `static` | Internal | Static | Yes | Tentative |
| File scope | `extern` | Matches prior visible declaration; external by default | Static | Yes | No |
| Block scope | None | None | Automatic | Yes | Yes (defined but uninitialized) |
| Block scope | `static` | None | Static | Yes | Yes (initialized to zero) |
| Block scope | `extern` | Matches prior visible declaration; external by default | Static | Invalid | No |

#### Function Declarations

| Scope | Specifier | Linkage | With Body | Without Body |
|-------|-----------|---------|-----------|--------------|
| File scope | None or `extern` | Matches prior visible declaration; external by default | Yes | No |
| File scope | `static` | Internal | Yes | No |
| Block scope | None or `extern` | Matches prior visible declaration; external by default | Invalid | No |
| Block scope | `static` | Invalid | Invalid | Invalid |

### First pass (Identifier Resolution):

#### Variables:

1. Rename each non-linked variable name to a unique one.
2. Check that variable assignments have valid left expressions (Var(String))
3. Check that all variables in expressions are declared
4. Check that local variables are not redeclared in the current scope

#### Functions:

1. Check that all function calls refer to declared identifiers
2. Check that definitions of functions do not live within blocks

#### Both

Check that identifier declarations do not contradict in having or not having linkage

### Second pass (Loop Annotation):

1. Annotate loop nodes in the ast with a unique identifier for each corresponding loop 
2. Check that break and continue statements live within loops 

### Third pass (Type Checking):

1. Check that function declarations are consistent everywhere, and adds name to symbol table
2. A function can't be called with the wrong number of arguments
3. A function can't be defined more than once (not really type checking but easy to implement here)

## TACKY Grammar
```
program = Program(top_level*)
top_level = Function(identifier, bool global, identifier* params, instruction* body)
    | StaticVariable(identifier, bool global, int init)
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
program = Program(top_level*)
top_level = Function(identifier name, bool global, instruction* instructions)
    | StaticVariable(identifier name, bool global, int init)
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
operand = Imm(int) | Reg(reg) | Pseudo(identifier) | Stack(int) | Data(identifier)
cond_code = E | NE | G | GE | L | LE
reg = AX | CX | DX | DI | SI | R8 | R9 | R10 | R11
```

. System V 64 bit calling ABI is implemented in this pass:
    - input/output regs
    - callee/caller saved regs
    - caller handles arg cleanup
    - 16 byte aligned before call.
    - .text, .data, .rodata and .bss semantics are implemented

### First pass (Tacky to ASM)

1. Convert Tacky to ASM, without register allocation (using Pseudo(identifier) for variables).
2. Copies arguments to pseudo variables at the start of each function, rather than using the corresponding registers/stack.

### Second pass (Register allocation)

1. Replace Pseudo(identifier) with Stack(int) for variables, and Reg(reg) for temps. 
2. Allocate stack space for each function, rounded up to a multiple of 16 bytes for alignment.

### Third pass (Instruction fix up)

1. Fix up instructions so that src and dst operands are not both memory addresses

## Code Emission

### External Linkage

If an identifier has external linkage, we emit it with a .globl directive. Otherwise we don't add the directive. Linker resolves external symbols at link time. 

To work with dynamic libraries, we use call @PLT for functions when the definition is not present in the translation unit, and a regular call otherwise.

### Internal Linkage

Internally linked identifiers are accessed through RIP offsets. This makes the program position independent, and implements internal linkage at compile time.

### No Linkage

Handled through the stack

### Storage Duration

- .data holds static storage duration objects with non-zero initializers.
- .bss holds static storage duration objects with zero initializers.

Automatic storage duration objects are allocated on the stack at runtime.
