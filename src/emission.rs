use crate::codegen;

pub fn emission_program(asm_program: &codegen::Program) -> String {
    let codegen::Program(asm_function) = asm_program;

    let mut program = String::new();

    program.push_str(&emission_function(asm_function));
    program.push_str("    .section .note.GNU-stack,\"\",@progbits");

    program
}

pub fn emission_function(asm_function: &codegen::Function) -> String {}

pub fn emission_instruction(asm_instructions: &codegen::Instruction) -> String {}

pub fn emission_operand(asm_operand: &codegen::Operand) -> String {}
