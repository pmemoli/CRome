use crate::{symbol::SymbolTable, tacky};

mod instruction_fixup;
mod register_allocation;
mod tacky_to_asm;

// program = Program(function_definition*)
#[derive(Debug)]
pub struct Program(pub Vec<Function>);

// function_definition = Function(identifier name, instruction* instructions)
#[derive(Debug)]
pub struct Function(pub String, pub Vec<Instruction>);

// instruction = Mov(operand src, operand dst)
//     | Unary(unary_operator, operand)
//     | Binary(binary_operator, operand, operand)
//     | Cmp(operand, operand)
//     | Idiv(operand)
//     | Cdq
//     | Jmp(identifier)
//     | JmpCC(cond_code, identifier)
//     | SetCC(cond_code, operand)
//     | Label(identifier)
//     | AllocateStack(int)
//     | DeallocateStack(int)
//     | Push(operand)
//     | Call(identifier)
//     | Ret
#[derive(Debug, Clone)]
pub enum Instruction {
    Mov(Operand, Operand),
    Unary(UnaryOperator, Operand),
    Binary(BinaryOperator, Operand, Operand),
    Cmp(Operand, Operand),
    Idiv(Operand),
    Cdq,
    Jmp(String),
    JmpCC(CondCode, String),
    SetCC(CondCode, Operand),
    Label(String),
    AllocateStack(usize),
    DeallocateStack(usize),
    Push(Operand),
    Call(String),
    Ret,
}

// unary_operator = Neg | Not
#[derive(Debug, Clone)]
pub enum UnaryOperator {
    Neg,
    Not,
}

// binary_operator = Add | Sub | Mult
#[derive(Debug, Clone)]
pub enum BinaryOperator {
    Add,
    Sub,
    Mult,
}

// operand = Imm(int) | Reg(reg) | Pseudo(identifier) | Stack(int)
#[derive(Debug, Clone)]
pub enum Operand {
    Imm(i32),
    Reg(Reg),
    Pseudo(String),
    Stack(isize),
}

// cond_code = E | NE | G | GE | L | LE
#[derive(Debug, Clone)]
pub enum CondCode {
    E,
    NE,
    G,
    GE,
    L,
    LE,
}

// reg = AX | CX | DX | DI | SI | R8 | R9 | R10 | R11
#[derive(Debug, Clone)]
pub enum Reg {
    AX,
    CX,
    DX,
    DI,
    SI,
    R8,
    R9,
    R10,
    R11,
}

// ASM codegen wrapper
pub fn codegen_program(program: &tacky::Program, symbol_table: &SymbolTable) -> Program {
    let asm_program = tacky_to_asm::tacky_program_to_asm(program);
    let asm_program =
        register_allocation::resolve_pseudo_registers_program(&asm_program, symbol_table);
    let asm_program = instruction_fixup::instruction_fixup_program(&asm_program, symbol_table);

    asm_program
}
