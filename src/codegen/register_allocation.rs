use std::collections::HashMap;

use super::*;
use crate::symbol::BackendSymbolMetadata;

// Second pass:
// 1. Replace Pseudo(identifier) with Stack(int)
// 2. Allocate stack space for automatic variables

pub struct LocalStack {
    variable_offsets: HashMap<String, usize>,
    total_stack_size: usize,
}

impl LocalStack {
    pub fn new() -> Self {
        LocalStack {
            variable_offsets: HashMap::new(),
            total_stack_size: 0,
        }
    }

    pub fn add(&mut self, variable: &String, ty: &AssemblyType) -> usize {
        self.total_stack_size += ty.size().next_multiple_of(ty.alignment());
        self.variable_offsets
            .insert(variable.clone(), self.total_stack_size);

        self.total_stack_size
    }

    pub fn get_offset(&self, variable: &String) -> Option<usize> {
        self.variable_offsets.get(variable).copied()
    }
}

pub fn resolve_pseudo_registers_program(
    program: &Program,
    symbol_table: &BackendSymbolTable,
) -> Program {
    let Program(top_level_structs) = program;

    let mut resolved_top_level_structs = Vec::new();
    for top_level_struct in top_level_structs {
        let resolved_top_level_struct = resolve_top_level(top_level_struct, symbol_table);
        resolved_top_level_structs.push(resolved_top_level_struct);
    }
    Program(resolved_top_level_structs)
}

pub fn resolve_top_level(top_level: &TopLevel, symbol_table: &BackendSymbolTable) -> TopLevel {
    match top_level {
        TopLevel::Function(identifier, global, instructions) => {
            let mut resolved_instructions = Vec::new();
            let mut local_stack = LocalStack::new();

            for instruction in instructions {
                resolved_instructions.push(resolve_pseudo_registers_instruction(
                    instruction,
                    symbol_table,
                    &mut local_stack,
                ));
            }

            // Allocate stack space for automatic variables
            let aligned_stack_size = local_stack.total_stack_size.next_multiple_of(16);
            resolved_instructions.insert(
                0,
                Instruction::Binary(
                    BinaryOperator::Sub,
                    AssemblyType::Quadword,
                    Operand::Imm(aligned_stack_size as i128),
                    Operand::Reg(Reg::SP),
                ),
            );

            TopLevel::Function(identifier.clone(), *global, resolved_instructions)
        }
        t => t.clone(),
    }
}

pub fn resolve_pseudo_registers_instruction(
    instruction: &Instruction,
    symbol_table: &BackendSymbolTable,
    local_stack: &mut LocalStack,
) -> Instruction {
    match instruction {
        Instruction::Mov(ty, src, dst) => {
            let resolved_src = resolve_pseudo_registers_operand(src, symbol_table, local_stack);
            let resolved_dst = resolve_pseudo_registers_operand(dst, symbol_table, local_stack);
            Instruction::Mov(ty.clone(), resolved_src, resolved_dst)
        }
        Instruction::Unary(unop, ty, op) => {
            let resolved_op = resolve_pseudo_registers_operand(op, symbol_table, local_stack);
            Instruction::Unary(unop.clone(), ty.clone(), resolved_op)
        }
        Instruction::Binary(binop, ty, op_1, op_2) => {
            let resolved_op_1 = resolve_pseudo_registers_operand(op_1, symbol_table, local_stack);
            let resolved_op_2 = resolve_pseudo_registers_operand(op_2, symbol_table, local_stack);
            Instruction::Binary(binop.clone(), ty.clone(), resolved_op_1, resolved_op_2)
        }
        Instruction::Idiv(ty, op) => {
            let resolved_op = resolve_pseudo_registers_operand(op, symbol_table, local_stack);
            Instruction::Idiv(ty.clone(), resolved_op)
        }
        Instruction::Div(ty, op) => {
            let resolved_op = resolve_pseudo_registers_operand(op, symbol_table, local_stack);
            Instruction::Div(ty.clone(), resolved_op)
        }
        Instruction::Cmp(ty, op_1, op_2) => {
            let resolved_op_1 = resolve_pseudo_registers_operand(op_1, symbol_table, local_stack);
            let resolved_op_2 = resolve_pseudo_registers_operand(op_2, symbol_table, local_stack);
            Instruction::Cmp(ty.clone(), resolved_op_1, resolved_op_2)
        }
        Instruction::SetCC(condcode, op) => {
            let resolved_op = resolve_pseudo_registers_operand(op, symbol_table, local_stack);
            Instruction::SetCC(condcode.clone(), resolved_op)
        }
        Instruction::Push(op) => {
            let resolved_op = resolve_pseudo_registers_operand(op, symbol_table, local_stack);
            Instruction::Push(resolved_op)
        }
        Instruction::Movsx(op_1, op_2) => {
            let resolved_op_1 = resolve_pseudo_registers_operand(op_1, symbol_table, local_stack);
            let resolved_op_2 = resolve_pseudo_registers_operand(op_2, symbol_table, local_stack);
            Instruction::Movsx(resolved_op_1, resolved_op_2)
        }
        Instruction::MovZeroExtend(op_1, op_2) => {
            let resolved_op_1 = resolve_pseudo_registers_operand(op_1, symbol_table, local_stack);
            let resolved_op_2 = resolve_pseudo_registers_operand(op_2, symbol_table, local_stack);
            Instruction::MovZeroExtend(resolved_op_1, resolved_op_2)
        }
        i => i.clone(),
    }
}

pub fn resolve_pseudo_registers_operand(
    operand: &Operand,
    symbol_table: &BackendSymbolTable,
    local_stack: &mut LocalStack,
) -> Operand {
    match operand {
        Operand::Pseudo(s) => {
            let symbol_info = symbol_table.map.get(s).unwrap();

            let BackendSymbolMetadata::ObjEntry { ty, is_static } = symbol_info else {
                panic!("Expected ObjEntry for variable {}", s);
            };

            if *is_static {
                Operand::Data(s.clone())
            } else {
                match local_stack.get_offset(s) {
                    Some(offset) => Operand::Stack(-(offset as isize)),
                    None => {
                        let offset = local_stack.add(s, ty);
                        Operand::Stack(-(offset as isize))
                    }
                }
            }
        }
        o => o.clone(),
    }
}
