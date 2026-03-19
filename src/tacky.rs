use crate::parser;

// program = Program(function_definition)
#[derive(Debug)]
pub struct Program(pub Function);

// function_definition = Function(identifier, instruction* body)
#[derive(Debug)]
pub struct Function(pub String, pub Vec<Instruction>);

// instruction = Return(val)
// | Unary(unary_operator, val src, val dst)
// | Binary(binary_operator, val src1, val src2, val dst)
#[derive(Debug)]
pub enum Instruction {
    Return(Val),
    Unary(UnaryOperator, Val, Val),
    Binary(BinaryOperator, Val, Val, Val),
}

// val = Constant(int) | Var(identifier)
#[derive(Debug, Clone)]
pub enum Val {
    Constant(i32),
    Var(String),
}

// unary_operator = Complement | Negate
#[derive(Debug, Clone)]
pub enum UnaryOperator {
    Complement,
    Negate,
}

// binary_operator = Add | Subtract | Multiply | Divide | Remainder
#[derive(Debug)]
pub enum BinaryOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
    Remainder,
}

pub fn make_temporary(var_counter: &mut usize) -> String {
    let temp_name = format!("tmp.{}", var_counter);
    *var_counter += 1;
    temp_name
}

pub fn ast_program_to_tacky(ast_program: &parser::Program) -> Program {
    let parser::Program(ast_function) = ast_program;
    let mut var_counter = 0;
    let tacky_function = ast_function_to_tacky(ast_function, &mut var_counter);
    Program(tacky_function)
}

pub fn ast_function_to_tacky(ast_function: &parser::Function, var_counter: &mut usize) -> Function {
    let parser::Function(ast_identifier, ast_statement) = ast_function;

    let identifier = ast_identifier.to_string();
    let instructions = ast_statement_to_tacky(ast_statement, var_counter);

    Function(identifier, instructions)
}

pub fn ast_statement_to_tacky(
    ast_statement: &parser::Statement,
    var_counter: &mut usize,
) -> Vec<Instruction> {
    let mut instructions = Vec::new();

    let parser::Statement::Return(expr) = ast_statement;
    let expr_value = ast_expression_to_tacky(expr, &mut instructions, var_counter);

    instructions.push(Instruction::Return(expr_value));

    instructions
}

pub fn ast_expression_to_tacky(
    ast_expr: &parser::Expr,
    instructions: &mut Vec<Instruction>,
    var_counter: &mut usize,
) -> Val {
    match ast_expr {
        parser::Expr::Constant(i) => Val::Constant(*i),
        parser::Expr::Unary(op, expr) => {
            let src = ast_expression_to_tacky(expr, instructions, var_counter);
            let dst_name = make_temporary(var_counter);
            let dst = Val::Var(dst_name);
            let tacky_op = ast_unop_to_tacky(op);
            instructions.push(Instruction::Unary(tacky_op, src, dst.clone()));
            dst
        }
        parser::Expr::Binary(op, left_expr, right_expr) => {
            let src_1 = ast_expression_to_tacky(left_expr, instructions, var_counter);
            let scr_2 = ast_expression_to_tacky(right_expr, instructions, var_counter);
            let dst_name = make_temporary(var_counter);
            let dst = Val::Var(dst_name);
            let tacky_op = ast_binop_to_tacky(op);
            instructions.push(Instruction::Binary(tacky_op, src_1, scr_2, dst.clone()));
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

pub fn ast_binop_to_tacky(ast_binop: &parser::BinaryOperator) -> BinaryOperator {
    match ast_binop {
        parser::BinaryOperator::Add => BinaryOperator::Add,
        parser::BinaryOperator::Subtract => BinaryOperator::Subtract,
        parser::BinaryOperator::Multiply => BinaryOperator::Multiply,
        parser::BinaryOperator::Divide => BinaryOperator::Divide,
        parser::BinaryOperator::Remainder => BinaryOperator::Remainder,
    }
}
