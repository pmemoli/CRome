use std::collections::HashMap;

use crate::{parser, symbol::SymbolTable};

type VarMap = HashMap<String, String>;

pub fn resolve_program(
    program: &parser::Program,
    symbol_table: &mut SymbolTable,
) -> parser::Program {
    let mut variable_map = VarMap::new();
    let parser::Program(function) = program;
    let resolved_function = resolve_function(function, &mut variable_map, symbol_table);
    parser::Program(resolved_function)
}

pub fn resolve_function(
    function: &parser::Function,
    variable_map: &mut VarMap,
    symbol_table: &mut SymbolTable,
) -> parser::Function {
    let parser::Function(identifier, block_items) = function;

    let resolved_block_items = block_items
        .iter()
        .map(|item| resolve_block_item(item, variable_map, symbol_table))
        .collect();

    parser::Function(identifier.clone(), resolved_block_items)
}

pub fn resolve_block_item(
    block_item: &parser::BlockItem,
    variable_map: &mut VarMap,
    symbol_table: &mut SymbolTable,
) -> parser::BlockItem {
    match block_item {
        parser::BlockItem::D(declaration) => {
            parser::BlockItem::D(resolve_declaration(declaration, variable_map, symbol_table))
        }
        parser::BlockItem::S(statement) => {
            parser::BlockItem::S(resolve_statement(statement, variable_map))
        }
    }
}

pub fn resolve_declaration(
    declaration: &parser::Declaration,
    variable_map: &mut VarMap,
    symbol_table: &mut SymbolTable,
) -> parser::Declaration {
    let parser::Declaration(name, init) = declaration;
    if variable_map.contains_key(name) {
        panic!("Duplicate variable declaration!");
    }

    let unique_name = SymbolTable::generate_variable(symbol_table);
    variable_map.insert(name.clone(), unique_name.clone());

    let resolved_init = init.as_ref().map(|expr| resolve_expr(expr, variable_map));

    parser::Declaration(unique_name, resolved_init)
}

pub fn resolve_statement(
    statement: &parser::Statement,
    variable_map: &mut VarMap,
) -> parser::Statement {
    match statement {
        parser::Statement::Return(expr) => {
            parser::Statement::Return(resolve_expr(expr, variable_map))
        }
        parser::Statement::Expression(expr) => {
            parser::Statement::Expression(resolve_expr(expr, variable_map))
        }
        parser::Statement::If(cond, then_branch, else_branch) => {
            let cond = resolve_expr(cond, variable_map);
            let then_branch = resolve_statement(then_branch.as_ref(), variable_map);
            let else_branch = else_branch
                .as_ref()
                .map(|stmt| Box::new(resolve_statement(stmt, variable_map)));

            parser::Statement::If(cond, Box::new(then_branch), else_branch)
        }
        parser::Statement::Null => parser::Statement::Null,
    }
}

pub fn resolve_expr(expr: &parser::Expr, variable_map: &mut VarMap) -> parser::Expr {
    match expr {
        parser::Expr::Assignment(left, right) => {
            let left = left.as_ref();
            let right = right.as_ref();

            if !matches!(left, parser::Expr::Var(_)) {
                panic!("Invalid lvalue!");
            }

            parser::Expr::Assignment(
                Box::new(resolve_expr(left, variable_map)),
                Box::new(resolve_expr(right, variable_map)),
            )
        }
        parser::Expr::Var(identifier) => {
            if let Some(name) = variable_map.get(identifier) {
                parser::Expr::Var(name.clone())
            } else {
                panic!("Undeclared variable!");
            }
        }
        parser::Expr::Unary(op, e) => {
            let e = e.as_ref();
            parser::Expr::Unary(op.clone(), Box::new(resolve_expr(e, variable_map)))
        }
        parser::Expr::Binary(op, left, right) => {
            let left = left.as_ref();
            let right = right.as_ref();

            parser::Expr::Binary(
                op.clone(),
                Box::new(resolve_expr(left, variable_map)),
                Box::new(resolve_expr(right, variable_map)),
            )
        }
        parser::Expr::Conditional(cond, then_branch, else_branch) => {
            let cond = cond.as_ref();
            let then_branch = then_branch.as_ref();
            let else_branch = else_branch.as_ref();

            parser::Expr::Conditional(
                Box::new(resolve_expr(cond, variable_map)),
                Box::new(resolve_expr(then_branch, variable_map)),
                Box::new(resolve_expr(else_branch, variable_map)),
            )
        }
        other => other.clone(),
    }
}
