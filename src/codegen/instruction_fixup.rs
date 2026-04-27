use super::*;

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
