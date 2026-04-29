use crate::{codegen, symbol::SymbolMetadata, symbol::SymbolTable};

pub enum OperandSize {
    Byte,
    Word,
    Dword,
    Qword,
}

pub fn emission_program(asm_program: &codegen::Program, symbol_table: &SymbolTable) -> String {
    let codegen::Program(asm_functions) = asm_program;

    let mut program = String::new();
    for asm_function in asm_functions {
        let function_str = emission_function(asm_function, symbol_table);
        program.push_str(&function_str);
    }
    program.push_str("\n    .section .note.GNU-stack,\"\",@progbits\n");

    program
}

pub fn emission_function(asm_function: &codegen::Function, symbol_table: &SymbolTable) -> String {
    let codegen::Function(name, instructions) = asm_function;

    let mut function = String::new();

    function.push_str(&format!("    .globl {}\n", name));
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

pub fn emission_instruction(
    asm_instructions: &codegen::Instruction,
    symbol_table: &SymbolTable,
) -> String {
    match asm_instructions {
        codegen::Instruction::Mov(src, dst) => {
            let src_str = emission_operand(src, OperandSize::Dword);
            let dst_str = emission_operand(dst, OperandSize::Dword);
            format!("movl {},{}", src_str, dst_str)
        }
        codegen::Instruction::Ret => {
            format!("movq %rbp, %rsp\n    popq %rbp\n    ret")
        }
        codegen::Instruction::Unary(unop, op) => {
            let unop_str = emission_unary_operator(unop);
            let op_str = emission_operand(op, OperandSize::Dword);
            format!("{} {}", unop_str, op_str)
        }
        codegen::Instruction::Binary(binop, src, dst) => {
            let binop_str = emission_binary_operator(binop);
            let src_str = emission_operand(src, OperandSize::Dword);
            let dst_str = emission_operand(dst, OperandSize::Dword);
            format!("{} {},{}", binop_str, src_str, dst_str)
        }
        codegen::Instruction::Cmp(src, dst) => {
            let src_str = emission_operand(src, OperandSize::Dword);
            let dst_str = emission_operand(dst, OperandSize::Dword);
            format!("cmpl {},{}", src_str, dst_str)
        }
        codegen::Instruction::Cdq => format!("cdq"),
        codegen::Instruction::Idiv(op) => {
            let op_str = emission_operand(op, OperandSize::Dword);
            format!("idivl {}", op_str)
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
        codegen::Instruction::AllocateStack(i) => format!("subq ${}, %rsp", i),
        codegen::Instruction::DeallocateStack(i) => format!("addq ${}, %rsp", i),
        codegen::Instruction::Push(op) => {
            let op_str = emission_operand(op, OperandSize::Qword);
            format!("pushq {}", op_str)
        }
        codegen::Instruction::Call(label) => {
            if let Some(symbol_info) = symbol_table.map.get(label) {
                match symbol_info.metadata {
                    SymbolMetadata::Function { defined } => {
                        if defined {
                            format!("call {}", label)
                        } else {
                            format!("call {}@PLT", label)
                        }
                    }
                    _ => panic!("Attempting to call a non-function symbol: {}", label),
                }
            } else {
                panic!("Undefined symbol: {}", label);
            }
        }
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
    }
}

pub fn emission_operand(asm_operand: &codegen::Operand, size: OperandSize) -> String {
    match asm_operand {
        codegen::Operand::Imm(i) => format!("${}", i),
        codegen::Operand::Reg(r) => emission_register(r, size),
        codegen::Operand::Stack(i) => format!("{}(%rbp)", i),
        _ => panic!("Unexpected operand type in emission"),
    }
}

pub fn emission_unary_operator(asm_unop: &codegen::UnaryOperator) -> String {
    match asm_unop {
        codegen::UnaryOperator::Not => String::from("notl"),
        codegen::UnaryOperator::Neg => String::from("negl"),
    }
}

pub fn emission_binary_operator(asm_unop: &codegen::BinaryOperator) -> String {
    match asm_unop {
        codegen::BinaryOperator::Add => String::from("addl"),
        codegen::BinaryOperator::Sub => String::from("subl"),
        codegen::BinaryOperator::Mult => String::from("imull"),
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
    }
}
