use crate::codegen;

pub fn emission_program(asm_program: &codegen::Program) -> String {
    let codegen::Program(asm_function) = asm_program;

    let mut program = String::new();

    program.push_str(&emission_function(asm_function));
    program.push_str("    .section .note.GNU-stack,\"\",@progbits\n");

    program
}

pub fn emission_function(asm_function: &codegen::Function) -> String {
    let codegen::Function(name, instructions) = asm_function;

    let mut function = String::new();

    function.push_str(&format!("    .globl {}\n", name));
    function.push_str(&format!("{}:\n", name));

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
        codegen::Instruction::Ret => String::from("ret"),
    }
}

pub fn emission_operand(asm_operand: &codegen::Operand) -> String {
    match asm_operand {
        codegen::Operand::Imm(i) => format!("${}", i),
        codegen::Operand::Register => String::from("%eax"),
    }
}
