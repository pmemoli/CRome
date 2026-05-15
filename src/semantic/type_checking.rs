use super::*;

use crate::{symbol::SymbolMetadata, symbol::Type};

// Third pass (Type Checking):

// 1. Check that function declaration types are consistent everywhere, and their linkage is too
// 2. A function can't be called with the wrong number of arguments
// 3. A function can't be defined more than once (not really type checking but easy to implement here)

// It also fills the symbol table with variable and function information

pub fn typecheck_program(program: &parser::Program, symbol_table: &mut SymbolTable) {
    let parser::Program(declarations) = program;
    let mut new_declarations = Vec::new();
    for declaration in declarations {
        new_declarations.push(typecheck_declaration(declaration, symbol_table))
    }
}

pub fn typecheck_declaration(declaration: &parser::Declaration, symbol_table: &mut SymbolTable) {
    match declaration {
        parser::Declaration::VarDecl(var_decl) => {
            typecheck_variable_declaration(var_decl, symbol_table);
        }
        parser::Declaration::FunDecl(func_decl) => {
            typecheck_function_declaration(func_decl, symbol_table);
        }
    }
}

pub fn typecheck_variable_declaration(
    variable_declaration: &parser::VariableDeclaration,
    symbol_table: &mut SymbolTable,
) {
    let parser::VariableDeclaration(_, init, _) = variable_declaration;
    if let Some(e) = init.as_ref() {
        typecheck_expr(e, symbol_table);
    }
}

pub fn typecheck_function_declaration(
    function_declaration: &parser::FunctionDeclaration,
    symbol_table: &mut SymbolTable,
) {
    let parser::FunctionDeclaration(name, parameters, body, storage_class) = function_declaration;
    let mut already_defined = false;
    let mut global = !matches!(storage_class, Some(parser::StorageClass::Static));

    // Check that function declarations are consistent everywhere.
    // Check that the function is not defined more than once.
    if let Some(symbol_info) = symbol_table.map.get(name) {
        if symbol_info.ty != Type::FunType(parameters.len()) {
            panic!("Incompatible declarations")
        }

        match symbol_info.metadata {
            SymbolMetadata::Function { defined } => {
                already_defined = defined;
                if defined && body.is_some() {
                    panic!("Function defined twice");
                }
            }
            _ => panic!("Incompatible declarations"),
        }
    }

    symbol_table.insert_function(name, parameters.len(), body.is_some() || already_defined);

    if let Some(block) = body.as_ref() {
        typecheck_block(block, symbol_table);
    }
}

pub fn typecheck_block(block: &parser::Block, symbol_table: &mut SymbolTable) {
    let parser::Block(block_items) = block;
    for block_item in block_items {
        match block_item {
            parser::BlockItem::D(declaration) => typecheck_declaration(declaration, symbol_table),
            parser::BlockItem::S(statement) => typecheck_statement(statement, symbol_table),
        }
    }
}

pub fn typecheck_statement(statement: &parser::Statement, symbol_table: &mut SymbolTable) {
    match statement {
        parser::Statement::Return(expr) => typecheck_expr(expr, symbol_table),
        parser::Statement::Expression(expr) => typecheck_expr(expr, symbol_table),
        parser::Statement::If(cond, then_branch, else_branch) => {
            typecheck_expr(cond, symbol_table);
            typecheck_statement(then_branch.as_ref(), symbol_table);
            if let Some(else_stmt) = else_branch.as_ref() {
                typecheck_statement(else_stmt.as_ref(), symbol_table);
            }
        }
        parser::Statement::Compound(block) => typecheck_block(block, symbol_table),
        parser::Statement::While(cond, body, _) => {
            typecheck_expr(cond, symbol_table);
            typecheck_statement(body.as_ref(), symbol_table);
        }
        parser::Statement::DoWhile(body, cond, _) => {
            typecheck_statement(body.as_ref(), symbol_table);
            typecheck_expr(cond, symbol_table);
        }
        parser::Statement::For(init_1, init_2, init_3, body, _) => {
            match init_1 {
                parser::ForInit::InitDecl(decl) => {
                    typecheck_variable_declaration(decl, symbol_table)
                }
                parser::ForInit::InitExp(opt_expr) => {
                    if let Some(expr) = opt_expr.as_ref() {
                        typecheck_expr(expr, symbol_table);
                    }
                }
            }

            if let Some(expr) = init_2.as_ref() {
                typecheck_expr(expr, symbol_table);
            }

            if let Some(expr) = init_3.as_ref() {
                typecheck_expr(expr, symbol_table);
            }

            typecheck_statement(body.as_ref(), symbol_table);
        }
        _ => {}
    }
}

pub fn typecheck_expr(expr: &parser::Expr, symbol_table: &mut SymbolTable) {
    // Checks that types are used correctly
    match expr {
        parser::Expr::FunctionCall(name, arguments) => {
            if let Some(info) = symbol_table.map.get(name) {
                let f_type = &info.ty;
                match f_type {
                    Type::Int => panic!("Variable used as function name"),
                    Type::FunType(n) => {
                        if n != &arguments.len() {
                            panic!("Function called with wrong amount of parameters")
                        }
                    }
                }

                for arg in arguments {
                    typecheck_expr(arg, symbol_table);
                }
            } else {
                panic!("Undeclared function");
            }
        }

        parser::Expr::Var(name) => {
            if let Some(info) = symbol_table.map.get(name) {
                let f_type = &info.ty;
                match f_type {
                    Type::FunType(_) => panic!("Function name used as variable"),
                    _ => {}
                }
            } else {
                panic!("Undeclared variable");
            }
        }

        parser::Expr::Assignment(left, right) => {
            typecheck_expr(left.as_ref(), symbol_table);
            typecheck_expr(right.as_ref(), symbol_table);
        }

        parser::Expr::Unary(_, e) => typecheck_expr(e.as_ref(), symbol_table),

        parser::Expr::Binary(_, left, right) => {
            typecheck_expr(left.as_ref(), symbol_table);
            typecheck_expr(right.as_ref(), symbol_table);
        }

        parser::Expr::Conditional(cond, then_branch, else_branch) => {
            typecheck_expr(cond.as_ref(), symbol_table);
            typecheck_expr(then_branch.as_ref(), symbol_table);
            typecheck_expr(else_branch.as_ref(), symbol_table);
        }

        _ => {}
    }
}
