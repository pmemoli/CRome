use super::*;
use crate::symbol::{SymbolMetadata, SymbolTable};

// Second pass: Replace Pseudo(identifier) with Stack(int)
pub fn resolve_pseudo_registers_program(program: &Program, symbol_table: &SymbolTable) -> Program {
    let Program(top_level_structs) = program;

    let mut resolved_top_level_structs = Vec::new();
    for top_level_struct in top_level_structs {
        let resolved_top_level_struct = resolve_top_level(top_level_struct, symbol_table);
        resolved_top_level_structs.push(resolved_top_level_struct);
    }
    Program(resolved_top_level_structs)
}

pub fn resolve_top_level(top_level: &TopLevel, symbol_table: &SymbolTable) -> TopLevel {
    match top_level {
        TopLevel::Function(identifier, global, instructions) => {
            let mut resolved_instructions = Vec::new();
            for instruction in instructions {
                resolved_instructions.push(resolve_pseudo_registers_instruction(
                    instruction,
                    symbol_table,
                ));
            }

            TopLevel::Function(identifier.clone(), *global, resolved_instructions)
        }
        t => t.clone(),
    }
}

pub fn resolve_pseudo_registers_instruction(
    instruction: &Instruction,
    symbol_table: &SymbolTable,
) -> Instruction {
    match instruction {
        Instruction::Mov(src, dst) => {
            let resolved_src = resolve_pseudo_registers_operand(src, symbol_table);
            let resolved_dst = resolve_pseudo_registers_operand(dst, symbol_table);
            Instruction::Mov(resolved_src, resolved_dst)
        }
        Instruction::Unary(unop, op) => {
            let resolved_op = resolve_pseudo_registers_operand(op, symbol_table);
            Instruction::Unary(unop.clone(), resolved_op)
        }
        Instruction::Binary(binop, op_1, op_2) => {
            let resolved_op_1 = resolve_pseudo_registers_operand(op_1, symbol_table);
            let resolved_op_2 = resolve_pseudo_registers_operand(op_2, symbol_table);
            Instruction::Binary(binop.clone(), resolved_op_1, resolved_op_2)
        }
        Instruction::Idiv(op) => {
            let resolved_op = resolve_pseudo_registers_operand(op, symbol_table);
            Instruction::Idiv(resolved_op)
        }
        Instruction::Cmp(op_1, op_2) => {
            let resolved_op_1 = resolve_pseudo_registers_operand(op_1, symbol_table);
            let resolved_op_2 = resolve_pseudo_registers_operand(op_2, symbol_table);
            Instruction::Cmp(resolved_op_1, resolved_op_2)
        }
        Instruction::SetCC(condcode, op) => {
            let resolved_op = resolve_pseudo_registers_operand(op, symbol_table);
            Instruction::SetCC(condcode.clone(), resolved_op)
        }
        Instruction::Push(op) => {
            let resolved_op = resolve_pseudo_registers_operand(op, symbol_table);
            Instruction::Push(resolved_op)
        }
        i => i.clone(),
    }
}

pub fn resolve_pseudo_registers_operand(operand: &Operand, symbol_table: &SymbolTable) -> Operand {
    match operand {
        Operand::Pseudo(s) => {
            let symbol_info = symbol_table.map.get(s).unwrap();
            if let SymbolMetadata::Variable { stack_offset } = symbol_info.metadata {
                Operand::Stack(stack_offset.clone())
            } else {
                panic!("Expected variable symbol as pseudo operand, found function symbol");
            }
        }
        o => o.clone(),
    }
}
