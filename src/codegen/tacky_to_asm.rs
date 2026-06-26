use std::collections::HashMap;

use super::*;
use crate::parser::Const;
use crate::symbol::Type;
use crate::tacky;

// Static constant struct
pub struct StaticConstants {
    map: HashMap<String, StaticConstantData>,
}

pub struct StaticConstantData {
    alignment: usize,
    init: StaticInit,
}

impl StaticConstants {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    pub fn get_by_value(&self, value: f64, alignment: usize) -> Option<&String> {
        self.map.iter().find_map(|(name, data)| {
            if let StaticInit::DoubleInit(existing_f) = &data.init {
                if existing_f.to_bits() == value.to_bits() && data.alignment == alignment {
                    return Some(name);
                }
            }
            None
        })
    }

    pub fn insert(&mut self, name: String, alignment: usize, value: f64) {
        let init = StaticInit::DoubleInit(value);

        self.map
            .insert(name, StaticConstantData { alignment, init });
    }
}

// First pass: Convert Tacky to ASM AST (with temp variables as pseudoregisters)
pub fn tacky_program_to_asm(
    tacky_program: &tacky::Program,
    symbol_table: &mut SymbolTable,
) -> Program {
    let tacky::Program(tacky_top_level) = tacky_program;

    let mut asm_top_level = Vec::new();
    let mut static_constant_names = StaticConstants::new();
    for tacky_top_object in tacky_top_level {
        asm_top_level.push(tacky_top_level_to_asm(
            tacky_top_object,
            symbol_table,
            &mut static_constant_names,
        ));
    }

    for (name, data) in &static_constant_names.map {
        asm_top_level.push(TopLevel::StaticConstant(
            name.to_string(),
            data.alignment,
            data.init.clone(),
        ));
    }

    Program(asm_top_level)
}

pub fn tacky_top_level_to_asm(
    tacky_top_level: &tacky::TopLevel,
    symbol_table: &mut SymbolTable,
    static_constant_names: &mut StaticConstants,
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
            let reg_int_order = vec![Reg::DI, Reg::SI, Reg::DX, Reg::CX, Reg::R8, Reg::R9];
            let reg_float_order = vec![
                Reg::XMM0,
                Reg::XMM1,
                Reg::XMM2,
                Reg::XMM3,
                Reg::XMM4,
                Reg::XMM5,
                Reg::XMM6,
                Reg::XMM7,
            ];

            let mut reg_int_params = Vec::new();
            let mut reg_float_params = Vec::new();
            let mut stack_params = Vec::new();

            for i in 0..tacky_arguments.len() {
                let arg_name = &tacky_arguments[i];
                let arg_type = symbol_table.identifier_type(arg_name).unwrap();

                if arg_type.is_floating_point() {
                    if reg_float_params.len() < reg_float_order.len() {
                        reg_float_params.push(arg_name);
                    } else {
                        stack_params.push(arg_name);
                    }
                } else {
                    if reg_int_params.len() < reg_int_order.len() {
                        reg_int_params.push(arg_name);
                    } else {
                        stack_params.push(arg_name);
                    }
                }
            }

            for i in 0..reg_int_params.len() {
                let arg = reg_int_params[i].to_string();
                let arg_type = symbol_table.identifier_type(&arg).unwrap();
                let arg_asm_type = symbol_type_to_asm_type(arg_type);
                let src = Operand::Reg(reg_int_order[i].clone());
                asm_instructions.push(Instruction::Mov(arg_asm_type, src, Operand::Pseudo(arg)));
            }

            for i in 0..reg_float_params.len() {
                let arg = reg_float_params[i].to_string();
                let arg_type = symbol_table.identifier_type(&arg).unwrap();
                let arg_asm_type = symbol_type_to_asm_type(arg_type);
                let src = Operand::Reg(reg_float_order[i].clone());
                asm_instructions.push(Instruction::Mov(arg_asm_type, src, Operand::Pseudo(arg)));
            }

            for i in 0..stack_params.len() {
                let arg = stack_params[i].to_string();
                let arg_type = symbol_table.identifier_type(&arg).unwrap();
                let arg_asm_type = symbol_type_to_asm_type(arg_type);
                asm_instructions.push(Instruction::Mov(
                    arg_asm_type,
                    Operand::Stack(8 * (i + 2) as isize), // First arg is at RSP + 16 (old RBP + ret address)
                    Operand::Pseudo(arg),
                ));
            }

            for instruction in tacky_instructions {
                let mut asm_instrs =
                    tacky_instruction_to_asm(instruction, symbol_table, static_constant_names);
                asm_instructions.append(&mut asm_instrs);
            }

            let identifier = tacky_identifier.to_string();

            TopLevel::Function(identifier, *tacky_global, asm_instructions)
        }
        tacky::TopLevel::StaticVariable(identifier, global, ty, init) => {
            let alignment = symbol_type_to_asm_type(ty).alignment();

            TopLevel::StaticVariable(identifier.to_string(), *global, alignment, init.clone())
        }
    }
}

pub fn tacky_instruction_to_asm(
    tacky_function: &tacky::Instruction,
    symbol_table: &mut SymbolTable,
    static_constant_names: &mut StaticConstants,
) -> Vec<Instruction> {
    match tacky_function {
        tacky::Instruction::Return(val) => {
            let val_asm_type = tacky_value_type_asm(val, symbol_table);
            let ty = tacky_value_type(val, symbol_table);

            let src_asm_op = tacky_val_to_asm_operand(val, symbol_table, static_constant_names);

            match ty {
                t if ty.is_integer() => {
                    let dst_asm_op = Operand::Reg(Reg::AX);
                    vec![
                        Instruction::Mov(val_asm_type, src_asm_op, dst_asm_op),
                        Instruction::Ret,
                    ]
                }
                t if ty.is_floating_point() => {
                    let dst_asm_op = Operand::Reg(Reg::XMM0);
                    vec![
                        Instruction::Mov(val_asm_type, src_asm_op, dst_asm_op),
                        Instruction::Ret,
                    ]
                }
                _ => panic!("Unsupported return type in codegen"),
            }
        }

        tacky::Instruction::Unary(unop, src, dst) => {
            let src_asm_type = tacky_value_type_asm(src, symbol_table);
            let dst_asm_type = tacky_value_type_asm(dst, symbol_table);
            let src_asm_op = tacky_val_to_asm_operand(src, symbol_table, static_constant_names);
            let dst_asm_op = tacky_val_to_asm_operand(dst, symbol_table, static_constant_names);

            let ty = tacky_value_type(src, symbol_table);

            match (unop, ty) {
                (tacky::UnaryOperator::Not, t) if t.is_integer() => {
                    vec![
                        Instruction::Cmp(src_asm_type, Operand::Imm(0), src_asm_op),
                        Instruction::Mov(dst_asm_type, Operand::Imm(0), dst_asm_op.clone()),
                        Instruction::SetCC(CondCode::E, dst_asm_op),
                    ]
                }
                (tacky::UnaryOperator::Not, t) if t.is_floating_point() => {
                    vec![
                        Instruction::Binary(
                            BinaryOperator::Xor,
                            AssemblyType::Double,
                            Operand::Reg(Reg::XMM14),
                            Operand::Reg(Reg::XMM14),
                        ),
                        Instruction::Cmp(
                            AssemblyType::Double,
                            src_asm_op,
                            Operand::Reg(Reg::XMM14),
                        ),
                        Instruction::Mov(dst_asm_type, Operand::Imm(0), dst_asm_op.clone()),
                        Instruction::SetCC(CondCode::E, dst_asm_op),
                    ]
                }
                (tacky::UnaryOperator::Complement, _) => {
                    vec![
                        Instruction::Mov(src_asm_type.clone(), src_asm_op, dst_asm_op.clone()),
                        Instruction::Unary(UnaryOperator::Not, src_asm_type, dst_asm_op),
                    ]
                }
                (tacky::UnaryOperator::Negate, t) if t.is_floating_point() => {
                    // Xorpd needs this 16 byte aligned rather than just 8
                    let zero_constant_name =
                        create_float_constant(-0.0, 16, symbol_table, static_constant_names);

                    vec![
                        Instruction::Mov(AssemblyType::Double, src_asm_op, dst_asm_op.clone()),
                        Instruction::Binary(
                            BinaryOperator::Xor,
                            AssemblyType::Double,
                            Operand::Data(zero_constant_name),
                            dst_asm_op.clone(),
                        ),
                    ]
                }
                (tacky::UnaryOperator::Negate, t) if t.is_integer() => {
                    vec![
                        Instruction::Mov(src_asm_type.clone(), src_asm_op, dst_asm_op.clone()),
                        Instruction::Unary(UnaryOperator::Neg, src_asm_type, dst_asm_op),
                    ]
                }
                _ => panic!("Unsupported unary operator/type combination in codegen"),
            }
        }
        tacky::Instruction::Binary(op, src_a, src_b, dst) => {
            let src_a_asm_op = tacky_val_to_asm_operand(src_a, symbol_table, static_constant_names);
            let src_b_asm_op = tacky_val_to_asm_operand(src_b, symbol_table, static_constant_names);
            let src_a_asm_type = tacky_value_type_asm(src_a, symbol_table);

            let dst_asm_op = tacky_val_to_asm_operand(dst, symbol_table, static_constant_names);
            let dst_asm_type = tacky_value_type_asm(dst, symbol_table);

            let ty = tacky_value_type(src_a, symbol_table);

            match (op, ty) {
                (tacky::BinaryOperator::Divide, ty) if ty.signed() && ty.is_integer() => {
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
                (tacky::BinaryOperator::Divide, ty) if !ty.signed() && ty.is_integer() => {
                    vec![
                        Instruction::Mov(
                            src_a_asm_type.clone(),
                            src_a_asm_op,
                            Operand::Reg(Reg::AX),
                        ),
                        Instruction::Mov(
                            src_a_asm_type.clone(),
                            Operand::Imm(0),
                            Operand::Reg(Reg::DX),
                        ),
                        Instruction::Div(src_a_asm_type.clone(), src_b_asm_op),
                        Instruction::Mov(src_a_asm_type, Operand::Reg(Reg::AX), dst_asm_op),
                    ]
                }
                (tacky::BinaryOperator::Divide, Type::Double) => {
                    vec![
                        Instruction::Mov(src_a_asm_type.clone(), src_a_asm_op, dst_asm_op.clone()),
                        Instruction::Binary(
                            BinaryOperator::DivDouble,
                            src_a_asm_type.clone(),
                            src_b_asm_op,
                            dst_asm_op,
                        ),
                    ]
                }
                (tacky::BinaryOperator::Remainder, ty) if ty.signed() => {
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
                (tacky::BinaryOperator::Remainder, ty) if ty.is_integer() => {
                    vec![
                        Instruction::Mov(
                            src_a_asm_type.clone(),
                            src_a_asm_op,
                            Operand::Reg(Reg::AX),
                        ),
                        Instruction::Mov(
                            src_a_asm_type.clone(),
                            Operand::Imm(0),
                            Operand::Reg(Reg::DX),
                        ),
                        Instruction::Div(src_a_asm_type.clone(), src_b_asm_op),
                        Instruction::Mov(src_a_asm_type, Operand::Reg(Reg::DX), dst_asm_op),
                    ]
                }

                (
                    tacky::BinaryOperator::Equal
                    | tacky::BinaryOperator::NotEqual
                    | tacky::BinaryOperator::LessThan
                    | tacky::BinaryOperator::LessOrEqual
                    | tacky::BinaryOperator::GreaterThan
                    | tacky::BinaryOperator::GreaterOrEqual,
                    ty,
                ) if ty.is_integer() || ty.is_floating_point() => {
                    let cond_code = if ty.signed() && ty.is_integer() {
                        tacky_binop_to_cond_asm(op)
                    } else {
                        tacky_unsigned_binop_to_cond_asm(op)
                    };

                    vec![
                        Instruction::Cmp(src_a_asm_type, src_b_asm_op, src_a_asm_op),
                        Instruction::Mov(dst_asm_type, Operand::Imm(0), dst_asm_op.clone()),
                        Instruction::SetCC(cond_code, dst_asm_op),
                    ]
                }

                (
                    tacky::BinaryOperator::Add
                    | tacky::BinaryOperator::Subtract
                    | tacky::BinaryOperator::Multiply,
                    _,
                ) => {
                    let binop_asm_op = tacky_binop_to_binop_asm(op);

                    vec![
                        Instruction::Mov(src_a_asm_type.clone(), src_a_asm_op, dst_asm_op.clone()),
                        Instruction::Binary(binop_asm_op, src_a_asm_type, src_b_asm_op, dst_asm_op),
                    ]
                }

                _ => panic!("Unsupported binary operator/type combination in codegen"),
            }
        }
        tacky::Instruction::Copy(src, dst) => {
            let src_asm_type = tacky_value_type_asm(src, symbol_table);
            let src_asm_op = tacky_val_to_asm_operand(src, symbol_table, static_constant_names);
            let dst_asm_op = tacky_val_to_asm_operand(dst, symbol_table, static_constant_names);
            vec![Instruction::Mov(src_asm_type, src_asm_op, dst_asm_op)]
        }
        tacky::Instruction::Jump(label) => vec![Instruction::Jmp(label.to_string())],
        tacky::Instruction::JumpIfZero(cond, label) => {
            let cond_asm_type = tacky_value_type_asm(cond, symbol_table);
            let cond_asm_op = tacky_val_to_asm_operand(cond, symbol_table, static_constant_names);
            let ty = tacky_value_type(cond, symbol_table);

            match ty {
                t if t.is_floating_point() => {
                    vec![
                        Instruction::Binary(
                            BinaryOperator::Xor,
                            AssemblyType::Double,
                            Operand::Reg(Reg::XMM14),
                            Operand::Reg(Reg::XMM14),
                        ),
                        Instruction::Cmp(
                            AssemblyType::Double,
                            cond_asm_op,
                            Operand::Reg(Reg::XMM14),
                        ),
                        Instruction::JmpCC(CondCode::E, label.to_string()),
                    ]
                }
                _ => vec![
                    Instruction::Cmp(cond_asm_type, Operand::Imm(0), cond_asm_op),
                    Instruction::JmpCC(CondCode::E, label.to_string()),
                ],
            }
        }
        tacky::Instruction::JumpIfNotZero(cond, label) => {
            let cond_asm_type = tacky_value_type_asm(cond, symbol_table);
            let cond_asm_op = tacky_val_to_asm_operand(cond, symbol_table, static_constant_names);
            let ty = tacky_value_type(cond, symbol_table);

            match ty {
                t if t.is_floating_point() => {
                    vec![
                        Instruction::Binary(
                            BinaryOperator::Xor,
                            AssemblyType::Double,
                            Operand::Reg(Reg::XMM14),
                            Operand::Reg(Reg::XMM14),
                        ),
                        Instruction::Cmp(
                            AssemblyType::Double,
                            cond_asm_op,
                            Operand::Reg(Reg::XMM14),
                        ),
                        Instruction::JmpCC(CondCode::NE, label.to_string()),
                    ]
                }
                _ => vec![
                    Instruction::Cmp(cond_asm_type, Operand::Imm(0), cond_asm_op),
                    Instruction::JmpCC(CondCode::NE, label.to_string()),
                ],
            }
        }
        tacky::Instruction::Label(label) => vec![Instruction::Label(label.to_string())],
        tacky::Instruction::FunCall(..) => {
            tacky_fun_call_to_asm(tacky_function, symbol_table, static_constant_names)
        }
        tacky::Instruction::SignExtend(src, dst) => {
            let asm_src = tacky_val_to_asm_operand(src, symbol_table, static_constant_names);
            let asm_dst = tacky_val_to_asm_operand(dst, symbol_table, static_constant_names);
            vec![Instruction::Movsx(asm_src, asm_dst)]
        }
        tacky::Instruction::Truncate(src, dst) => {
            let asm_src = tacky_val_to_asm_operand(src, symbol_table, static_constant_names);
            let asm_dst = tacky_val_to_asm_operand(dst, symbol_table, static_constant_names);
            vec![Instruction::Mov(AssemblyType::Longword, asm_src, asm_dst)]
        }
        tacky::Instruction::ZeroExtend(src, dst) => {
            let asm_src = tacky_val_to_asm_operand(src, symbol_table, static_constant_names);
            let asm_dst = tacky_val_to_asm_operand(dst, symbol_table, static_constant_names);
            vec![Instruction::MovZeroExtend(asm_src, asm_dst)]
        }
        tacky::Instruction::IntToDouble(src, dst) => {
            let asm_src = tacky_val_to_asm_operand(src, symbol_table, static_constant_names);
            let asm_dst = tacky_val_to_asm_operand(dst, symbol_table, static_constant_names);

            let asm_src_type = tacky_value_type_asm(src, symbol_table);

            vec![Instruction::Cvtsi2sd(asm_src_type, asm_src, asm_dst)]
        }
        tacky::Instruction::UIntToDouble(src, dst) => {
            let asm_src = tacky_val_to_asm_operand(src, symbol_table, static_constant_names);
            let asm_dst = tacky_val_to_asm_operand(dst, symbol_table, static_constant_names);

            let asm_src_type = tacky_value_type_asm(src, symbol_table);

            vec![Instruction::Vcvtusi2sd(asm_src_type, asm_src, asm_dst)]
        }
        tacky::Instruction::DoubleToInt(src, dst) => {
            let asm_src = tacky_val_to_asm_operand(src, symbol_table, static_constant_names);
            let asm_dst = tacky_val_to_asm_operand(dst, symbol_table, static_constant_names);

            let asm_dst_type = tacky_value_type_asm(dst, symbol_table);

            vec![Instruction::Cvttsd2si(asm_dst_type, asm_src, asm_dst)]
        }
        tacky::Instruction::DoubleToUInt(src, dst) => {
            let asm_src = tacky_val_to_asm_operand(src, symbol_table, static_constant_names);
            let asm_dst = tacky_val_to_asm_operand(dst, symbol_table, static_constant_names);

            let asm_dst_type = tacky_value_type_asm(dst, symbol_table);

            vec![Instruction::Vcvttsd2usi(asm_dst_type, asm_src, asm_dst)]
        }
    }
}

pub fn tacky_fun_call_to_asm(
    tacky_fun_call: &tacky::Instruction,
    symbol_table: &mut SymbolTable,
    static_constant_names: &mut StaticConstants,
) -> Vec<Instruction> {
    let tacky::Instruction::FunCall(identifier, args, dst) = tacky_fun_call else {
        panic!("Expected a function call instruction");
    };

    let mut instructions = Vec::new();

    // Pass parameters according to ABI
    let reg_int_order = vec![Reg::DI, Reg::SI, Reg::DX, Reg::CX, Reg::R8, Reg::R9];
    let reg_float_order = vec![
        Reg::XMM0,
        Reg::XMM1,
        Reg::XMM2,
        Reg::XMM3,
        Reg::XMM4,
        Reg::XMM5,
        Reg::XMM6,
        Reg::XMM7,
    ];

    let mut reg_int_params = Vec::new();
    let mut reg_float_params = Vec::new();
    let mut stack_params = Vec::new();

    for i in 0..args.len() {
        let arg = &args[i];
        let arg_type = tacky_value_type(arg, symbol_table);

        if arg_type.is_floating_point() {
            if reg_float_params.len() < reg_float_order.len() {
                reg_float_params.push(arg);
            } else {
                stack_params.push(arg);
            }
        } else {
            if reg_int_params.len() < reg_int_order.len() {
                reg_int_params.push(arg);
            } else {
                stack_params.push(arg);
            }
        }
    }

    // Add stack padding to ensure its 16-byte aligned
    let stack_padding = if stack_params.len() % 2 == 1 {
        instructions.push(Instruction::Binary(
            BinaryOperator::Sub,
            AssemblyType::Quadword,
            Operand::Imm(8),
            Operand::Reg(Reg::SP),
        ));
        8
    } else {
        0
    };

    for i in 0..reg_int_params.len() {
        let arg = reg_int_params[i];
        let asm_arg_type = tacky_value_type_asm(arg, symbol_table);
        let asm_arg = tacky_val_to_asm_operand(arg, symbol_table, static_constant_names);
        let dst = Operand::Reg(reg_int_order[i].clone());
        instructions.push(Instruction::Mov(asm_arg_type, asm_arg, dst));
    }

    for i in 0..reg_float_params.len() {
        let arg = reg_float_params[i];
        let asm_arg_type = tacky_value_type_asm(arg, symbol_table);
        let asm_arg = tacky_val_to_asm_operand(arg, symbol_table, static_constant_names);
        let dst = Operand::Reg(reg_float_order[i].clone());
        instructions.push(Instruction::Mov(asm_arg_type, asm_arg, dst));
    }

    for i in (0..stack_params.len()).rev() {
        let arg = stack_params[i];
        let asm_arg = tacky_val_to_asm_operand(arg, symbol_table, static_constant_names);

        // Could be fixed in the instruction fixup pass, but whatever
        match asm_arg {
            Operand::Imm(_) | Operand::Reg(_) => {
                instructions.push(Instruction::Push(asm_arg));
            }
            Operand::Pseudo(_) | Operand::Stack(_) | Operand::Data(_) => {
                let asm_arg_type = tacky_value_type_asm(arg, symbol_table);
                let mov_type = match asm_arg_type {
                    AssemblyType::Longword => AssemblyType::Longword,
                    _ => AssemblyType::Quadword,
                };
                instructions.push(Instruction::Mov(mov_type, asm_arg, Operand::Reg(Reg::R10)));
                instructions.push(Instruction::Push(Operand::Reg(Reg::R10)));
            }
        }
    }

    // Call function
    instructions.push(Instruction::Call(identifier.to_string()));

    // Cleanup arguments
    let bytes_to_cleanup = stack_params.len() * 8 + stack_padding as usize;
    if bytes_to_cleanup > 0 {
        instructions.push(Instruction::Binary(
            BinaryOperator::Add,
            AssemblyType::Quadword,
            Operand::Imm(bytes_to_cleanup as i128),
            Operand::Reg(Reg::SP),
        ))
    }

    // Retrieve return value
    let dst_asm_op = tacky_val_to_asm_operand(dst, symbol_table, static_constant_names);
    let dst_asm_type = tacky_value_type_asm(dst, symbol_table);

    match dst_asm_type {
        AssemblyType::Double => {
            instructions.push(Instruction::Mov(
                dst_asm_type,
                Operand::Reg(Reg::XMM0),
                dst_asm_op.clone(),
            ));
        }
        _ => {
            instructions.push(Instruction::Mov(
                dst_asm_type,
                Operand::Reg(Reg::AX),
                dst_asm_op,
            ));
        }
    }

    instructions
}

pub fn tacky_val_to_asm_operand(
    tacky_function: &tacky::Val,
    symbol_table: &mut SymbolTable,
    static_constant_names: &mut StaticConstants,
) -> Operand {
    match tacky_function {
        tacky::Val::Constant(c) => match c {
            Const::ConstInt(i) => Operand::Imm(*i as i128),
            Const::ConstUInt(u) => Operand::Imm(*u as i128),
            Const::ConstLong(i) => Operand::Imm(*i as i128),
            Const::ConstULong(u) => Operand::Imm(*u as i128),
            Const::ConstDouble(f) => {
                let const_name = create_float_constant(
                    *f,
                    AssemblyType::Double.alignment(),
                    symbol_table,
                    static_constant_names,
                );
                Operand::Data(const_name)
            }
        },
        tacky::Val::Var(s) => Operand::Pseudo(s.to_string()),
    }
}

pub fn create_float_constant(
    f: f64,
    alignment: usize,
    symbol_table: &mut SymbolTable,
    static_constant_names: &mut StaticConstants,
) -> String {
    let existing_const = static_constant_names.get_by_value(f, alignment);

    if let Some(existing_const_name) = existing_const {
        existing_const_name.clone()
    } else {
        let const_name = symbol_table.unique_var_name();
        static_constant_names.insert(const_name.clone(), alignment, f);
        const_name
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

pub fn tacky_unsigned_binop_to_cond_asm(tacky_binop: &tacky::BinaryOperator) -> CondCode {
    match tacky_binop {
        tacky::BinaryOperator::Equal => CondCode::E,
        tacky::BinaryOperator::NotEqual => CondCode::NE,
        tacky::BinaryOperator::LessThan => CondCode::B,
        tacky::BinaryOperator::LessOrEqual => CondCode::BE,
        tacky::BinaryOperator::GreaterThan => CondCode::A,
        tacky::BinaryOperator::GreaterOrEqual => CondCode::AE,
        _ => panic!("Can't convert non-comparison binary operator to condition code in codegen"),
    }
}

pub fn tacky_value_type_asm(tacky_val: &tacky::Val, symbol_table: &SymbolTable) -> AssemblyType {
    match tacky_val {
        tacky::Val::Var(var_name) => {
            let var_info = symbol_table
                .get(var_name)
                .expect(&format!("Variable {} not found in symbol table", var_name));

            symbol_type_to_asm_type(&var_info.ty)
        }
        tacky::Val::Constant(c) => match c {
            Const::ConstInt(_) => AssemblyType::Longword,
            Const::ConstUInt(_) => AssemblyType::Longword,
            Const::ConstLong(_) => AssemblyType::Quadword,
            Const::ConstULong(_) => AssemblyType::Quadword,
            Const::ConstDouble(_) => AssemblyType::Double,
        },
    }
}

pub fn symbol_type_to_asm_type(tacky_type: &Type) -> AssemblyType {
    match tacky_type {
        Type::Int => AssemblyType::Longword,
        Type::UInt => AssemblyType::Longword,
        Type::Long => AssemblyType::Quadword,
        Type::ULong => AssemblyType::Quadword,
        Type::Double => AssemblyType::Double,
        _ => panic!("Unsupported type for assembly codegen"),
    }
}

pub fn tacky_value_type(tacky_val: &tacky::Val, symbol_table: &SymbolTable) -> Type {
    match tacky_val {
        tacky::Val::Var(var_name) => {
            let var_info = symbol_table
                .get(var_name)
                .expect(&format!("Variable {} not found in symbol table", var_name));

            var_info.ty.clone()
        }
        tacky::Val::Constant(c) => match c {
            Const::ConstInt(_) => Type::Int,
            Const::ConstUInt(_) => Type::UInt,
            Const::ConstLong(_) => Type::Long,
            Const::ConstULong(_) => Type::ULong,
            Const::ConstDouble(_) => Type::Double,
        },
    }
}
