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
                fixed_instructions.extend(instruction_fixup(instruction));
            }

            TopLevel::Function(identifier.clone(), *global, fixed_instructions)
        }
        t => t.clone(),
    }
}

pub fn instruction_fixup(instruction: &Instruction) -> Vec<Instruction> {
    instruction_fixup_large_imm(instruction)
        .iter()
        .flat_map(instruction_fixup_invalid_operands)
        .collect()
}

pub fn instruction_fixup_invalid_operands(instruction: &Instruction) -> Vec<Instruction> {
    match instruction {
        Instruction::Mov(ty, src, dst) if src.is_memory_operand() && dst.is_memory_operand() => {
            vec![
                Instruction::Mov(ty.clone(), src.clone(), Operand::Reg(Reg::R10)),
                Instruction::Mov(ty.clone(), Operand::Reg(Reg::R10), dst.clone()),
            ]
        }

        Instruction::Idiv(ty, op @ Operand::Imm(_)) => {
            vec![
                Instruction::Mov(ty.clone(), op.clone(), Operand::Reg(Reg::R10)),
                Instruction::Idiv(ty.clone(), Operand::Reg(Reg::R10)),
            ]
        }

        Instruction::Binary(
            binop @ BinaryOperator::Add | binop @ BinaryOperator::Sub,
            ty,
            src,
            dst,
        ) if src.is_memory_operand() && dst.is_memory_operand() => {
            vec![
                Instruction::Mov(ty.clone(), src.clone(), Operand::Reg(Reg::R10)),
                Instruction::Binary(
                    binop.clone(),
                    ty.clone(),
                    Operand::Reg(Reg::R10),
                    dst.clone(),
                ),
            ]
        }

        Instruction::Binary(BinaryOperator::Mult, ty, src, dst) if dst.is_memory_operand() => {
            vec![
                Instruction::Mov(ty.clone(), dst.clone(), Operand::Reg(Reg::R11)),
                Instruction::Binary(
                    BinaryOperator::Mult,
                    ty.clone(),
                    src.clone(),
                    Operand::Reg(Reg::R11),
                ),
                Instruction::Mov(ty.clone(), Operand::Reg(Reg::R11), dst.clone()),
            ]
        }

        Instruction::Cmp(ty, src, dst) if src.is_memory_operand() && dst.is_memory_operand() => {
            vec![
                Instruction::Mov(ty.clone(), src.clone(), Operand::Reg(Reg::R10)),
                Instruction::Cmp(ty.clone(), Operand::Reg(Reg::R10), dst.clone()),
            ]
        }

        Instruction::Cmp(ty, src, dst @ Operand::Imm(_)) => {
            vec![
                Instruction::Mov(ty.clone(), dst.clone(), Operand::Reg(Reg::R11)),
                Instruction::Cmp(ty.clone(), src.clone(), Operand::Reg(Reg::R11)),
            ]
        }

        Instruction::Push(op) if op.is_memory_operand() => {
            vec![
                Instruction::Mov(AssemblyType::Quadword, op.clone(), Operand::Reg(Reg::R10)),
                Instruction::Push(Operand::Reg(Reg::R10)),
            ]
        }

        // We only consider movslq (src is always a longword)
        Instruction::Movsx(src @ Operand::Imm(_), dst) if dst.is_memory_operand() => {
            vec![
                Instruction::Mov(AssemblyType::Longword, src.clone(), Operand::Reg(Reg::R10)),
                Instruction::Movsx(Operand::Reg(Reg::R10), Operand::Reg(Reg::R11)),
                Instruction::Mov(AssemblyType::Quadword, Operand::Reg(Reg::R11), dst.clone()),
            ]
        }

        Instruction::Movsx(src, dst) if src.is_memory_operand() && dst.is_memory_operand() => {
            vec![
                Instruction::Mov(AssemblyType::Longword, src.clone(), Operand::Reg(Reg::R10)),
                Instruction::Movsx(Operand::Reg(Reg::R10), Operand::Reg(Reg::R11)),
                Instruction::Mov(AssemblyType::Quadword, Operand::Reg(Reg::R11), dst.clone()),
            ]
        }

        i => vec![i.clone()],
    }
}

// Quadword versions of add, sub, imul, cmp and push can't handle imm values outside of ints (need fixup)
pub fn instruction_fixup_large_imm(instruction: &Instruction) -> Vec<Instruction> {
    match instruction {
        Instruction::Mov(AssemblyType::Quadword, src @ Operand::Imm(imm), dst)
            if src.is_large_imm_operand() =>
        {
            vec![
                Instruction::Mov(AssemblyType::Quadword, src.clone(), Operand::Reg(Reg::R10)),
                Instruction::Mov(AssemblyType::Quadword, Operand::Reg(Reg::R10), dst.clone()),
            ]
        }

        Instruction::Binary(
            binop @ BinaryOperator::Add
            | binop @ BinaryOperator::Sub
            | binop @ BinaryOperator::Mult,
            AssemblyType::Quadword,
            src @ Operand::Imm(imm),
            dst,
        ) if src.is_large_imm_operand() => {
            vec![
                Instruction::Mov(AssemblyType::Quadword, src.clone(), Operand::Reg(Reg::R10)),
                Instruction::Binary(
                    binop.clone(),
                    AssemblyType::Quadword,
                    Operand::Reg(Reg::R10),
                    dst.clone(),
                ),
            ]
        }

        Instruction::Cmp(AssemblyType::Quadword, src @ Operand::Imm(imm), dst)
            if src.is_large_imm_operand() =>
        {
            vec![
                Instruction::Mov(AssemblyType::Quadword, src.clone(), Operand::Reg(Reg::R10)),
                Instruction::Cmp(AssemblyType::Quadword, Operand::Reg(Reg::R10), dst.clone()),
            ]
        }

        Instruction::Push(src @ Operand::Imm(imm)) if src.is_large_imm_operand() => {
            vec![
                Instruction::Mov(AssemblyType::Quadword, src.clone(), Operand::Reg(Reg::R10)),
                Instruction::Push(Operand::Reg(Reg::R10)),
            ]
        }

        i => vec![i.clone()],
    }
}
