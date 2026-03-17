use crate::codegen;

pub fn emission_program(asm_program: &codegen::Program) -> String {
    let codegen::Program(asm_function) = asm_program;

    format!(
        "{}    .section .note.GNU-stack,\"\",@progbits\n",
        &emission_function(asm_function)
    )
}

pub fn emission_function(asm_function: &codegen::Function) -> String {
    let codegen::Function(name, instructions) = asm_function;

    let mut function = String::new();

    function.push_str(&format!("    .globl {}\n", name));
    function.push_str(&format!("{}:\n", name));

    // Prologue
    function.push_str("    pushq %rbp\n");
    function.push_str("    movq %rsp, %rbp\n");

    for instruction in instructions {
        let instruction_str = emission_instruction(instruction);
        function.push_str(&format!("    {}\n", instruction_str));
    }

    function
}

pub fn emission_instruction(asm_instructions: &codegen::Instruction) -> String {
    match asm_instructions {
        codegen::Instruction::Mov(src, dst) => {
            let src_str = emission_operand(src);
            let dst_str = emission_operand(dst);
            format!("movl {},{}", src_str, dst_str)
        }
        codegen::Instruction::Ret => {
            format!("movq %rbp, %rsp\n    popq %rbp\n    ret")
        }
        codegen::Instruction::Unary(unop, op) => {
            let unop_str = emission_unary_operator(unop);
            let op_str = emission_operand(op);
            format!("{} {}", unop_str, op_str)
        }
        codegen::Instruction::AllocateStack(i) => format!("subq ${}, %rsp", i),
    }
}

pub fn emission_operand(asm_operand: &codegen::Operand) -> String {
    match asm_operand {
        codegen::Operand::Imm(i) => format!("${}", i),
        codegen::Operand::Reg(codegen::Reg::AX) => String::from("%eax"),
        codegen::Operand::Reg(codegen::Reg::R10) => String::from("%r10d"),
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
