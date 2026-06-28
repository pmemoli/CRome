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

// Very weird logic, not modular and doesn't scale at all
pub fn instruction_fixup(instruction: &Instruction) -> Vec<Instruction> {
    instruction_fixup_large_imm(instruction)
        .iter()
        .flat_map(instruction_fixup_invalid_operands)
        .collect()
}

pub fn instruction_fixup_invalid_operands(instruction: &Instruction) -> Vec<Instruction> {
    match instruction {
        Instruction::Mov(ty, src, dst) if src.is_memory_operand() && dst.is_memory_operand() => {
            let src_reg = src_register(ty);
            vec![
                Instruction::Mov(ty.clone(), src.clone(), src_reg.clone()),
                Instruction::Mov(ty.clone(), src_reg.clone(), dst.clone()),
            ]
        }

        Instruction::Idiv(ty, op @ Operand::Imm(_)) => {
            let src_reg = src_register(ty);
            vec![
                Instruction::Mov(ty.clone(), op.clone(), src_reg.clone()),
                Instruction::Idiv(ty.clone(), src_reg),
            ]
        }

        Instruction::Div(ty, op @ Operand::Imm(_)) => {
            let src_reg = src_register(ty);
            vec![
                Instruction::Mov(ty.clone(), op.clone(), src_reg.clone()),
                Instruction::Div(ty.clone(), src_reg),
            ]
        }

        // Double binops need to have dst as a register (checked before integer mem-mem)
        Instruction::Binary(binop, AssemblyType::Double, src, dst)
            if !dst.is_register_operand() =>
        {
            let dst_reg = dst_register(&AssemblyType::Double);
            vec![
                Instruction::Mov(AssemblyType::Double, dst.clone(), dst_reg.clone()),
                Instruction::Binary(
                    binop.clone(),
                    AssemblyType::Double,
                    src.clone(),
                    dst_reg.clone(),
                ),
                Instruction::Mov(AssemblyType::Double, dst_reg, dst.clone()),
            ]
        }

        Instruction::Binary(
            binop @ BinaryOperator::Add | binop @ BinaryOperator::Sub,
            ty,
            src,
            dst,
        ) if src.is_memory_operand() && dst.is_memory_operand() => {
            let src_reg = src_register(ty);
            vec![
                Instruction::Mov(ty.clone(), src.clone(), src_reg.clone()),
                Instruction::Binary(binop.clone(), ty.clone(), src_reg, dst.clone()),
            ]
        }

        Instruction::Binary(BinaryOperator::Mult, ty, src, dst) if dst.is_memory_operand() => {
            let dst_reg = dst_register(ty);
            vec![
                Instruction::Mov(ty.clone(), dst.clone(), dst_reg.clone()),
                Instruction::Binary(
                    BinaryOperator::Mult,
                    ty.clone(),
                    src.clone(),
                    dst_reg.clone(),
                ),
                Instruction::Mov(ty.clone(), dst_reg, dst.clone()),
            ]
        }

        // Double cmp needs to have dst as a register
        Instruction::Cmp(AssemblyType::Double, src, dst) if !dst.is_register_operand() => {
            let dst_reg = dst_register(&AssemblyType::Double);
            vec![
                Instruction::Mov(AssemblyType::Double, dst.clone(), dst_reg.clone()),
                Instruction::Cmp(AssemblyType::Double, src.clone(), dst_reg),
            ]
        }

        Instruction::Cmp(ty, src, dst) if src.is_memory_operand() && dst.is_memory_operand() => {
            let src_reg = src_register(ty);

            vec![
                Instruction::Mov(ty.clone(), src.clone(), src_reg.clone()),
                Instruction::Cmp(ty.clone(), src_reg.clone(), dst.clone()),
            ]
        }

        Instruction::Cmp(ty, src, dst @ Operand::Imm(_)) => {
            let dst_reg = dst_register(ty);
            vec![
                Instruction::Mov(ty.clone(), dst.clone(), dst_reg.clone()),
                Instruction::Cmp(ty.clone(), src.clone(), dst_reg),
            ]
        }

        Instruction::Push(op) if op.is_memory_operand() => {
            let src_reg = src_register(&AssemblyType::Quadword);
            vec![
                Instruction::Mov(AssemblyType::Quadword, op.clone(), src_reg.clone()),
                Instruction::Push(src_reg),
            ]
        }

        // These only correspond to extending longwords to quadwords (32 to 64 bits)
        Instruction::Movsx(src @ Operand::Imm(_), dst) if dst.is_memory_operand() => {
            let src_reg = src_register(&AssemblyType::Longword);
            let dst_reg = dst_register(&AssemblyType::Quadword);
            vec![
                Instruction::Mov(AssemblyType::Longword, src.clone(), src_reg.clone()),
                Instruction::Movsx(src_reg, dst_reg.clone()),
                Instruction::Mov(AssemblyType::Quadword, dst_reg, dst.clone()),
            ]
        }

        Instruction::Movsx(src, dst) if src.is_memory_operand() && dst.is_memory_operand() => {
            let src_reg = src_register(&AssemblyType::Longword);
            let dst_reg = dst_register(&AssemblyType::Quadword);
            vec![
                Instruction::Mov(AssemblyType::Longword, src.clone(), src_reg.clone()),
                Instruction::Movsx(src_reg, dst_reg.clone()),
                Instruction::Mov(AssemblyType::Quadword, dst_reg, dst.clone()),
            ]
        }

        Instruction::MovZeroExtend(src, dst @ Operand::Reg(_)) => {
            vec![Instruction::Mov(
                AssemblyType::Longword,
                src.clone(),
                dst.clone(),
            )]
        }

        Instruction::MovZeroExtend(src, dst) if dst.is_memory_operand() => {
            let dst_reg = dst_register(&AssemblyType::Quadword);
            vec![
                Instruction::Mov(AssemblyType::Longword, src.clone(), dst_reg.clone()),
                Instruction::Mov(AssemblyType::Quadword, dst_reg, dst.clone()),
            ]
        }

        // Signed integer to double need to have dst as a register, and src can't be const
        Instruction::IntToFloat(ty, src, dst)
            if !dst.is_register_operand() || matches!(src, Operand::Imm(_)) =>
        {
            let mut out = Vec::new();

            let src = if matches!(src, Operand::Imm(_)) {
                let src_reg = src_register(ty);
                out.push(Instruction::Mov(ty.clone(), src.clone(), src_reg.clone()));
                src_reg
            } else {
                src.clone()
            };

            if dst.is_register_operand() {
                out.push(Instruction::IntToFloat(ty.clone(), src, dst.clone()));
            } else {
                let dst_reg = dst_register(&AssemblyType::Double);
                out.push(Instruction::IntToFloat(ty.clone(), src, dst_reg.clone()));
                out.push(Instruction::Mov(AssemblyType::Double, dst_reg, dst.clone()));
            }

            out
        }

        // Unsigned integer to double need to have dst as a register, and src can't be const
        Instruction::UIntToFloat(ty, src, dst)
            if !dst.is_register_operand() || matches!(src, Operand::Imm(_)) =>
        {
            let mut out = Vec::new();

            let src = if matches!(src, Operand::Imm(_)) {
                let src_reg = src_register(ty);
                out.push(Instruction::Mov(ty.clone(), src.clone(), src_reg.clone()));
                src_reg
            } else {
                src.clone()
            };

            if dst.is_register_operand() {
                out.push(Instruction::UIntToFloat(ty.clone(), src, dst.clone()));
            } else {
                let dst_reg = dst_register(&AssemblyType::Double);
                out.push(Instruction::UIntToFloat(ty.clone(), src, dst_reg.clone()));
                out.push(Instruction::Mov(AssemblyType::Double, dst_reg, dst.clone()));
            }

            out
        }

        // Double to signed integer need to have dst as a register
        Instruction::FloatToInt(ty, src, dst) if !dst.is_register_operand() => {
            let dst_reg = dst_register(ty);
            vec![
                Instruction::FloatToInt(ty.clone(), src.clone(), dst_reg.clone()),
                Instruction::Mov(ty.clone(), dst_reg, dst.clone()),
            ]
        }

        // Double to unsigned integer need to have dst as a register
        Instruction::FloatToUInt(ty, src, dst) if !dst.is_register_operand() => {
            let dst_reg = dst_register(ty);
            vec![
                Instruction::FloatToUInt(ty.clone(), src.clone(), dst_reg.clone()),
                Instruction::Mov(ty.clone(), dst_reg, dst.clone()),
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

pub fn src_register(ty: &AssemblyType) -> Operand {
    match ty {
        AssemblyType::Double | AssemblyType::Float => Operand::Reg(Reg::XMM14),
        _ => Operand::Reg(Reg::R10),
    }
}

pub fn dst_register(ty: &AssemblyType) -> Operand {
    match ty {
        AssemblyType::Double | AssemblyType::Float => Operand::Reg(Reg::XMM15),
        _ => Operand::Reg(Reg::R11),
    }
}
