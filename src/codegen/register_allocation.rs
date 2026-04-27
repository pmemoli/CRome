use super::*;

// Second pass: Replace Pseudo(identifier) with Stack(int)
pub fn resolve_pseudo_registers_program(program: &mut Program, symbol_table: &mut SymbolTable) {
    let Program(function) = program;
    resolve_pseudo_registers_function(function, symbol_table);
}

pub fn resolve_pseudo_registers_function(function: &mut Function, symbol_table: &mut SymbolTable) {
    let Function(_, instructions) = function;

    instructions
        .iter_mut()
        .for_each(|i| resolve_pseudo_registers_instruction(i, symbol_table));
}

pub fn resolve_pseudo_registers_instruction(
    instruction: &mut Instruction,
    symbol_table: &mut SymbolTable,
) {
    match instruction {
        Instruction::Mov(src, dst) => {
            resolve_pseudo_registers_operand(src, symbol_table);
            resolve_pseudo_registers_operand(dst, symbol_table);
        }
        Instruction::Unary(_, op) => {
            resolve_pseudo_registers_operand(op, symbol_table);
        }
        Instruction::Binary(_, op_1, op_2) => {
            resolve_pseudo_registers_operand(op_1, symbol_table);
            resolve_pseudo_registers_operand(op_2, symbol_table);
        }
        Instruction::Idiv(op) => {
            resolve_pseudo_registers_operand(op, symbol_table);
        }
        Instruction::Cmp(op_1, op_2) => {
            resolve_pseudo_registers_operand(op_1, symbol_table);
            resolve_pseudo_registers_operand(op_2, symbol_table);
        }
        Instruction::SetCC(_, op) => {
            resolve_pseudo_registers_operand(op, symbol_table);
        }
        _ => {}
    }
}

pub fn resolve_pseudo_registers_operand(operand: &mut Operand, symbol_table: &mut SymbolTable) {
    match operand {
        Operand::Pseudo(s) => {
            let symbol_info = symbol_table.get(s);
            let stack_offset = symbol_info.stack_offset;
            *operand = Operand::Stack(stack_offset);
        }
        _ => {}
    }
}
