use super::*;

pub fn instruction_fixup_program(program: &Program) -> Program {
    let Program(top_level_structs) = program;

    let mut fixed_top_level = Vec::new();
    for top_level_struct in top_level_structs {
        fixed_top_level.push(instruction_fixup_top_level(top_level_struct));
    }

    Program(fixed_top_level)
}

pub fn instruction_fixup_top_level(top_level: &TopLevel) -> TopLevel {
    match top_level {
        TopLevel::Function(identifier, global, instructions) => {
            let mut fixed_instructions = Vec::new();
            for instruction in instructions {
                fixed_instructions.extend(instruction_fixup_instruction(instruction));
            }

            TopLevel::Function(identifier.clone(), *global, fixed_instructions)
        }
        t => t.clone(),
    }
}

pub fn instruction_fixup_instruction(instruction: &Instruction) -> Vec<Instruction> {
    match instruction {
        Instruction::Mov(src, dst) if src.is_memory_operand() && dst.is_memory_operand() => {
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
            src,
            dst,
        ) if src.is_memory_operand() && dst.is_memory_operand() => {
            vec![
                Instruction::Mov(src.clone(), Operand::Reg(Reg::R10)),
                Instruction::Binary(binop.clone(), Operand::Reg(Reg::R10), dst.clone()),
            ]
        }

        Instruction::Binary(BinaryOperator::Mult, src, dst) if dst.is_memory_operand() => {
            vec![
                Instruction::Mov(dst.clone(), Operand::Reg(Reg::R11)),
                Instruction::Binary(BinaryOperator::Mult, src.clone(), Operand::Reg(Reg::R11)),
                Instruction::Mov(Operand::Reg(Reg::R11), dst.clone()),
            ]
        }

        Instruction::Cmp(src, dst) if src.is_memory_operand() && dst.is_memory_operand() => {
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

        Instruction::Push(op) if op.is_memory_operand() => {
            vec![
                Instruction::Mov(op.clone(), Operand::Reg(Reg::R10)),
                Instruction::Push(Operand::Reg(Reg::R10)),
            ]
        }

        i => vec![i.clone()],
    }
}
