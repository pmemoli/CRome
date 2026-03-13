use crate::parser;

// ASM AST Specification
// program = Program(function_definition)
// function_definition = Function(identifier name, instruction* instructions)
// instruction = Mov(operand src, operand dst) | Ret
// operand = Imm(int) | Register

#[derive(Debug)]
pub struct Program(pub Function);

#[derive(Debug)]
pub struct Function(pub String, pub Vec<Instruction>);

#[derive(Debug)]
pub enum Instruction {
    Mov(Operand, Operand),
    Ret,
}

#[derive(Debug)]
pub enum Operand {
    Imm(i32),
    Register,
}

pub fn codegen_program(ast_program: &parser::Program) -> Program {
    let parser::Program(ast_function) = ast_program;
    let asm_function = codegen_function(ast_function);
    Program(asm_function)
}

pub fn codegen_function(ast_function: &parser::Function) -> Function {
    let parser::Function(ast_name, ast_body) = ast_function;
    let asm_instructions = codegen_statement(ast_body);
    Function(ast_name.to_string(), asm_instructions)
}

pub fn codegen_statement(ast_statement: &parser::Statement) -> Vec<Instruction> {
    let mut instructions = Vec::new();

    match ast_statement {
        parser::Statement::Return(exp) => {
            let src = codegen_expression(exp);
            let dst = Operand::Register;
            instructions.push(Instruction::Mov(src, dst));
            instructions.push(Instruction::Ret);
        }
    }
    instructions
}

pub fn codegen_expression(ast_expression: &parser::Expr) -> Operand {
    match ast_expression {
        parser::Expr::Constant(i) => Operand::Imm(*i),
        _ => panic!("Malformed Expression"),
    }
}
