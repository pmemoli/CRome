use super::*;
use crate::symbol::{SymbolMetadata, SymbolTable};

// Second pass: Replace Pseudo(identifier) with Stack(int)
pub fn resolve_pseudo_registers_program(
    program: &Program,
    symbol_table: &mut SymbolTable,
) -> Program {
    let Program(functions) = program;

    let mut resolved_functions = Vec::new();
    for function in functions {
        let resolved_function = resolve_pseudo_registers_function(function, symbol_table);
        resolved_functions.push(resolved_function);
    }
    Program(resolved_functions)
}

pub fn resolve_pseudo_registers_function(
    function: &Function,
    symbol_table: &mut SymbolTable,
) -> Function {
    let Function(identifier, instructions) = function;

    let mut resolved_instructions = Vec::new();
    for instruction in instructions {
        let resolved_instruction = resolve_pseudo_registers_instruction(instruction, symbol_table);
        resolved_instructions.push(resolved_instruction);
    }

    Function(identifier.clone(), resolved_instructions)
}

pub fn resolve_pseudo_registers_instruction(
    instruction: &Instruction,
    symbol_table: &mut SymbolTable,
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

pub fn resolve_pseudo_registers_operand(
    operand: &Operand,
    symbol_table: &mut SymbolTable,
) -> Operand {
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
