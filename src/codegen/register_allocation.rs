use super::*;
use crate::symbol::{SymbolMetadata, SymbolTable};

use std::collections::HashMap;

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
            let mut automatic_variable_counter: HashMap<String, isize> = HashMap::new();

            for instruction in instructions {
                resolved_instructions.push(resolve_pseudo_registers_instruction(
                    instruction,
                    symbol_table,
                    &mut automatic_variable_counter,
                ));
            }

            // Allocate stack space for automatic variables
            let stack_size = (automatic_variable_counter.len() * 4).next_multiple_of(16);
            resolved_instructions.insert(0, Instruction::AllocateStack(stack_size));

            TopLevel::Function(identifier.clone(), *global, resolved_instructions)
        }
        t => t.clone(),
    }
}

pub fn resolve_pseudo_registers_instruction(
    instruction: &Instruction,
    symbol_table: &SymbolTable,
    automatic_variable_counter: &mut HashMap<String, isize>,
) -> Instruction {
    match instruction {
        Instruction::Mov(src, dst) => {
            let resolved_src =
                resolve_pseudo_registers_operand(src, symbol_table, automatic_variable_counter);
            let resolved_dst =
                resolve_pseudo_registers_operand(dst, symbol_table, automatic_variable_counter);
            Instruction::Mov(resolved_src, resolved_dst)
        }
        Instruction::Unary(unop, op) => {
            let resolved_op =
                resolve_pseudo_registers_operand(op, symbol_table, automatic_variable_counter);
            Instruction::Unary(unop.clone(), resolved_op)
        }
        Instruction::Binary(binop, op_1, op_2) => {
            let resolved_op_1 =
                resolve_pseudo_registers_operand(op_1, symbol_table, automatic_variable_counter);
            let resolved_op_2 =
                resolve_pseudo_registers_operand(op_2, symbol_table, automatic_variable_counter);
            Instruction::Binary(binop.clone(), resolved_op_1, resolved_op_2)
        }
        Instruction::Idiv(op) => {
            let resolved_op =
                resolve_pseudo_registers_operand(op, symbol_table, automatic_variable_counter);
            Instruction::Idiv(resolved_op)
        }
        Instruction::Cmp(op_1, op_2) => {
            let resolved_op_1 =
                resolve_pseudo_registers_operand(op_1, symbol_table, automatic_variable_counter);
            let resolved_op_2 =
                resolve_pseudo_registers_operand(op_2, symbol_table, automatic_variable_counter);
            Instruction::Cmp(resolved_op_1, resolved_op_2)
        }
        Instruction::SetCC(condcode, op) => {
            let resolved_op =
                resolve_pseudo_registers_operand(op, symbol_table, automatic_variable_counter);
            Instruction::SetCC(condcode.clone(), resolved_op)
        }
        Instruction::Push(op) => {
            let resolved_op =
                resolve_pseudo_registers_operand(op, symbol_table, automatic_variable_counter);
            Instruction::Push(resolved_op)
        }
        i => i.clone(),
    }
}

pub fn resolve_pseudo_registers_operand(
    operand: &Operand,
    symbol_table: &SymbolTable,
    automatic_variable_counter: &mut HashMap<String, isize>,
) -> Operand {
    match operand {
        Operand::Pseudo(s) => {
            let symbol_info = symbol_table.map.get(s).unwrap();

            match symbol_info.metadata {
                SymbolMetadata::StaticVariable { .. } => Operand::Data(s.clone()),
                SymbolMetadata::LocalVariable => {
                    if let Some(offset) = automatic_variable_counter.get(s) {
                        Operand::Stack(-(*offset))
                    } else {
                        let offset = ((automatic_variable_counter.len() + 1) * 4) as isize;
                        automatic_variable_counter.insert(s.clone(), offset);
                        Operand::Stack(-offset)
                    }
                }
                _ => panic!("Expected variable symbol as pseudo operand, found function symbol"),
            }
        }
        o => o.clone(),
    }
}
