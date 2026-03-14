use crate::parser;

// program = Program(function_definition)
#[derive(Debug)]
pub struct Program(Function);

// function_definition = Function(identifier, instruction* body)
#[derive(Debug)]
pub struct Function(String, Vec<Instruction>);

// instruction = Return(val) | Unary(unary_operator, val src, val dst)
#[derive(Debug)]
pub enum Instruction {
    Return(Val),
    Unary(UnaryOperator, Val, Val),
}

// val = Constant(int) | Var(identifier)
#[derive(Debug)]
pub enum Val {
    Constant(i32),
    Var(String),
}

// unary_operator = Complement | Negate
#[derive(Debug)]
pub enum UnaryOperator {
    Complement,
    Negate,
}

pub fn ast_program_to_tacky(ast_program: &parser::Program) -> Program {
    let parser::Program(ast_function) = ast_program;
    let tacky_function = ast_function_to_tacky(ast_function);
    Program(tacky_function)
}

pub fn ast_function_to_tacky(ast_function: &parser::Function) -> Function {
    let parser::Function(ast_identifier, ast_statement) = ast_function;

    let identifier = ast_identifier.to_string();
    let instructions = ast_statement_to_tacky(ast_statement);

    Function(identifier, instructions)
}

pub fn ast_statement_to_tacky(ast_statement: &parser::Statement) -> Vec<Instruction> {
    let mut instructions = Vec::new();

    let parser::Statement::Return(expr) = ast_statement;
    let expr_value = ast_expression_to_tacky(expr, &instructions);

    instructions.push(Instruction::Return(expr_value));

    instructions
}

pub fn ast_expression_to_tacky(
    ast_expr: &parser::Expr,
    instructions: &mut Vec<Instruction>,
) -> Val {
    match ast_expr {
        parser::Expr::Constant(i) => Val::Constant(*i),
        parser::Expr::Unary(op, expr) {
            let src = ast_expression_to_tacky(expr, instructions); 
            let dst_name = make_temporary();
            let dst = Val::Var(dst_name);
            let tacky_op = ast_unop_to_tacky(op);
            instructions.push(Instruction::Unary(tacky_op, src, dst));
            dst
        }
    }
}

pub fn ast_unop_to_tacky(ast_unop: &parser::UnaryOperator) -> UnaryOperator {
    match ast_unop {
        parser::UnaryOperator::Complement => UnaryOperator::Complement,
        parser::UnaryOperator::Negate => UnaryOperator::Negate,
    }
}
