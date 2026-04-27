use super::*;
use crate::tacky;

// First pass: Convert Tacky to ASM AST (with temp variables as pseudoregisters)
pub fn tacky_program_to_asm(tacky_program: &tacky::Program) -> Program {
    let tacky::Program(tacky_functions) = tacky_program;

    let mut asm_functions = Vec::new();
    for tacky_function in tacky_functions {
        asm_functions.push(tacky_function_to_asm(tacky_function));
    }
    Program(asm_functions)
}

pub fn tacky_function_to_asm(tacky_function: &tacky::Function) -> Function {
    let tacky::Function(tacky_identifier, tacky_arguments, tacky_instructions) = tacky_function;

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
