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

    let mut asm_instructions = Vec::new();

    // Passes arguments as pseudovariables to later clobber caller saved registers freely
    let reg_order = vec![Reg::DI, Reg::SI, Reg::DX, Reg::CX, Reg::R8, Reg::R9];
    for i in 0..tacky_arguments.len() {
        let arg = tacky_arguments[i].to_string();

        if i < reg_order.len() {
            let src = Operand::Reg(reg_order[i].clone());
            asm_instructions.push(Instruction::Mov(src, Operand::Pseudo(arg)));
        } else {
            let j = i - reg_order.len();
            asm_instructions.push(Instruction::Mov(
                Operand::Stack(8 * (j + 2) as isize), // First arg is at RSP + 16 (old RBP + ret address)
                Operand::Pseudo(arg),
            ));
        }
    }

    for instruction in tacky_instructions {
        let mut asm_instrs = tacky_instruction_to_asm(instruction);
        asm_instructions.append(&mut asm_instrs);
    }

    let identifier = tacky_identifier.to_string();

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
        tacky::Instruction::FunCall(identifier, args, dst) => {
            let mut instructions = Vec::new();

            let reg_order = vec![Reg::DI, Reg::SI, Reg::DX, Reg::CX, Reg::R8, Reg::R9];

            // Add padding to ensure stack is 16-byte aligned before call instruction
            let stack_args = (args.len() as isize - reg_order.len() as isize).max(0) as usize;
            let mut stack_padding = 0;
            if stack_args % 2 == 1 {
                stack_padding = 8;
                instructions.push(Instruction::AllocateStack(stack_padding));
            }

            // Pass args according to ABI
            for i in 0..args.len() {
                if i < reg_order.len() {
                    let asm_arg = tacky_val_to_asm(&args[i]);
                    let dst = Operand::Reg(reg_order[i].clone());
                    instructions.push(Instruction::Mov(asm_arg, dst));
                } else {
                    // We push stack arguments in reverse order
                    let stack_arg_number = i - reg_order.len();
                    let asm_arg = tacky_val_to_asm(&args[args.len() - 1 - stack_arg_number]);

                    instructions.push(Instruction::Push(asm_arg));
                }
            }

            // Call function
            instructions.push(Instruction::Call(identifier.to_string()));

            // Cleanup arguments
            let stack_arguments = (args.len() as isize - reg_order.len() as isize).max(0) as usize;
            let bytes_to_cleanup = stack_arguments * 8 + stack_padding;
            if bytes_to_cleanup > 0 {
                instructions.push(Instruction::DeallocateStack(bytes_to_cleanup));
            }

            // Retrieve return value
            let dst_asm_op = tacky_val_to_asm(dst);
            instructions.push(Instruction::Mov(Operand::Reg(Reg::AX), dst_asm_op));

            instructions
        }
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
