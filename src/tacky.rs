use crate::symbol::SymbolTable;
use std::collections::HashMap;

use crate::parser;

// program = Program(function_definition)
#[derive(Debug)]
pub struct Program(pub Function);

// function_definition = Function(identifier, instruction* body)
#[derive(Debug)]
pub struct Function(pub String, pub Vec<Instruction>);

// instruction = Return(val)
//     | Unary(unary_operator, val src, val dst)
//     | Binary(binary_operator, val src1, val src2, val dst)
//     | Copy(val src, val dst)
//     | Jump(identifier target)
//     | JumpIfZero(val condition, identifier target)
//     | JumpIfNotZero(val condition, identifier target)
//     | Label(identifier)
#[derive(Debug)]
pub enum Instruction {
    Return(Val),
    Unary(UnaryOperator, Val, Val),
    Binary(BinaryOperator, Val, Val, Val),
    Copy(Val, Val),
    Jump(String),
    JumpIfZero(Val, String),
    JumpIfNotZero(Val, String),
    Label(String),
}

// val = Constant(int) | Var(identifier)
#[derive(Debug, Clone)]
pub enum Val {
    Constant(i32),
    Var(String),
}

// unary_operator = Complement | Negate | Not
#[derive(Debug, Clone)]
pub enum UnaryOperator {
    Complement,
    Negate,
    Not,
}

// binary_operator = Add | Subtract | Multiply | Divide | Remainder | Equal | NotEqual
//     | LessThan | LessOrEqual | GreaterThan | GreaterOrEqual
#[derive(Debug)]
pub enum BinaryOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
    Remainder,
    Equal,
    NotEqual,
    LessThan,
    LessOrEqual,
    GreaterThan,
    GreaterOrEqual,
}

pub fn ast_program_to_tacky(
    ast_program: &parser::Program,
    symbol_table: &mut SymbolTable,
) -> Program {
    let parser::Program(ast_function) = ast_program;
    let tacky_function = ast_function_to_tacky(ast_function, symbol_table);

    Program(tacky_function)
}

pub fn ast_function_to_tacky(
    ast_function: &parser::Function,
    symbol_table: &mut SymbolTable,
) -> Function {
    let parser::Function(ast_identifier, ast_statement) = ast_function;

    let identifier = ast_identifier.to_string();
    let instructions = ast_statement_to_tacky(ast_statement, symbol_table);

    Function(identifier, instructions)
}

pub fn ast_statement_to_tacky(
    ast_statement: &parser::Statement,
    symbol_table: &mut SymbolTable,
) -> Vec<Instruction> {
    let mut instructions = Vec::new();

    let parser::Statement::Return(expr) = ast_statement;
    let expr_value = ast_expression_to_tacky(expr, &mut instructions, symbol_table);

    instructions.push(Instruction::Return(expr_value));

    instructions
}

pub fn ast_expression_to_tacky(
    ast_expr: &parser::Expr,
    instructions: &mut Vec<Instruction>,
    symbol_table: &mut SymbolTable,
) -> Val {
    match ast_expr {
        parser::Expr::Constant(i) => Val::Constant(*i),
        parser::Expr::Unary(op, expr) => {
            let src = ast_expression_to_tacky(expr, instructions, symbol_table);
            let dst_name = SymbolTable::generate_variable(symbol_table);
            let dst = Val::Var(dst_name);
            let tacky_op = ast_unop_to_tacky(op);
            instructions.push(Instruction::Unary(tacky_op, src, dst.clone()));
            dst
        }
        parser::Expr::Binary(op, left_expr, right_expr) => match op {
            // Short circuit binary operators
            parser::BinaryOperator::And | parser::BinaryOperator::Or => {
                let label_idx = SymbolTable::generate_label_idx(symbol_table);

                let val_1 = ast_expression_to_tacky(left_expr, instructions, symbol_table);

                if let parser::BinaryOperator::And = op {
                    instructions.push(Instruction::JumpIfZero(
                        val_1,
                        format!("shortcircuit.{}", label_idx),
                    ));
                } else {
                    instructions.push(Instruction::JumpIfNotZero(
                        val_1,
                        format!("shortcircuit.{}", label_idx),
                    ));
                }

                let val_2 = ast_expression_to_tacky(right_expr, instructions, symbol_table);

                if let parser::BinaryOperator::And = op {
                    instructions.push(Instruction::JumpIfZero(
                        val_2,
                        format!("shortcircuit.{}", label_idx),
                    ))
                } else {
                    instructions.push(Instruction::JumpIfNotZero(
                        val_2,
                        format!("shortcircuit.{}", label_idx),
                    ));
                }

                let dst_name = SymbolTable::generate_variable(symbol_table);
                let dst = Val::Var(dst_name);

                instructions.push(Instruction::Copy(Val::Constant(1), dst.clone()));
                instructions.push(Instruction::Jump(format!("end.{}", label_idx)));

                instructions.push(Instruction::Label(format!("shortcircuit.{}", label_idx)));
                instructions.push(Instruction::Copy(Val::Constant(0), dst.clone()));

                instructions.push(Instruction::Label(format!("end.{}", label_idx)));

                dst
            }
            _ => {
                let val_1 = ast_expression_to_tacky(left_expr, instructions, symbol_table);
                let val_2 = ast_expression_to_tacky(right_expr, instructions, symbol_table);
                let dst_name = SymbolTable::generate_variable(symbol_table);
                let dst = Val::Var(dst_name);
                let tacky_op = ast_binop_to_tacky(op);
                instructions.push(Instruction::Binary(tacky_op, val_1, val_2, dst.clone()));
                dst
            }
        },
    }
}

pub fn ast_unop_to_tacky(ast_unop: &parser::UnaryOperator) -> UnaryOperator {
    match ast_unop {
        parser::UnaryOperator::Complement => UnaryOperator::Complement,
        parser::UnaryOperator::Negate => UnaryOperator::Negate,
        parser::UnaryOperator::Not => UnaryOperator::Not,
    }
}

pub fn ast_binop_to_tacky(ast_binop: &parser::BinaryOperator) -> BinaryOperator {
    match ast_binop {
        parser::BinaryOperator::Add => BinaryOperator::Add,
        parser::BinaryOperator::Subtract => BinaryOperator::Subtract,
        parser::BinaryOperator::Multiply => BinaryOperator::Multiply,
        parser::BinaryOperator::Divide => BinaryOperator::Divide,
        parser::BinaryOperator::Remainder => BinaryOperator::Remainder,
        parser::BinaryOperator::Equal => BinaryOperator::Equal,
        parser::BinaryOperator::NotEqual => BinaryOperator::NotEqual,
        parser::BinaryOperator::LessThan => BinaryOperator::LessThan,
        parser::BinaryOperator::LessThanOrEqual => BinaryOperator::LessOrEqual,
        parser::BinaryOperator::GreaterThan => BinaryOperator::GreaterThan,
        parser::BinaryOperator::GreaterThanOrEqual => BinaryOperator::GreaterOrEqual,
        _ => panic!("Unsupported binary operator: {:?}", ast_binop), // Missing AND and OR
    }
}
