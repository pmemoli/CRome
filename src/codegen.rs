use crate::{
    symbol::{AssemblyType, BackendSymbolTable, StaticInit, SymbolTable},
    tacky,
};

mod instruction_fixup;
mod register_allocation;
mod tacky_to_asm;

// program = Program(top_level*)
#[derive(Debug)]
pub struct Program(pub Vec<TopLevel>);

// top_level = Function(identifier name, bool global, instruction* instructions)
//     | StaticVariable(identifier name, bool global, int alignment, static_init init)
//     | StaticConstant(identifier name, int alignment, static_init init)
#[derive(Debug, Clone)]
pub enum TopLevel {
    Function(String, bool, Vec<Instruction>),
    StaticVariable(String, bool, usize, StaticInit),
    StaticConstant(String, usize, StaticInit),
}

// instruction = Mov(assembly_type, operand src, operand dst)
//     | Movsx(operand src, operand dst)
//     | MovZeroExtend(operand src, operand dst)
//     | Cvttsd2si(assembly_type dst_type, operand src, operand dst)
//     | Cvtsi2sd(assembly_type src_type, operand src, operand dst)
//     | Vcvttsd2usi(assembly_type dst_type, operand src, operand dst)
//     | Vcvtusi2sd(assembly_type dst_type, operand src, operand dst)
//     | Unary(unary_operator, assembly_type, operand)
//     | Binary(binary_operator, assembly_type, operand, operand)
//     | Cmp(assembly_type, operand, operand)
//     | Idiv(assembly_type, operand)
//     | Div(assembly_type, operand)
//     | Cdq(assembly_type)
//     | Jmp(identifier)
//     | JmpCC(cond_code, identifier)
//     | SetCC(cond_code, operand)
//     | Label(identifier)
//     | Push(operand)
//     | Call(identifier)
//     | Ret
#[derive(Debug, Clone)]
pub enum Instruction {
    Mov(AssemblyType, Operand, Operand),
    Movsx(Operand, Operand),
    MovZeroExtend(Operand, Operand),
    Cvttsd2si(AssemblyType, Operand, Operand), // Double to Int/Long
    Cvtsi2sd(AssemblyType, Operand, Operand),  // Int/Long to Double
    Vcvttsd2usi(AssemblyType, Operand, Operand), // Double to Unsigned Int/Long
    Vcvtusi2sd(AssemblyType, Operand, Operand), // Unsigned Int/Long to Double
    Unary(UnaryOperator, AssemblyType, Operand),
    Binary(BinaryOperator, AssemblyType, Operand, Operand),
    Cmp(AssemblyType, Operand, Operand),
    Idiv(AssemblyType, Operand),
    Div(AssemblyType, Operand),
    Cdq(AssemblyType),
    Jmp(String),
    JmpCC(CondCode, String),
    SetCC(CondCode, Operand),
    Label(String),
    Push(Operand),
    Call(String),
    Ret,
}

// unary_operator = Neg | Not | Shr
#[derive(Debug, Clone)]
pub enum UnaryOperator {
    Neg,
    Not,
    Shr,
}

// binary_operator = Add | Sub | Mult | DivDouble | And | Or | Xor
#[derive(Debug, Clone)]
pub enum BinaryOperator {
    Add,
    Sub,
    Mult,
    DivDouble,
    And,
    Or,
    Xor,
}

// operand = Imm(int) | Reg(reg) | Pseudo(identifier) | Stack(int) | Data(identifier)
#[derive(Debug, Clone)]
pub enum Operand {
    Imm(i128),
    Reg(Reg),
    Pseudo(String),
    Stack(isize),
    Data(String),
}

impl Operand {
    pub fn is_memory_operand(&self) -> bool {
        matches!(self, Operand::Stack(_) | Operand::Data(_))
    }

    pub fn is_large_imm_operand(&self) -> bool {
        if let Operand::Imm(i) = self {
            let converted: Result<i32, _> = (*i).try_into();
            !converted.is_ok()
        } else {
            false
        }
    }
}

// cond_code = E | NE | G | GE | L | LE | A | AE | B | BE
#[derive(Debug, Clone)]
pub enum CondCode {
    E,
    NE,
    G,
    GE,
    L,
    LE,
    A,
    AE,
    B,
    BE,
}

// reg = AX | CX | DX | DI | SI | R8 | R9 | R10 | R11 | SP
//     | XMM0 | XMM1 | XMM2 | XMM3 | XMM4 | XMM5 | XMM6 | XMM7 | XMM14 | XMM15
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
    SP,
    XMM0,
    XMM1,
    XMM2,
    XMM3,
    XMM4,
    XMM5,
    XMM6,
    XMM7,
    XMM14,
    XMM15,
}

// ASM codegen wrapper
pub fn codegen_program(program: &tacky::Program, symbol_table: &SymbolTable) -> Program {
    let asm_program = tacky_to_asm::tacky_program_to_asm(program, symbol_table);
    // let backend_symbol_table = BackendSymbolTable::new(symbol_table.clone());
    // let asm_program =
    //     register_allocation::resolve_pseudo_registers_program(&asm_program, &backend_symbol_table);
    // let asm_program = instruction_fixup::instruction_fixup_program(&asm_program);

    asm_program
}
