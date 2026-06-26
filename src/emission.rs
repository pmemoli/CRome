use crate::{
    codegen,
    symbol::{AssemblyType, StaticInit, SymbolMetadata, SymbolTable},
};

#[derive(Debug, Clone, Copy)]
pub enum OperandSize {
    Byte,
    Word,
    Dword,
    Qword,
}

pub fn emission_program(asm_program: &codegen::Program, symbol_table: &SymbolTable) -> String {
    let codegen::Program(asm_top_level_structs) = asm_program;

    let mut program = String::new();
    for asm_top_level_struct in asm_top_level_structs {
        let top_level_str = emission_top_level(asm_top_level_struct, symbol_table);
        program.push_str(&top_level_str);
    }
    program.push_str("\n    .section .note.GNU-stack,\"\",@progbits\n");

    program
}

pub fn emission_top_level(asm_top_level: &codegen::TopLevel, symbol_table: &SymbolTable) -> String {
    match asm_top_level {
        codegen::TopLevel::Function(name, global, instructions) => {
            let mut function = String::new();

            if *global {
                function.push_str(&format!("    .globl {}\n", name));
            }
            function.push_str("    .text\n");
            function.push_str(&format!("{}:\n", name));

            // Prologue
            function.push_str("    pushq %rbp\n");
            function.push_str("    movq %rsp, %rbp\n");

            for instruction in instructions {
                let instruction_str = emission_instruction(instruction, symbol_table);
                function.push_str(&format!("    {}\n", instruction_str));
            }

            function
        }

        codegen::TopLevel::StaticVariable(name, global, alignment, init) => {
            let global_directive = if *global {
                format!("    .globl {}\n", name)
            } else {
                String::new()
            };
            let alignment_directive = format!("    .align {}\n", alignment);
            let init_directive = match init {
                StaticInit::IntInit(i) => {
                    if *i == 0 {
                        format!("    .zero 4\n")
                    } else {
                        format!("    .long {}\n", i)
                    }
                }
                StaticInit::UIntInit(i) => {
                    if *i == 0 {
                        format!("    .zero 4\n")
                    } else {
                        format!("    .long {}\n", i)
                    }
                }
                StaticInit::LongInit(i) => {
                    if *i == 0 {
                        format!("    .zero 8\n")
                    } else {
                        format!("    .quad {}\n", i)
                    }
                }
                StaticInit::ULongInit(i) => {
                    if *i == 0 {
                        format!("    .zero 8\n")
                    } else {
                        format!("    .quad {}\n", i)
                    }
                }
                StaticInit::DoubleInit(f) => format!("    .quad {}\n", f.to_bits()),
            };

            let init_value: i128 = match init {
                StaticInit::IntInit(i) => *i as i128,
                StaticInit::UIntInit(i) => *i as i128,
                StaticInit::LongInit(i) => *i as i128,
                StaticInit::ULongInit(i) => *i as i128,
                StaticInit::DoubleInit(_) => -1 as i128, // doubles never go into .bss
            };

            let section_directive = if init_value == 0 {
                "    .bss\n"
            } else {
                "    .data\n"
            };

            let mut static_var = String::new();
            static_var.push_str(&global_directive);
            static_var.push_str(section_directive);
            static_var.push_str(&alignment_directive);
            static_var.push_str(&format!("{}:\n", name));
            static_var.push_str(&init_directive);

            static_var
        }

        codegen::TopLevel::StaticConstant(name, alignment, init) => {
            let alignment_directive = format!("    .align {}\n", alignment);
            let init_directive = match init {
                StaticInit::DoubleInit(f) => format!("    .quad {}\n", f.to_bits()),
                _ => panic!("Unsupported static constant type"),
            };

            let mut static_const = String::new();
            static_const.push_str("    .section .rodata\n");
            static_const.push_str(&alignment_directive);
            static_const.push_str(&format!("{}:\n", name));
            static_const.push_str(&init_directive);

            static_const
        }
    }
}

pub fn emission_instruction(
    asm_instructions: &codegen::Instruction,
    symbol_table: &SymbolTable,
) -> String {
    match asm_instructions {
        codegen::Instruction::Mov(ty, src, dst) => {
            let suffix = emission_type_suffix(ty);
            let size = operand_size_from_type(ty);
            let src_str = emission_operand(src, size);
            let dst_str = emission_operand(dst, size);
            format!("mov{} {},{}", suffix, src_str, dst_str)
        }
        codegen::Instruction::Movsx(src, dst) => {
            let src_str = emission_operand(src, OperandSize::Dword);
            let dst_str = emission_operand(dst, OperandSize::Qword);
            format!("movslq {},{}", src_str, dst_str)
        }
        codegen::Instruction::Ret => {
            // epilogue
            format!("movq %rbp, %rsp\n    popq %rbp\n    ret")
        }
        codegen::Instruction::Unary(unop, ty, op) => {
            let unop_str = emission_unary_operator(unop);
            let suffix = emission_type_suffix(ty);
            let op_str = emission_operand(op, operand_size_from_type(ty));
            format!("{}{} {}", unop_str, suffix, op_str)
        }
        codegen::Instruction::Binary(
            codegen::BinaryOperator::Xor,
            AssemblyType::Double,
            src,
            dst,
        ) => {
            let src_str = emission_operand(src, OperandSize::Qword);
            let dst_str = emission_operand(dst, OperandSize::Qword);
            format!("xorpd {},{}", src_str, dst_str)
        }
        codegen::Instruction::Binary(
            codegen::BinaryOperator::Mult,
            AssemblyType::Double,
            src,
            dst,
        ) => {
            let src_str = emission_operand(src, OperandSize::Qword);
            let dst_str = emission_operand(dst, OperandSize::Qword);
            format!("mulsd {},{}", src_str, dst_str)
        }
        codegen::Instruction::Binary(binop, ty, src, dst) => {
            let binop_str = emission_binary_operator(binop);
            let suffix = emission_type_suffix(ty);
            let size = operand_size_from_type(ty);
            let src_str = emission_operand(src, size);
            let dst_str = emission_operand(dst, size);
            format!("{}{} {},{}", binop_str, suffix, src_str, dst_str)
        }
        codegen::Instruction::Cmp(AssemblyType::Double, src, dst) => {
            let src_str = emission_operand(src, OperandSize::Qword);
            let dst_str = emission_operand(dst, OperandSize::Qword);
            format!("comisd {},{}", src_str, dst_str)
        }
        codegen::Instruction::Cmp(ty, src, dst) => {
            let suffix = emission_type_suffix(ty);
            let size = operand_size_from_type(ty);
            let src_str = emission_operand(src, size);
            let dst_str = emission_operand(dst, size);
            format!("cmp{} {},{}", suffix, src_str, dst_str)
        }
        codegen::Instruction::Cdq(ty) => match ty {
            AssemblyType::Longword => format!("cdq"),
            AssemblyType::Quadword => format!("cqo"),
            _ => panic!("cdq/cqo only supports Longword and Quadword types"),
        },
        codegen::Instruction::Idiv(ty, op) => {
            let suffix = emission_type_suffix(ty);
            let op_str = emission_operand(op, operand_size_from_type(ty));
            format!("idiv{} {}", suffix, op_str)
        }
        codegen::Instruction::Div(ty, op) => {
            let suffix = emission_type_suffix(ty);
            let op_str = emission_operand(op, operand_size_from_type(ty));
            format!("div{} {}", suffix, op_str)
        }
        codegen::Instruction::Jmp(label) => format!("jmp .L{}", label),
        codegen::Instruction::JmpCC(cond_code, label) => {
            let cond_code_str = emission_cond_code(cond_code);
            format!("j{} .L{}", cond_code_str, label)
        }
        codegen::Instruction::SetCC(cond_code, op) => {
            let cond_code_str = emission_cond_code(cond_code);
            let op_str = emission_operand(op, OperandSize::Byte);
            format!("set{} {}", cond_code_str, op_str)
        }
        codegen::Instruction::Label(label) => format!(".L{}:", label),
        codegen::Instruction::Push(op) => {
            let op_str = emission_operand(op, OperandSize::Qword);
            format!("pushq {}", op_str)
        }
        codegen::Instruction::Cvttsd2si(ty, src, dst) => {
            let suffix = emission_type_suffix(ty);
            let src_str = emission_operand(src, OperandSize::Qword);
            let dst_str = emission_operand(dst, operand_size_from_type(ty));
            format!("cvttsd2si{} {},{}", suffix, src_str, dst_str)
        }
        codegen::Instruction::Cvtsi2sd(ty, src, dst) => {
            let suffix = emission_type_suffix(ty);
            let src_str = emission_operand(src, operand_size_from_type(ty));
            let dst_str = emission_operand(dst, OperandSize::Qword);
            format!("cvtsi2sd{} {},{}", suffix, src_str, dst_str)
        }
        codegen::Instruction::Vcvtusi2sd(ty, src, dst) => {
            let suffix = emission_type_suffix(ty);
            let src_str = emission_operand(src, operand_size_from_type(ty));
            let dst_str = emission_operand(dst, OperandSize::Qword);
            format!("vcvtusi2sd{} {},{},{}", suffix, src_str, dst_str, dst_str)
        }
        codegen::Instruction::Vcvttsd2usi(ty, src, dst) => {
            let src_str = emission_operand(src, OperandSize::Qword);
            let dst_str = emission_operand(dst, operand_size_from_type(ty));
            format!("vcvttsd2usi {},{}", src_str, dst_str)
        }
        codegen::Instruction::Call(label) => {
            if let Some(symbol_info) = symbol_table.map.get(label) {
                match symbol_info.metadata {
                    SymbolMetadata::Function { .. } => {
                        format!("call {}@PLT", label)
                    }
                    _ => panic!("Attempting to call a non-function symbol: {}", label),
                }
            } else {
                panic!("Undefined symbol: {}", label);
            }
        }
        _ => panic!("Unexpected instruction type in emission"),
    }
}

pub fn operand_size_from_type(asm_type: &AssemblyType) -> OperandSize {
    match asm_type {
        AssemblyType::Longword => OperandSize::Dword,
        AssemblyType::Quadword => OperandSize::Qword,
        AssemblyType::Double => OperandSize::Qword,
    }
}

pub fn emission_register(asm_reg: &codegen::Reg, size: OperandSize) -> String {
    match (asm_reg, size) {
        (codegen::Reg::AX, OperandSize::Byte) => String::from("%al"),
        (codegen::Reg::AX, OperandSize::Word) => String::from("%ax"),
        (codegen::Reg::AX, OperandSize::Dword) => String::from("%eax"),
        (codegen::Reg::AX, OperandSize::Qword) => String::from("%rax"),
        (codegen::Reg::CX, OperandSize::Byte) => String::from("%cl"),
        (codegen::Reg::CX, OperandSize::Word) => String::from("%cx"),
        (codegen::Reg::CX, OperandSize::Dword) => String::from("%ecx"),
        (codegen::Reg::CX, OperandSize::Qword) => String::from("%rcx"),
        (codegen::Reg::DX, OperandSize::Byte) => String::from("%dl"),
        (codegen::Reg::DX, OperandSize::Word) => String::from("%dx"),
        (codegen::Reg::DX, OperandSize::Dword) => String::from("%edx"),
        (codegen::Reg::DX, OperandSize::Qword) => String::from("%rdx"),
        (codegen::Reg::DI, OperandSize::Byte) => String::from("%dil"),
        (codegen::Reg::DI, OperandSize::Word) => String::from("%di"),
        (codegen::Reg::DI, OperandSize::Dword) => String::from("%edi"),
        (codegen::Reg::DI, OperandSize::Qword) => String::from("%rdi"),
        (codegen::Reg::SI, OperandSize::Byte) => String::from("%sil"),
        (codegen::Reg::SI, OperandSize::Word) => String::from("%si"),
        (codegen::Reg::SI, OperandSize::Dword) => String::from("%esi"),
        (codegen::Reg::SI, OperandSize::Qword) => String::from("%rsi"),
        (codegen::Reg::R8, OperandSize::Byte) => String::from("%r8b"),
        (codegen::Reg::R8, OperandSize::Word) => String::from("%r8w"),
        (codegen::Reg::R8, OperandSize::Dword) => String::from("%r8d"),
        (codegen::Reg::R8, OperandSize::Qword) => String::from("%r8"),
        (codegen::Reg::R9, OperandSize::Byte) => String::from("%r9b"),
        (codegen::Reg::R9, OperandSize::Word) => String::from("%r9w"),
        (codegen::Reg::R9, OperandSize::Dword) => String::from("%r9d"),
        (codegen::Reg::R9, OperandSize::Qword) => String::from("%r9"),
        (codegen::Reg::R10, OperandSize::Byte) => String::from("%r10b"),
        (codegen::Reg::R10, OperandSize::Word) => String::from("%r10w"),
        (codegen::Reg::R10, OperandSize::Dword) => String::from("%r10d"),
        (codegen::Reg::R10, OperandSize::Qword) => String::from("%r10"),
        (codegen::Reg::R11, OperandSize::Byte) => String::from("%r11b"),
        (codegen::Reg::R11, OperandSize::Word) => String::from("%r11w"),
        (codegen::Reg::R11, OperandSize::Dword) => String::from("%r11d"),
        (codegen::Reg::R11, OperandSize::Qword) => String::from("%r11"),

        // XMM0 regs
        (codegen::Reg::XMM0, OperandSize::Qword) => String::from("%xmm0"),
        (codegen::Reg::XMM1, OperandSize::Qword) => String::from("%xmm1"),
        (codegen::Reg::XMM2, OperandSize::Qword) => String::from("%xmm2"),
        (codegen::Reg::XMM3, OperandSize::Qword) => String::from("%xmm3"),
        (codegen::Reg::XMM4, OperandSize::Qword) => String::from("%xmm4"),
        (codegen::Reg::XMM5, OperandSize::Qword) => String::from("%xmm5"),
        (codegen::Reg::XMM6, OperandSize::Qword) => String::from("%xmm6"),
        (codegen::Reg::XMM7, OperandSize::Qword) => String::from("%xmm7"),
        (codegen::Reg::XMM14, OperandSize::Qword) => String::from("%xmm14"),
        (codegen::Reg::XMM15, OperandSize::Qword) => String::from("%xmm15"),

        // Stack regs
        (codegen::Reg::SP, OperandSize::Qword) => String::from("%rsp"),

        _ => panic!(
            "Invalid register and operand size combination: {:?} with size {:?}",
            asm_reg, size
        ),
    }
}

pub fn emission_operand(asm_operand: &codegen::Operand, size: OperandSize) -> String {
    match asm_operand {
        codegen::Operand::Imm(i) => format!("${}", i),
        codegen::Operand::Reg(r) => emission_register(r, size),
        codegen::Operand::Stack(i) => format!("{}(%rbp)", i),
        codegen::Operand::Data(label) => format!("{}(%rip)", label),
        _ => panic!("Unexpected operand type in emission"),
    }
}

pub fn emission_type_suffix(asm_type: &AssemblyType) -> String {
    match asm_type {
        AssemblyType::Longword => String::from("l"),
        AssemblyType::Quadword => String::from("q"),
        AssemblyType::Double => String::from("sd"),
    }
}

pub fn emission_unary_operator(asm_unop: &codegen::UnaryOperator) -> String {
    match asm_unop {
        codegen::UnaryOperator::Not => String::from("not"),
        codegen::UnaryOperator::Neg => String::from("neg"),
    }
}

pub fn emission_binary_operator(asm_unop: &codegen::BinaryOperator) -> String {
    match asm_unop {
        codegen::BinaryOperator::Add => String::from("add"),
        codegen::BinaryOperator::Sub => String::from("sub"),
        codegen::BinaryOperator::Mult => String::from("imul"),
        codegen::BinaryOperator::DivDouble => String::from("div"),
        _ => panic!("Unexpected binary operator in emission"),
    }
}

pub fn emission_cond_code(asm_cond_code: &codegen::CondCode) -> String {
    match asm_cond_code {
        codegen::CondCode::E => String::from("e"),
        codegen::CondCode::NE => String::from("ne"),
        codegen::CondCode::L => String::from("l"),
        codegen::CondCode::LE => String::from("le"),
        codegen::CondCode::G => String::from("g"),
        codegen::CondCode::GE => String::from("ge"),
        codegen::CondCode::A => String::from("a"),
        codegen::CondCode::AE => String::from("ae"),
        codegen::CondCode::B => String::from("b"),
        codegen::CondCode::BE => String::from("be"),
    }
}
