use crate::tacky;

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
#[derive(Debug)]
pub enum Instruction {
    Mov(Operand, Operand),
    Unary(UnaryOperator, Operand),
    AllocateStack(i32),
    Ret,
}

// unary_operator = Neg | Not
#[derive(Debug)]
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
    let mut tacky_instructions = Vec::new();
    match tacky_function {
        tacky::Instruction::Return(val) => {
            let src_asm_op = tacky_val_to_asm(val);
            let dst_asm_op = Operand::Reg(Reg::AX);
            tacky_instructions.push(Instruction::Mov(src_asm_op, dst_asm_op));
            tacky_instructions.push(Instruction::Ret);
        }

        tacky::Instruction::Unary(unop, src, dst) => {
            let unop_asm_op = tacky_unop_to_asm(unop);
            let src_asm_op = tacky_val_to_asm(src);
            let dst_asm_op = tacky_val_to_asm(dst);
            tacky_instructions.push(Instruction::Mov(src_asm_op, dst_asm_op.clone()));
            tacky_instructions.push(Instruction::Unary(unop_asm_op, dst_asm_op));
        }
    }

    tacky_instructions
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
pub fn resolve_pseudo_registers_program(program: &Program) -> Program {
    let Program(function) = program;
    let resolved_function = resolve_pseudo_registers_function(function);
    Program(resolved_function)
}

pub fn resolve_pseudo_registers_function(function: &Function) -> Function {
    let Function(identifier, instructions) = function;
    let resolved_instructions = instructions
        .into_iter()
        .flat_map(resolve_pseudo_registers_instruction)
        .collect();

    Function(identifier.clone(), resolved_instructions)
}

pub fn resolve_pseudo_registers_instruction(instruction: &Instruction) -> Vec<Instruction> {}

pub fn resolve_pseudo_registers_operand(operand: &Operand) -> Operand {}
