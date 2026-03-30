use crate::{symbol::SymbolTable, tacky};

// program = Program(function_definition)
#[derive(Debug)]
pub struct Program(pub Function);

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
    AllocateStack(u32),
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
    Stack(u32),
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

// reg = AX | DX | R10 | R11
#[derive(Debug, Clone)]
pub enum Reg {
    AX,
    DX,
    R10,
    R11,
}

// First pass: Convert Tacky to ASM AST (with temp variables as pseudoregisters)
pub fn tacky_program_to_asm(tacky_program: &tacky::Program) -> Program {
    let tacky::Program(tacky_function) = tacky_program;
    let asm_function = tacky_function_to_asm(tacky_function);
    Program(asm_function)
}

pub fn tacky_function_to_asm(tacky_function: &tacky::Function) -> Function {
    let tacky::Function(tacky_identifier, tacky_instructions) = tacky_function;

    let identifier = tacky_identifier.to_string();
    let asm_instructions = tacky_instructions
        .into_iter()
        .flat_map(tacky_instruction_to_asm)
        .collect();

    Function(identifier, asm_instructions)
}

pub fn tacky_instruction_to_asm(tacky_function: &tacky::Instruction) -> Vec<Instruction> {
    match tacky_function {
        tacky::Instruction::Return(val) => {
            let src_asm_op = tacky_val_to_asm(val);
            let dst_asm_op = Operand::Reg(Reg::AX);
            vec![Instruction::Mov(src_asm_op, dst_asm_op), Instruction::Ret]
        }

        tacky::Instruction::Unary(unop, src, dst) => {
            let src_asm_op = tacky_val_to_asm(src);
            let dst_asm_op = tacky_val_to_asm(dst);

            match unop {
                tacky::UnaryOperator::Not => {
                    vec![
                        Instruction::Cmp(Operand::Imm(0), src_asm_op),
                        Instruction::Mov(Operand::Imm(0), dst_asm_op.clone()),
                        Instruction::SetCC(CondCode::E, dst_asm_op),
                    ]
                }
                _ => {
                    let unop_asm_op = tacky_unop_to_unop_asm(unop);
                    vec![
                        Instruction::Mov(src_asm_op, dst_asm_op.clone()),
                        Instruction::Unary(unop_asm_op, dst_asm_op),
                    ]
                }
            }
        }
        tacky::Instruction::Binary(op, src_a, src_b, dst) => {
            let src_a_asm_op = tacky_val_to_asm(src_a);
            let src_b_asm_op = tacky_val_to_asm(src_b);
            let dst_asm_op = tacky_val_to_asm(dst);

            match op {
                tacky::BinaryOperator::Divide => {
                    vec![
                        Instruction::Mov(src_a_asm_op, Operand::Reg(Reg::AX)),
                        Instruction::Cdq,
                        Instruction::Idiv(src_b_asm_op),
                        Instruction::Mov(Operand::Reg(Reg::AX), dst_asm_op),
                    ]
                }
                tacky::BinaryOperator::Remainder => {
                    vec![
                        Instruction::Mov(src_a_asm_op, Operand::Reg(Reg::AX)),
                        Instruction::Cdq,
                        Instruction::Idiv(src_b_asm_op),
                        Instruction::Mov(Operand::Reg(Reg::DX), dst_asm_op),
                    ]
                }

                tacky::BinaryOperator::Equal
                | tacky::BinaryOperator::NotEqual
                | tacky::BinaryOperator::LessThan
                | tacky::BinaryOperator::LessOrEqual
                | tacky::BinaryOperator::GreaterThan
                | tacky::BinaryOperator::GreaterOrEqual => {
                    let cond_code = tacky_binop_to_cond_asm(op);
                    vec![
                        Instruction::Cmp(src_b_asm_op, src_a_asm_op),
                        Instruction::Mov(Operand::Imm(0), dst_asm_op.clone()),
                        Instruction::SetCC(cond_code, dst_asm_op),
                    ]
                }

                _ => {
                    let binop_asm_op = tacky_binop_to_binop_asm(op);
                    vec![
                        Instruction::Mov(src_a_asm_op, dst_asm_op.clone()),
                        Instruction::Binary(binop_asm_op, src_b_asm_op, dst_asm_op),
                    ]
                }
            }
        }
        tacky::Instruction::Copy(src, dst) => {
            let src_asm_op = tacky_val_to_asm(src);
            let dst_asm_op = tacky_val_to_asm(dst);
            vec![Instruction::Mov(src_asm_op, dst_asm_op)]
        }
        tacky::Instruction::Jump(label) => vec![Instruction::Jmp(label.to_string())],
        tacky::Instruction::JumpIfZero(cond, label) => {
            let cond_asm_op = tacky_val_to_asm(cond);
            vec![
                Instruction::Cmp(Operand::Imm(0), cond_asm_op),
                Instruction::JmpCC(CondCode::E, label.to_string()),
            ]
        }
        tacky::Instruction::JumpIfNotZero(cond, label) => {
            let cond_asm_op = tacky_val_to_asm(cond);
            vec![
                Instruction::Cmp(Operand::Imm(0), cond_asm_op),
                Instruction::JmpCC(CondCode::NE, label.to_string()),
            ]
        }
        tacky::Instruction::Label(label) => vec![Instruction::Label(label.to_string())],
    }
}

pub fn tacky_val_to_asm(tacky_function: &tacky::Val) -> Operand {
    match tacky_function {
        tacky::Val::Constant(i) => Operand::Imm(*i),
        tacky::Val::Var(s) => Operand::Pseudo(s.to_string()),
    }
}

pub fn tacky_unop_to_unop_asm(tacky_unop: &tacky::UnaryOperator) -> UnaryOperator {
    match tacky_unop {
        tacky::UnaryOperator::Complement => UnaryOperator::Not,
        tacky::UnaryOperator::Negate => UnaryOperator::Neg,
        _ => panic!(
            "Can't convert non-complement/non-negate unary operator to unary operator in codegen"
        ),
    }
}

pub fn tacky_binop_to_binop_asm(tacky_binop: &tacky::BinaryOperator) -> BinaryOperator {
    match tacky_binop {
        tacky::BinaryOperator::Add => BinaryOperator::Add,
        tacky::BinaryOperator::Subtract => BinaryOperator::Sub,
        tacky::BinaryOperator::Multiply => BinaryOperator::Mult,
        _ => panic!("Can't convert non-arithmetic binary operator to binary operator in codegen"),
    }
}

pub fn tacky_binop_to_cond_asm(tacky_binop: &tacky::BinaryOperator) -> CondCode {
    match tacky_binop {
        tacky::BinaryOperator::Equal => CondCode::E,
        tacky::BinaryOperator::NotEqual => CondCode::NE,
        tacky::BinaryOperator::LessThan => CondCode::L,
        tacky::BinaryOperator::LessOrEqual => CondCode::LE,
        tacky::BinaryOperator::GreaterThan => CondCode::G,
        tacky::BinaryOperator::GreaterOrEqual => CondCode::GE,
        _ => panic!("Can't convert non-comparison binary operator to condition code in codegen"),
    }
}

// Second pass: Replace Pseudo(identifier) with Stack(int)
pub fn resolve_pseudo_registers_program(program: &mut Program, symbol_table: &mut SymbolTable) {
    let Program(function) = program;
    resolve_pseudo_registers_function(function, symbol_table);
}

pub fn resolve_pseudo_registers_function(function: &mut Function, symbol_table: &mut SymbolTable) {
    let Function(_, instructions) = function;

    instructions
        .iter_mut()
        .for_each(|i| resolve_pseudo_registers_instruction(i, symbol_table));
}

pub fn resolve_pseudo_registers_instruction(
    instruction: &mut Instruction,
    symbol_table: &mut SymbolTable,
) {
    match instruction {
        Instruction::Mov(src, dst) => {
            resolve_pseudo_registers_operand(src, symbol_table);
            resolve_pseudo_registers_operand(dst, symbol_table);
        }
        Instruction::Unary(_, op) => {
            resolve_pseudo_registers_operand(op, symbol_table);
        }
        Instruction::Binary(_, op_1, op_2) => {
            resolve_pseudo_registers_operand(op_1, symbol_table);
            resolve_pseudo_registers_operand(op_2, symbol_table);
        }
        Instruction::Idiv(op) => {
            resolve_pseudo_registers_operand(op, symbol_table);
        }
        Instruction::Cmp(op_1, op_2) => {
            resolve_pseudo_registers_operand(op_1, symbol_table);
            resolve_pseudo_registers_operand(op_2, symbol_table);
        }
        Instruction::SetCC(_, op) => {
            resolve_pseudo_registers_operand(op, symbol_table);
        }
        _ => {}
    }
}

pub fn resolve_pseudo_registers_operand(operand: &mut Operand, symbol_table: &mut SymbolTable) {
    match operand {
        Operand::Pseudo(s) => {
            let symbol_info = symbol_table.get(s);
            let stack_offset = symbol_info.stack_offset;
            *operand = Operand::Stack(stack_offset);
        }
        _ => {}
    }
}

// Third pass: Allocate stack and fix instruction operands
pub fn instruction_fixup_program(program: &mut Program, symbol_table: &mut SymbolTable) {
    let Program(function) = program;
    instruction_fixup_function(function, symbol_table);
}

pub fn instruction_fixup_function(function: &mut Function, symbol_table: &mut SymbolTable) {
    let Function(_, instructions) = function;

    let stack_size = SymbolTable::stack_size(symbol_table);

    let mut allocated_instructions = vec![Instruction::AllocateStack(stack_size)];
    let fixed_instructions = instructions
        .into_iter()
        .flat_map(instruction_fixup_instruction);
    allocated_instructions.extend(fixed_instructions);

    *instructions = allocated_instructions;
}

pub fn instruction_fixup_instruction(instruction: &mut Instruction) -> Vec<Instruction> {
    match instruction {
        Instruction::Mov(src @ Operand::Stack(_), dst @ Operand::Stack(_)) => {
            vec![
                Instruction::Mov(src.clone(), Operand::Reg(Reg::R10)),
                Instruction::Mov(Operand::Reg(Reg::R10), dst.clone()),
            ]
        }

        Instruction::Idiv(op @ Operand::Imm(_)) => {
            vec![
                Instruction::Mov(op.clone(), Operand::Reg(Reg::R10)),
                Instruction::Idiv(Operand::Reg(Reg::R10)),
            ]
        }

        Instruction::Binary(
            binop @ BinaryOperator::Add | binop @ BinaryOperator::Sub,
            src @ Operand::Stack(_),
            dst @ Operand::Stack(_),
        ) => {
            vec![
                Instruction::Mov(src.clone(), Operand::Reg(Reg::R10)),
                Instruction::Binary(binop.clone(), Operand::Reg(Reg::R10), dst.clone()),
            ]
        }

        Instruction::Binary(BinaryOperator::Mult, src, dst @ Operand::Stack(_)) => {
            vec![
                Instruction::Mov(dst.clone(), Operand::Reg(Reg::R11)),
                Instruction::Binary(BinaryOperator::Mult, src.clone(), Operand::Reg(Reg::R11)),
                Instruction::Mov(Operand::Reg(Reg::R11), dst.clone()),
            ]
        }

        Instruction::Cmp(src @ Operand::Stack(_), dst @ Operand::Stack(_)) => {
            vec![
                Instruction::Mov(src.clone(), Operand::Reg(Reg::R10)),
                Instruction::Cmp(Operand::Reg(Reg::R10), dst.clone()),
            ]
        }

        Instruction::Cmp(src, dst @ Operand::Imm(_)) => {
            vec![
                Instruction::Mov(dst.clone(), Operand::Reg(Reg::R11)),
                Instruction::Cmp(src.clone(), Operand::Reg(Reg::R11)),
            ]
        }

        i => vec![i.clone()],
    }
}

// ASM gen wrapper for each pass
pub fn codegen_program(program: &tacky::Program, symbol_table: &mut SymbolTable) -> Program {
    let mut asm_program = tacky_program_to_asm(program);
    resolve_pseudo_registers_program(&mut asm_program, symbol_table);
    instruction_fixup_program(&mut asm_program, symbol_table);

    asm_program
}
