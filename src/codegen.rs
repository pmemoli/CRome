use crate::tacky;
use std::collections::HashMap;

// program = Program(function_definition)
#[derive(Debug)]
pub struct Program(pub Function);

// function_definition = Function(identifier name, instruction* instructions)
#[derive(Debug)]
pub struct Function(pub String, pub Vec<Instruction>);

// instruction = Mov(operand src, operand dst)
// | Unary(unary_operator, operand)
// | AllocateStack(int)
// | Ret
#[derive(Debug, Clone)]
pub enum Instruction {
    Mov(Operand, Operand),
    Unary(UnaryOperator, Operand),
    AllocateStack(i32),
    Ret,
}

// unary_operator = Neg | Not
#[derive(Debug, Clone)]
pub enum UnaryOperator {
    Neg,
    Not,
}

// operand = Imm(int) | Reg(reg) | Pseudo(identifier) | Stack(int)
#[derive(Debug, Clone)]
pub enum Operand {
    Imm(i32),
    Reg(Reg),
    Pseudo(String),
    Stack(i32),
}

// reg = AX | R10
#[derive(Debug, Clone)]
pub enum Reg {
    AX,
    R10,
}

// First pass: Convert Tacky to ASM AST (with temp variables as pseudoregisters)
pub fn tacky_program_to_asm(tacky_program: &tacky::Program) -> Program {
    let tacky::Program(tacky_function) = tacky_program;
    let asm_function = tacky_function_to_asm(tacky_function);
    Program(asm_function)
}

pub fn tacky_function_to_asm(tacky_function: &tacky::Function) -> Function {
    let tacky::Function(tacky_identifier, tacky_instructions) = tacky_function;

    let identifier = tacky_identifier.to_string();
    let asm_instructions = tacky_instructions
        .into_iter()
        .flat_map(tacky_instruction_to_asm)
        .collect();

    Function(identifier, asm_instructions)
}

pub fn tacky_instruction_to_asm(tacky_function: &tacky::Instruction) -> Vec<Instruction> {
    match tacky_function {
        tacky::Instruction::Return(val) => {
            let src_asm_op = tacky_val_to_asm(val);
            let dst_asm_op = Operand::Reg(Reg::AX);
            vec![Instruction::Mov(src_asm_op, dst_asm_op), Instruction::Ret]
        }

        tacky::Instruction::Unary(unop, src, dst) => {
            let unop_asm_op = tacky_unop_to_asm(unop);
            let src_asm_op = tacky_val_to_asm(src);
            let dst_asm_op = tacky_val_to_asm(dst);
            vec![
                Instruction::Mov(src_asm_op, dst_asm_op.clone()),
                Instruction::Unary(unop_asm_op, dst_asm_op),
            ]
        }
    }
}

pub fn tacky_val_to_asm(tacky_function: &tacky::Val) -> Operand {
    match tacky_function {
        tacky::Val::Constant(i) => Operand::Imm(*i),
        tacky::Val::Var(s) => Operand::Pseudo(s.to_string()),
    }
}

pub fn tacky_unop_to_asm(tacky_unop: &tacky::UnaryOperator) -> UnaryOperator {
    match tacky_unop {
        tacky::UnaryOperator::Complement => UnaryOperator::Not,
        tacky::UnaryOperator::Negate => UnaryOperator::Neg,
    }
}

// Second pass: Replace Pseudo(identifier) with Stack(int)
pub fn resolve_pseudo_registers_program(program: &Program) -> (Program, i32) {
    // Returns the resolved program and the total stack size needed for temp variables.

    let Program(function) = program;

    let mut identifier_map = HashMap::new();

    let resolved_function = resolve_pseudo_registers_function(function, &mut identifier_map);
    (
        Program(resolved_function),
        (identifier_map.len() as i32) * 4,
    )
}

pub fn resolve_pseudo_registers_function(
    function: &Function,
    identifier_map: &mut HashMap<String, i32>,
) -> Function {
    let Function(identifier, instructions) = function;

    let resolved_instructions = instructions
        .into_iter()
        .flat_map(|i| resolve_pseudo_registers_instruction(i, identifier_map))
        .collect();

    Function(identifier.clone(), resolved_instructions)
}

pub fn resolve_pseudo_registers_instruction(
    instruction: &Instruction,
    identifier_map: &mut HashMap<String, i32>,
) -> Vec<Instruction> {
    match instruction {
        Instruction::Mov(src, dst) => {
            let resolved_src = resolve_pseudo_registers_operand(src, identifier_map);
            let resolved_dst = resolve_pseudo_registers_operand(dst, identifier_map);
            vec![Instruction::Mov(resolved_src, resolved_dst)]
        }
        Instruction::Unary(unop, op) => {
            let resolved_op = resolve_pseudo_registers_operand(op, identifier_map);
            vec![Instruction::Unary(unop.clone(), resolved_op)]
        }
        i => vec![i.clone()],
    }
}

pub fn resolve_pseudo_registers_operand(
    operand: &Operand,
    identifier_map: &mut HashMap<String, i32>,
) -> Operand {
    match operand {
        Operand::Pseudo(s) => match identifier_map.get(s) {
            Some(i) => Operand::Stack(*i),
            None => {
                // Each temp variable (only primitive value is i32) gets assigned 4 bytes.
                let new_stack_offset = -((identifier_map.len() + 1) as i32 * 4);
                identifier_map.insert(s.clone(), new_stack_offset);
                Operand::Stack(new_stack_offset)
            }
        },
        o => o.clone(),
    }
}

// Third pass: Allocate stack and fix instruction operands
pub fn allocate_stack_program(program: &Program, stack_size: i32) -> Program {
    let Program(function) = program;

    let allocated_function = allocate_stack_function(function, stack_size);
    Program(allocated_function)
}

pub fn allocate_stack_function(function: &Function, stack_size: i32) -> Function {
    let Function(identifier, instructions) = function;

    let mut allocated_instructions = vec![Instruction::AllocateStack(stack_size)];
    let fixed_instructions = instructions
        .into_iter()
        .flat_map(allocate_stack_instruction);
    allocated_instructions.extend(fixed_instructions);

    Function(identifier.clone(), allocated_instructions)
}

pub fn allocate_stack_instruction(instruction: &Instruction) -> Vec<Instruction> {
    match instruction {
        Instruction::Mov(op_a @ Operand::Stack(_), op_b @ Operand::Stack(_)) => {
            vec![
                Instruction::Mov(op_a.clone(), Operand::Reg(Reg::R10)),
                Instruction::Mov(Operand::Reg(Reg::R10), op_b.clone()),
            ]
        }
        i => vec![i.clone()],
    }
}
