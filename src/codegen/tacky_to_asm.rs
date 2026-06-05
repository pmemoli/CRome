use super::*;
use crate::parser::Const;
use crate::symbol::Type;
use crate::tacky;

// First pass: Convert Tacky to ASM AST (with temp variables as pseudoregisters)
pub fn tacky_program_to_asm(tacky_program: &tacky::Program, symbol_table: &SymbolTable) -> Program {
    let tacky::Program(tacky_top_level) = tacky_program;

    let mut asm_top_level = Vec::new();
    for tacky_top_object in tacky_top_level {
        asm_top_level.push(tacky_top_level_to_asm(tacky_top_object, symbol_table));
    }
    Program(asm_top_level)
}

pub fn tacky_top_level_to_asm(
    tacky_top_level: &tacky::TopLevel,
    symbol_table: &SymbolTable,
) -> TopLevel {
    match tacky_top_level {
        tacky::TopLevel::Function(
            tacky_identifier,
            tacky_global,
            tacky_arguments,
            tacky_instructions,
        ) => {
            let mut asm_instructions = Vec::new();

            // Passes arguments as pseudovariables to later clobber caller saved registers freely
            let reg_order = vec![Reg::DI, Reg::SI, Reg::DX, Reg::CX, Reg::R8, Reg::R9];
            for i in 0..tacky_arguments.len() {
                let arg = tacky_arguments[i].to_string();

                let arg_type = symbol_table.identifier_type(&arg).unwrap();
                let arg_asm_type = symbol_type_to_asm_type(arg_type);

                if i < reg_order.len() {
                    let src = Operand::Reg(reg_order[i].clone());
                    asm_instructions.push(Instruction::Mov(
                        arg_asm_type,
                        src,
                        Operand::Pseudo(arg),
                    ));
                } else {
                    let j = i - reg_order.len();
                    asm_instructions.push(Instruction::Mov(
                        arg_asm_type,
                        Operand::Stack(8 * (j + 2) as isize), // First arg is at RSP + 16 (old RBP + ret address)
                        Operand::Pseudo(arg),
                    ));
                }
            }

            for instruction in tacky_instructions {
                let mut asm_instrs = tacky_instruction_to_asm(instruction, symbol_table);
                asm_instructions.append(&mut asm_instrs);
            }

            let identifier = tacky_identifier.to_string();

            TopLevel::Function(identifier, *tacky_global, asm_instructions)
        }
        tacky::TopLevel::StaticVariable(identifier, global, ty, init) => {
            let alignment = match symbol_type_to_asm_type(ty) {
                AssemblyType::Longword => 4,
                AssemblyType::Quadword => 8,
            };

            TopLevel::StaticVariable(identifier.to_string(), *global, alignment, init.clone())
        }
    }
}

pub fn tacky_instruction_to_asm(
    tacky_function: &tacky::Instruction,
    symbol_table: &SymbolTable,
) -> Vec<Instruction> {
    match tacky_function {
        tacky::Instruction::Return(val) => {
            let val_asm_type = tacky_value_type(val, symbol_table);
            let src_asm_op = tacky_val_to_asm_operand(val);
            let dst_asm_op = Operand::Reg(Reg::AX);
            vec![
                Instruction::Mov(val_asm_type, src_asm_op, dst_asm_op),
                Instruction::Ret,
            ]
        }

        tacky::Instruction::Unary(unop, src, dst) => {
            let src_asm_type = tacky_value_type(src, symbol_table);
            let dst_asm_type = tacky_value_type(dst, symbol_table);
            let src_asm_op = tacky_val_to_asm_operand(src);
            let dst_asm_op = tacky_val_to_asm_operand(dst);

            match unop {
                tacky::UnaryOperator::Not => {
                    vec![
                        Instruction::Cmp(src_asm_type, Operand::Imm(0), src_asm_op),
                        Instruction::Mov(dst_asm_type, Operand::Imm(0), dst_asm_op.clone()),
                        Instruction::SetCC(CondCode::E, dst_asm_op),
                    ]
                }
                _ => {
                    let unop_asm_op = tacky_unop_to_unop_asm(unop);
                    vec![
                        Instruction::Mov(src_asm_type.clone(), src_asm_op, dst_asm_op.clone()),
                        Instruction::Unary(unop_asm_op, src_asm_type, dst_asm_op),
                    ]
                }
            }
        }
        tacky::Instruction::Binary(op, src_a, src_b, dst) => {
            let src_a_asm_type = tacky_value_type(src_a, symbol_table);
            let dst_asm_type = tacky_value_type(dst, symbol_table);
            let src_a_asm_op = tacky_val_to_asm_operand(src_a);
            let src_b_asm_op = tacky_val_to_asm_operand(src_b);
            let dst_asm_op = tacky_val_to_asm_operand(dst);

            match op {
                tacky::BinaryOperator::Divide => {
                    vec![
                        Instruction::Mov(
                            src_a_asm_type.clone(),
                            src_a_asm_op,
                            Operand::Reg(Reg::AX),
                        ),
                        Instruction::Cdq(src_a_asm_type.clone()),
                        Instruction::Idiv(src_a_asm_type.clone(), src_b_asm_op),
                        Instruction::Mov(src_a_asm_type, Operand::Reg(Reg::AX), dst_asm_op),
                    ]
                }
                tacky::BinaryOperator::Remainder => {
                    vec![
                        Instruction::Mov(
                            src_a_asm_type.clone(),
                            src_a_asm_op,
                            Operand::Reg(Reg::AX),
                        ),
                        Instruction::Cdq(src_a_asm_type.clone()),
                        Instruction::Idiv(src_a_asm_type.clone(), src_b_asm_op),
                        Instruction::Mov(src_a_asm_type, Operand::Reg(Reg::DX), dst_asm_op),
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
                        Instruction::Cmp(src_a_asm_type, src_b_asm_op, src_a_asm_op),
                        Instruction::Mov(dst_asm_type, Operand::Imm(0), dst_asm_op.clone()),
                        Instruction::SetCC(cond_code, dst_asm_op),
                    ]
                }

                _ => {
                    let binop_asm_op = tacky_binop_to_binop_asm(op);
                    vec![
                        Instruction::Mov(src_a_asm_type.clone(), src_a_asm_op, dst_asm_op.clone()),
                        Instruction::Binary(binop_asm_op, src_a_asm_type, src_b_asm_op, dst_asm_op),
                    ]
                }
            }
        }
        tacky::Instruction::Copy(src, dst) => {
            let src_asm_type = tacky_value_type(src, symbol_table);
            let src_asm_op = tacky_val_to_asm_operand(src);
            let dst_asm_op = tacky_val_to_asm_operand(dst);
            vec![Instruction::Mov(src_asm_type, src_asm_op, dst_asm_op)]
        }
        tacky::Instruction::Jump(label) => vec![Instruction::Jmp(label.to_string())],
        tacky::Instruction::JumpIfZero(cond, label) => {
            let cond_asm_type = tacky_value_type(cond, symbol_table);
            let cond_asm_op = tacky_val_to_asm_operand(cond);
            vec![
                Instruction::Cmp(cond_asm_type, Operand::Imm(0), cond_asm_op),
                Instruction::JmpCC(CondCode::E, label.to_string()),
            ]
        }
        tacky::Instruction::JumpIfNotZero(cond, label) => {
            let cond_asm_type = tacky_value_type(cond, symbol_table);
            let cond_asm_op = tacky_val_to_asm_operand(cond);
            vec![
                Instruction::Cmp(cond_asm_type, Operand::Imm(0), cond_asm_op),
                Instruction::JmpCC(CondCode::NE, label.to_string()),
            ]
        }
        tacky::Instruction::Label(label) => vec![Instruction::Label(label.to_string())],
        tacky::Instruction::FunCall(..) => tacky_fun_call_to_asm(tacky_function, symbol_table),
        tacky::Instruction::SignExtend(src, dst) => {
            let asm_src = tacky_val_to_asm_operand(src);
            let asm_dst = tacky_val_to_asm_operand(dst);
            vec![Instruction::Movsx(asm_src, asm_dst)]
        }
        tacky::Instruction::Truncate(src, dst) => {
            let asm_src = tacky_val_to_asm_operand(src);
            let asm_dst = tacky_val_to_asm_operand(dst);
            vec![Instruction::Mov(AssemblyType::Longword, asm_src, asm_dst)]
        }
    }
}

pub fn tacky_fun_call_to_asm(
    tacky_fun_call: &tacky::Instruction,
    symbol_table: &SymbolTable,
) -> Vec<Instruction> {
    let tacky::Instruction::FunCall(identifier, args, dst) = tacky_fun_call else {
        panic!("Expected a function call instruction");
    };

    let mut instructions = Vec::new();

    let reg_order = vec![Reg::DI, Reg::SI, Reg::DX, Reg::CX, Reg::R8, Reg::R9];

    // Add padding to ensure stack is 16-byte aligned before call instruction
    let stack_args = (args.len() as isize - reg_order.len() as isize).max(0) as usize;
    let mut stack_padding = 0;
    if stack_args % 2 == 1 {
        stack_padding = 8;
        instructions.push(Instruction::Binary(
            BinaryOperator::Sub,
            AssemblyType::Quadword,
            Operand::Imm(stack_padding),
            Operand::Reg(Reg::SP),
        ))
    }

    // Pass args according to ABI
    for i in 0..args.len() {
        if i < reg_order.len() {
            let asm_arg_type = tacky_value_type(&args[i], symbol_table);
            let asm_arg = tacky_val_to_asm_operand(&args[i]);
            let dst = Operand::Reg(reg_order[i].clone());
            instructions.push(Instruction::Mov(asm_arg_type, asm_arg, dst));
        } else {
            // We push stack arguments in reverse order
            let stack_arg_number = i - reg_order.len();
            let asm_arg = tacky_val_to_asm_operand(&args[args.len() - 1 - stack_arg_number]);

            instructions.push(Instruction::Push(asm_arg));
        }
    }

    // Call function
    instructions.push(Instruction::Call(identifier.to_string()));

    // Cleanup arguments
    let stack_arguments = (args.len() as isize - reg_order.len() as isize).max(0) as usize;
    let bytes_to_cleanup = stack_arguments * 8 + stack_padding as usize;
    if bytes_to_cleanup > 0 {
        instructions.push(Instruction::Binary(
            BinaryOperator::Add,
            AssemblyType::Quadword,
            Operand::Imm(bytes_to_cleanup as isize),
            Operand::Reg(Reg::SP),
        ))
    }

    // Retrieve return value
    let dst_asm_op = tacky_val_to_asm_operand(dst);
    let dst_asm_type = tacky_value_type(dst, symbol_table);
    instructions.push(Instruction::Mov(
        dst_asm_type,
        Operand::Reg(Reg::AX),
        dst_asm_op,
    ));

    instructions
}

pub fn tacky_val_to_asm_operand(tacky_function: &tacky::Val) -> Operand {
    match tacky_function {
        tacky::Val::Constant(c) => match c {
            Const::ConstInt(i) => Operand::Imm(*i as isize),
            Const::ConstLong(i) => Operand::Imm(*i as isize),
        },
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

pub fn tacky_value_type(tacky_val: &tacky::Val, symbol_table: &SymbolTable) -> AssemblyType {
    match tacky_val {
        tacky::Val::Var(var_name) => {
            let var_info = symbol_table
                .get(var_name)
                .expect(&format!("Variable {} not found in symbol table", var_name));

            symbol_type_to_asm_type(&var_info.ty)
        }
        tacky::Val::Constant(c) => match c {
            Const::ConstInt(_) => AssemblyType::Longword,
            Const::ConstLong(_) => AssemblyType::Quadword,
        },
    }
}

pub fn symbol_type_to_asm_type(tacky_type: &Type) -> AssemblyType {
    match tacky_type {
        Type::Int => AssemblyType::Longword,
        Type::Long => AssemblyType::Quadword,
        _ => panic!("Unsupported type for assembly codegen"),
    }
}
