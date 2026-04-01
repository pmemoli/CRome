use std::collections::HashMap;

use crate::{parser, symbol::SymbolTable};

// First pass: Variable resolution
#[derive(Debug, Clone)]
pub struct VarInfo {
    unique_name: String,
    declared_in_scope: bool,
}

type VarMap = HashMap<String, VarInfo>;

pub fn new_scope_var_map(variable_map: &VarMap) -> VarMap {
    let mut new_variable_map = variable_map.clone();

    for var_info in new_variable_map.values_mut() {
        var_info.declared_in_scope = false;
    }

    new_variable_map
}

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
    let parser::Function(identifier, block) = function;
    let resolved_block = resolve_block(block, variable_map, symbol_table);
    parser::Function(identifier.clone(), resolved_block)
}

pub fn resolve_block(
    block: &parser::Block,
    variable_map: &mut VarMap,
    symbol_table: &mut SymbolTable,
) -> parser::Block {
    let parser::Block(block_items) = block;
    let resolved_block_items = block_items
        .iter()
        .map(|item| resolve_block_item(item, variable_map, symbol_table))
        .collect();

    parser::Block(resolved_block_items)
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
            parser::BlockItem::S(resolve_statement(statement, variable_map, symbol_table))
        }
    }
}

pub fn resolve_declaration(
    declaration: &parser::Declaration,
    variable_map: &mut VarMap,
    symbol_table: &mut SymbolTable,
) -> parser::Declaration {
    let parser::Declaration(name, init) = declaration;

    if matches!(variable_map.get(name), Some(v) if v.declared_in_scope) {
        panic!("Duplicate variable declaration!");
    }

    let unique_name = SymbolTable::generate_variable(symbol_table);
    variable_map.insert(
        name.clone(),
        VarInfo {
            unique_name: unique_name.clone(),
            declared_in_scope: true,
        },
    );

    let resolved_init = init.as_ref().map(|expr| resolve_expr(expr, variable_map));

    parser::Declaration(unique_name, resolved_init)
}

pub fn resolve_statement(
    statement: &parser::Statement,
    variable_map: &mut VarMap,
    symbol_table: &mut SymbolTable,
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
            let then_branch = resolve_statement(then_branch.as_ref(), variable_map, symbol_table);
            let else_branch = else_branch
                .as_ref()
                .map(|stmt| Box::new(resolve_statement(stmt, variable_map, symbol_table)));

            parser::Statement::If(cond, Box::new(then_branch), else_branch)
        }
        parser::Statement::Compound(block) => {
            let mut new_variable_map = new_scope_var_map(variable_map);
            let resolved_block = resolve_block(block, &mut new_variable_map, symbol_table);
            parser::Statement::Compound(resolved_block)
        }
        parser::Statement::While(cond, body, label) => {
            let cond = resolve_expr(cond, variable_map);
            let body = resolve_statement(body.as_ref(), variable_map, symbol_table);
            parser::Statement::While(cond, Box::new(body), label.clone())
        }
        parser::Statement::DoWhile(body, cond, label) => {
            let body = resolve_statement(body.as_ref(), variable_map, symbol_table);
            let cond = resolve_expr(cond, variable_map);
            parser::Statement::DoWhile(Box::new(body), cond, label.clone())
        }
        parser::Statement::For(for_init, opt_cond, opt_post, body, label) => {
            let mut new_variable_map = new_scope_var_map(variable_map);

            let for_init = resolve_for_init(for_init, &mut new_variable_map, symbol_table);
            let opt_cond = resolve_optional_expr(opt_cond, &mut new_variable_map);
            let opt_post = resolve_optional_expr(opt_post, &mut new_variable_map);
            let body = resolve_statement(body.as_ref(), &mut new_variable_map, symbol_table);

            parser::Statement::For(for_init, opt_cond, opt_post, Box::new(body), label.clone())
        }
        stmt => stmt.clone(),
    }
}

pub fn resolve_for_init(
    for_init: &parser::ForInit,
    variable_map: &mut VarMap,
    symbol_table: &mut SymbolTable,
) -> parser::ForInit {
    match for_init {
        parser::ForInit::InitDecl(decl) => {
            parser::ForInit::InitDecl(resolve_declaration(decl, variable_map, symbol_table))
        }
        parser::ForInit::InitExp(opt_expr) => {
            parser::ForInit::InitExp(resolve_optional_expr(opt_expr, variable_map))
        }
    }
}

pub fn resolve_optional_expr(
    opt_expr: &Option<parser::Expr>,
    variable_map: &mut VarMap,
) -> Option<parser::Expr> {
    opt_expr
        .as_ref()
        .map(|expr| resolve_expr(expr, variable_map))
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
            if let Some(var_info) = variable_map.get(identifier) {
                parser::Expr::Var(var_info.unique_name.clone())
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

// Second pass: Loop labeling
pub fn label_program(program: &parser::Program) -> parser::Program {
    let mut loop_idx = 0;
    let current_label = None;
    let parser::Program(function) = program;
    let resolved_function = label_function(function, &mut loop_idx, &current_label);
    parser::Program(resolved_function)
}

pub fn label_function(
    function: &parser::Function,
    loop_idx: &mut usize,
    current_label: &Option<String>,
) -> parser::Function {
    let parser::Function(identifier, block) = function;
    let labeled_block = label_block(block, loop_idx, current_label);
    parser::Function(identifier.clone(), labeled_block)
}

pub fn label_block(
    block: &parser::Block,
    loop_idx: &mut usize,
    current_label: &Option<String>,
) -> parser::Block {
    let parser::Block(block_items) = block;
    let labeled_block_items = block_items
        .iter()
        .map(|item| label_block_item(item, loop_idx, current_label))
        .collect();

    parser::Block(labeled_block_items)
}

pub fn label_block_item(
    block_item: &parser::BlockItem,
    loop_idx: &mut usize,
    current_label: &Option<String>,
) -> parser::BlockItem {
    match block_item {
        parser::BlockItem::D(declaration) => parser::BlockItem::D(declaration.clone()),
        parser::BlockItem::S(statement) => {
            parser::BlockItem::S(label_statement(statement, loop_idx, current_label))
        }
    }
}

pub fn label_statement(
    statement: &parser::Statement,
    loop_idx: &mut usize,
    current_label: &Option<String>,
) -> parser::Statement {
    match statement {
        parser::Statement::Break(_) => {
            if matches!(current_label, None) {
                panic!("Break statement outside of body");
            }

            parser::Statement::Break(current_label.clone())
        }
        parser::Statement::Continue(_) => {
            if matches!(current_label, None) {
                panic!("Continue statement outside of body");
            }

            parser::Statement::Continue(current_label.clone())
        }
        parser::Statement::While(cond_expr, body_stmt, _) => {
            *loop_idx += 1;
            let new_label = Some(format!("loop.{}", loop_idx));
            let labeled_body = label_statement(body_stmt.as_ref(), loop_idx, &new_label);

            parser::Statement::While(cond_expr.clone(), Box::new(labeled_body), new_label)
        }
        parser::Statement::DoWhile(body_stmt, cond_expr, _) => {
            *loop_idx += 1;
            let new_label = Some(format!("loop.{}", loop_idx));
            let labeled_body = label_statement(body_stmt.as_ref(), loop_idx, &new_label);

            parser::Statement::DoWhile(Box::new(labeled_body), cond_expr.clone(), new_label)
        }
        parser::Statement::For(init_1, init_2, init_3, body_stmt, _) => {
            *loop_idx += 1;
            let new_label = Some(format!("loop.{}", loop_idx));
            let labeled_body = label_statement(body_stmt.as_ref(), loop_idx, &new_label);

            parser::Statement::For(
                init_1.clone(),
                init_2.clone(),
                init_3.clone(),
                Box::new(labeled_body),
                new_label,
            )
        }
        parser::Statement::Compound(block) => {
            parser::Statement::Compound(label_block(block, loop_idx, current_label))
        }
        parser::Statement::If(cond_expr, then_stmt, else_stmt) => {
            let labeled_then = label_statement(then_stmt.as_ref(), loop_idx, current_label);
            let labeled_else = else_stmt
                .as_ref()
                .map(|s| Box::new(label_statement(s.as_ref(), loop_idx, current_label)));

            parser::Statement::If(cond_expr.clone(), Box::new(labeled_then), labeled_else)
        }
        stmt => stmt.clone(),
    }
}

// Wrapper for semantic analysis passes
pub fn semantic_analysis(ast: &parser::Program, symbol_table: &mut SymbolTable) -> parser::Program {
    let variable_resolution_ast = resolve_program(ast, symbol_table);
    let loop_labeling_ast = label_program(&variable_resolution_ast);
    loop_labeling_ast
}
