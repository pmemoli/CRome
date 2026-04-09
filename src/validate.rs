use std::collections::HashMap;

use crate::{
    parser,
    symbol::{SymbolInfo, SymbolTable, Type},
};

// First pass: Variable resolution
#[derive(Debug, Clone)]
pub struct IdentifierInfo {
    new_name: String,
    declared_in_scope: bool,
    has_linkage: bool,
}

fn generate_variable(i: &mut u32) -> String {
    *i += 1;
    format!("var.{}", i)
}

type IdentifierMap = HashMap<String, IdentifierInfo>;

pub fn new_scope_identifier_map(identifier_map: &IdentifierMap) -> IdentifierMap {
    let mut new_identifier_map = identifier_map.clone();

    for identifier_info in new_identifier_map.values_mut() {
        identifier_info.declared_in_scope = false;
    }

    new_identifier_map
}

pub fn resolve_program(program: &parser::Program) -> parser::Program {
    let mut identifier_map = IdentifierMap::new();
    let mut var_index = 0;

    let parser::Program(declarations) = program;
    let mut new_declarations = Vec::new();
    for declaration in declarations {
        new_declarations.push(resolve_function_declaration(
            declaration,
            &mut identifier_map,
            &mut var_index,
        ))
    }

    parser::Program(new_declarations)
}

pub fn resolve_declaration(
    declaration: &parser::Declaration,
    identifier_map: &mut IdentifierMap,
    var_index: &mut u32,
) -> parser::Declaration {
    match declaration {
        parser::Declaration::VarDecl(var_decl) => parser::Declaration::VarDecl(
            resolve_variable_declaration(var_decl, identifier_map, var_index),
        ),
        parser::Declaration::FunDecl(func_decl) => {
            let parser::FunctionDeclaration(_, _, init) = func_decl;

            // Declarations are always called outside top-level, which can't have
            // function definitions
            if !init.is_none() {
                panic!("Functions can't be defined outside top level")
            }

            parser::Declaration::FunDecl(resolve_function_declaration(
                func_decl,
                identifier_map,
                var_index,
            ))
        }
    }
}

pub fn resolve_block(
    block: &parser::Block,
    identifier_map: &mut IdentifierMap,
    var_index: &mut u32,
) -> parser::Block {
    let parser::Block(block_items) = block;
    let resolved_block_items = block_items
        .iter()
        .map(|item| resolve_block_item(item, identifier_map, var_index))
        .collect();

    parser::Block(resolved_block_items)
}

pub fn resolve_block_item(
    block_item: &parser::BlockItem,
    identifier_map: &mut IdentifierMap,
    var_index: &mut u32,
) -> parser::BlockItem {
    match block_item {
        parser::BlockItem::D(declaration) => {
            parser::BlockItem::D(resolve_declaration(declaration, identifier_map, var_index))
        }
        parser::BlockItem::S(statement) => {
            parser::BlockItem::S(resolve_statement(statement, identifier_map, var_index))
        }
    }
}

pub fn resolve_variable_declaration(
    declaration: &parser::VariableDeclaration,
    identifier_map: &mut IdentifierMap,
    var_index: &mut u32,
) -> parser::VariableDeclaration {
    let parser::VariableDeclaration(name, init) = declaration;

    if let Some(identifier_info) = identifier_map.get(name) {
        if identifier_info.declared_in_scope {
            panic!("Duplicate variable declaration!");
        }
    }

    let new_name = generate_variable(var_index);
    identifier_map.insert(
        name.clone(),
        IdentifierInfo {
            new_name: new_name.clone(),
            declared_in_scope: true,
            has_linkage: false,
        },
    );

    let resolved_init = init.as_ref().map(|expr| resolve_expr(expr, identifier_map));

    parser::VariableDeclaration(new_name, resolved_init)
}

pub fn resolve_function_declaration(
    declaration: &parser::FunctionDeclaration,
    identifier_map: &mut IdentifierMap,
    var_index: &mut u32,
) -> parser::FunctionDeclaration {
    let parser::FunctionDeclaration(name, parameters, body) = declaration;

    // Checks if local variable was declared in the same scope
    if let Some(identifier_info) = identifier_map.get(name) {
        if identifier_info.declared_in_scope && !identifier_info.has_linkage {
            panic!("Duplicate variable declaration!");
        }
    }

    // Maps identifier name
    identifier_map.insert(
        name.clone(),
        IdentifierInfo {
            new_name: name.clone(),
            declared_in_scope: true,
            has_linkage: true,
        },
    );

    // Resolves parameters and body
    let mut inner_map = new_scope_identifier_map(identifier_map);
    let mut new_params = Vec::new();
    for parameter in parameters {
        new_params.push(resolve_parameter(parameter, &mut inner_map, var_index));
    }

    match body {
        Some(block) => {
            let new_body = resolve_block(block, &mut inner_map, var_index);
            parser::FunctionDeclaration(name.clone(), new_params, Some(new_body))
        }
        None => parser::FunctionDeclaration(name.clone(), new_params, None),
    }
}

pub fn resolve_parameter(
    parameter_name: &String,
    identifier_map: &mut IdentifierMap,
    var_index: &mut u32,
) -> String {
    if let Some(identifier_info) = identifier_map.get(parameter_name) {
        if identifier_info.declared_in_scope {
            panic!("Duplicate variable declaration!");
        }
    }

    let new_name = generate_variable(var_index);
    identifier_map.insert(
        parameter_name.clone(),
        IdentifierInfo {
            new_name: new_name.clone(),
            declared_in_scope: true,
            has_linkage: false,
        },
    );

    new_name
}

pub fn resolve_statement(
    statement: &parser::Statement,
    variable_map: &mut IdentifierMap,
    var_index: &mut u32,
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
            let then_branch = resolve_statement(then_branch.as_ref(), variable_map, var_index);
            let else_branch = else_branch
                .as_ref()
                .map(|stmt| Box::new(resolve_statement(stmt, variable_map, var_index)));

            parser::Statement::If(cond, Box::new(then_branch), else_branch)
        }
        parser::Statement::Compound(block) => {
            let mut new_variable_map = new_scope_identifier_map(variable_map);
            let resolved_block = resolve_block(block, &mut new_variable_map, var_index);
            parser::Statement::Compound(resolved_block)
        }
        parser::Statement::While(cond, body, label) => {
            let cond = resolve_expr(cond, variable_map);
            let body = resolve_statement(body.as_ref(), variable_map, var_index);
            parser::Statement::While(cond, Box::new(body), label.clone())
        }
        parser::Statement::DoWhile(body, cond, label) => {
            let body = resolve_statement(body.as_ref(), variable_map, var_index);
            let cond = resolve_expr(cond, variable_map);
            parser::Statement::DoWhile(Box::new(body), cond, label.clone())
        }
        parser::Statement::For(for_init, opt_cond, opt_post, body, label) => {
            let mut new_variable_map = new_scope_identifier_map(variable_map);

            let for_init = resolve_for_init(for_init, &mut new_variable_map, var_index);
            let opt_cond = resolve_optional_expr(opt_cond, &mut new_variable_map);
            let opt_post = resolve_optional_expr(opt_post, &mut new_variable_map);
            let body = resolve_statement(body.as_ref(), &mut new_variable_map, var_index);

            parser::Statement::For(for_init, opt_cond, opt_post, Box::new(body), label.clone())
        }
        stmt => stmt.clone(),
    }
}

pub fn resolve_for_init(
    for_init: &parser::ForInit,
    variable_map: &mut IdentifierMap,
    var_index: &mut u32,
) -> parser::ForInit {
    match for_init {
        parser::ForInit::InitDecl(decl) => {
            parser::ForInit::InitDecl(resolve_variable_declaration(decl, variable_map, var_index))
        }
        parser::ForInit::InitExp(opt_expr) => {
            parser::ForInit::InitExp(resolve_optional_expr(opt_expr, variable_map))
        }
    }
}

pub fn resolve_optional_expr(
    opt_expr: &Option<parser::Expr>,
    variable_map: &mut IdentifierMap,
) -> Option<parser::Expr> {
    opt_expr
        .as_ref()
        .map(|expr| resolve_expr(expr, variable_map))
}

pub fn resolve_expr(expr: &parser::Expr, identifier_map: &mut IdentifierMap) -> parser::Expr {
    match expr {
        parser::Expr::Assignment(left, right) => {
            let left = left.as_ref();
            let right = right.as_ref();

            if !matches!(left, parser::Expr::Var(_)) {
                panic!("Invalid lvalue!");
            }

            parser::Expr::Assignment(
                Box::new(resolve_expr(left, identifier_map)),
                Box::new(resolve_expr(right, identifier_map)),
            )
        }
        parser::Expr::Var(identifier) => {
            if let Some(identifier_info) = identifier_map.get(identifier) {
                parser::Expr::Var(identifier_info.new_name.clone())
            } else {
                panic!("Undeclared variable!");
            }
        }
        parser::Expr::Unary(op, e) => {
            let e = e.as_ref();
            parser::Expr::Unary(op.clone(), Box::new(resolve_expr(e, identifier_map)))
        }
        parser::Expr::Binary(op, left, right) => {
            let left = left.as_ref();
            let right = right.as_ref();

            parser::Expr::Binary(
                op.clone(),
                Box::new(resolve_expr(left, identifier_map)),
                Box::new(resolve_expr(right, identifier_map)),
            )
        }
        parser::Expr::Conditional(cond, then_branch, else_branch) => {
            let cond = cond.as_ref();
            let then_branch = then_branch.as_ref();
            let else_branch = else_branch.as_ref();

            parser::Expr::Conditional(
                Box::new(resolve_expr(cond, identifier_map)),
                Box::new(resolve_expr(then_branch, identifier_map)),
                Box::new(resolve_expr(else_branch, identifier_map)),
            )
        }
        parser::Expr::FunctionCall(identifier, args) => {
            if let Some(identifier_info) = identifier_map.get(identifier) {
                let new_func_name = identifier_info.new_name.clone();
                let mut new_args = Vec::new();
                for arg in args {
                    new_args.push(resolve_expr(arg, identifier_map));
                }
                parser::Expr::FunctionCall(new_func_name, new_args)
            } else {
                panic!("Undeclared function");
            }
        }
        other => other.clone(),
    }
}

// Second pass: Loop labeling
pub fn label_program(program: &parser::Program) -> parser::Program {
    let mut loop_idx = 0;
    let current_label = None;

    let parser::Program(declarations) = program;
    let mut new_declarations = Vec::new();
    for declaration in declarations {
        new_declarations.push(label_function_declaration(
            declaration,
            &mut loop_idx,
            &current_label,
        ));
    }

    parser::Program(new_declarations)
}

pub fn label_function_declaration(
    function_declaration: &parser::FunctionDeclaration,
    loop_idx: &mut usize,
    current_label: &Option<String>,
) -> parser::FunctionDeclaration {
    let parser::FunctionDeclaration(identifier, parameters, body) = function_declaration;
    if let Some(block) = body.as_ref() {
        let new_block = label_block(block, loop_idx, current_label);
        parser::FunctionDeclaration(identifier.clone(), parameters.clone(), Some(new_block))
    } else {
        function_declaration.clone()
    }
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

// Third pass: Type checking
pub fn typecheck_variable_declaration(
    variable_declaration: &parser::VariableDeclaration,
    symbol_table: &mut SymbolTable,
) {
    let parser::VariableDeclaration(name, init) = variable_declaration;
    symbol_table.generate_variable(name);
    if let Some(e) = init.as_ref() {
        typecheck_expr(e, symbol_table);
    }
}

pub fn typecheck_function_declaration(
    function_declaration: &parser::FunctionDeclaration,
    symbol_table: &mut SymbolTable,
) {
    let parser::FunctionDeclaration(name, parameters, body) = function_declaration;
    let og_type = Type::FunType(parameters.len());
    let mut already_defined = false;

    // Check that function declarations are consistent everywhere.
    // Check that the function is not defined more than once.
    if let Some(symbol_info) = symbol_table.map.get(name) {
        if let SymbolInfo::Function { ty, defined } = symbol_info {
            if ty != &og_type {
                panic!("Incompatible declarations")
            }

            already_defined = *defined;
            if *defined && body.is_some() {
                panic!("Function defined twice")
            }
        } else {
            panic!("Incompatible declarations")
        }
    }

    symbol_table.generate_function(name, parameters.len(), body.is_some() || already_defined);

    if let Some(block) = body.as_ref() {
        // Only generate names for defined functions
        for param_name in parameters {
            symbol_table.generate_variable(param_name);
        }
        typecheck_block(block, symbol_table);
    }
}

// typecheck_exp(e, symbols):
// match e with
// | FunctionCall(f, args) ->
// f_type = symbols.get(f).type
// 1 if f_type == Int:
// fail("Variable used as function name")
// 2 if f_type.param_count != length(args):
// fail("Function called with the wrong number of arguments")
// 3 for arg in args:
// typecheck_exp(arg, symbols)
// | Var(v) ->
// 4 if symbols.get(v).type != Int:
// fail("Function name used as variable")
// | --snip--

pub fn typecheck_exp(expr: parser::Expr, symbol_table: &mut SymbolTable) {}

// Wrapper for semantic analysis passes
pub fn semantic_analysis(ast: &parser::Program, symbol_table: &mut SymbolTable) -> parser::Program {
    let identifier_resolution_ast = resolve_program(ast);
    let loop_labeling_ast = label_program(&identifier_resolution_ast);
    loop_labeling_ast
}
