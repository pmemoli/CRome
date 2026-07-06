use std::collections::HashMap;

use super::*;

// First pass: Identifier resolution

// Variables:

// 1. Rename each non-linked variable name to a unique one.
// 2. Check that all variables in expressions are declared

// Functions:

// 1. Check that all function calls refer to declared identifiers
// 2. Check that definitions of functions do not live within blocks

// AND: Check that identifier declarations do not contradict in having or not having linkage

// We use a declared identifier map for this
#[derive(Debug, Clone)]
pub struct IdentifierInfo {
    new_name: String,
    declared_in_current_scope: bool,
    has_linkage: bool,
}

type IdentifierMap = HashMap<String, IdentifierInfo>;

fn new_unique_variable_name(counter: &mut usize) -> String {
    *counter += 1;
    format!("var.{}", counter)
}

pub fn new_scope_identifier_map(identifier_map: &IdentifierMap) -> IdentifierMap {
    let mut new_identifier_map = identifier_map.clone();

    for identifier_info in new_identifier_map.values_mut() {
        identifier_info.declared_in_current_scope = false;
    }

    new_identifier_map
}

pub fn resolve_program(program: &parser::Program) -> parser::Program {
    let mut identifier_map = IdentifierMap::new();
    let mut counter = 0;

    let parser::Program(declarations) = program;
    let mut new_declarations = Vec::new();
    for declaration in declarations {
        new_declarations.push(resolve_file_scope_declaration(
            declaration,
            &mut identifier_map,
            &mut counter,
        ))
    }

    parser::Program(new_declarations)
}

pub fn resolve_file_scope_declaration(
    declaration: &parser::Declaration,
    identifier_map: &mut IdentifierMap,
    counter: &mut usize,
) -> parser::Declaration {
    match declaration {
        parser::Declaration::VarDecl(var_decl) => parser::Declaration::VarDecl(
            resolve_file_scope_variable_declaration(var_decl, identifier_map),
        ),
        parser::Declaration::FunDecl(func_decl) => parser::Declaration::FunDecl(
            resolve_function_declaration(func_decl, identifier_map, counter),
        ),
    }
}

pub fn resolve_file_scope_variable_declaration(
    declaration: &parser::VariableDeclaration,
    identifier_map: &mut IdentifierMap,
) -> parser::VariableDeclaration {
    let parser::VariableDeclaration(name, _, _, _) = declaration;

    identifier_map.insert(
        name.clone(),
        IdentifierInfo {
            new_name: name.clone(), // Storage duration is always static
            declared_in_current_scope: true,
            has_linkage: true, // Always has external or internal linkage
        },
    );

    declaration.clone()
}

pub fn resolve_block_scope_declaration(
    declaration: &parser::Declaration,
    identifier_map: &mut IdentifierMap,
    counter: &mut usize,
) -> parser::Declaration {
    match declaration {
        parser::Declaration::VarDecl(var_decl) => parser::Declaration::VarDecl(
            resolve_block_scope_variable_declaration(var_decl, identifier_map, counter),
        ),
        parser::Declaration::FunDecl(func_decl) => {
            let parser::FunctionDeclaration(_, _, init, _, storage_class) = func_decl;

            if storage_class == &Some(parser::StorageClass::Static) {
                panic!("Functions can't have static storage class within ");
            }

            if !init.is_none() {
                panic!("Functions can't be defined outside top level")
            }

            parser::Declaration::FunDecl(resolve_function_declaration(
                func_decl,
                identifier_map,
                counter,
            ))
        }
    }
}

pub fn resolve_block_scope_variable_declaration(
    declaration: &parser::VariableDeclaration,
    identifier_map: &mut IdentifierMap,
    counter: &mut usize,
) -> parser::VariableDeclaration {
    let parser::VariableDeclaration(name, init, ty, storage_class) = declaration;

    if let Some(identifier_info) = identifier_map.get(name) {
        // Checks for consistent linkage (variables only have linkage if extern)
        if identifier_info.declared_in_current_scope {
            if !(identifier_info.has_linkage
                && storage_class == &Some(parser::StorageClass::Extern))
            {
                panic!("Duplicate variable declaration!");
            }
        }
    }

    // External specifiers imply linkage (no unique name)
    if storage_class == &Some(parser::StorageClass::Extern) {
        if let Some(_) = init.as_ref() {
            panic!("External variable declaration can't have initializer");
        }

        identifier_map.insert(
            name.clone(),
            IdentifierInfo {
                new_name: name.clone(),
                declared_in_current_scope: true,
                has_linkage: true,
            },
        );

        declaration.clone()
    } else {
        let new_name = new_unique_variable_name(counter);
        identifier_map.insert(
            name.clone(),
            IdentifierInfo {
                new_name: new_name.clone(),
                declared_in_current_scope: true,
                has_linkage: false,
            },
        );

        let resolved_init = init.as_ref().map(|expr| resolve_expr(expr, identifier_map));

        parser::VariableDeclaration(new_name, resolved_init, ty.clone(), storage_class.clone())
    }
}

pub fn resolve_function_declaration(
    declaration: &parser::FunctionDeclaration,
    identifier_map: &mut IdentifierMap,
    counter: &mut usize,
) -> parser::FunctionDeclaration {
    let parser::FunctionDeclaration(name, parameters, body, ty, storage_class) = declaration;

    // Checks for consistent linkage (functions always have linkage)
    if let Some(identifier_info) = identifier_map.get(name) {
        if !identifier_info.has_linkage && identifier_info.declared_in_current_scope {
            panic!("Function name conflicts with local variable declared in the same scope");
        }
    }

    // Maps identifier name
    identifier_map.insert(
        name.clone(),
        IdentifierInfo {
            new_name: name.clone(),
            declared_in_current_scope: true,
            has_linkage: true,
        },
    );

    // Resolves parameters and body
    let mut inner_map = new_scope_identifier_map(identifier_map);
    let mut new_params = Vec::new();
    for parameter in parameters {
        new_params.push(resolve_parameter(parameter, &mut inner_map, counter));
    }

    match body {
        Some(block) => {
            let new_body = resolve_block(block, &mut inner_map, counter);
            parser::FunctionDeclaration(
                name.clone(),
                new_params,
                Some(new_body),
                ty.clone(),
                storage_class.clone(),
            )
        }
        None => parser::FunctionDeclaration(
            name.clone(),
            new_params,
            None,
            ty.clone(),
            storage_class.clone(),
        ),
    }
}

pub fn resolve_block(
    block: &parser::Block,
    identifier_map: &mut IdentifierMap,
    counter: &mut usize,
) -> parser::Block {
    let parser::Block(block_items) = block;
    let mut resolved_block_items = Vec::new();
    for item in block_items {
        resolved_block_items.push(resolve_block_item(item, identifier_map, counter));
    }

    parser::Block(resolved_block_items)
}

pub fn resolve_block_item(
    block_item: &parser::BlockItem,
    identifier_map: &mut IdentifierMap,
    counter: &mut usize,
) -> parser::BlockItem {
    match block_item {
        parser::BlockItem::D(declaration) => parser::BlockItem::D(resolve_block_scope_declaration(
            declaration,
            identifier_map,
            counter,
        )),
        parser::BlockItem::S(statement) => {
            parser::BlockItem::S(resolve_statement(statement, identifier_map, counter))
        }
    }
}

pub fn resolve_parameter(
    parameter_name: &String,
    identifier_map: &mut IdentifierMap,
    counter: &mut usize,
) -> String {
    if let Some(identifier_info) = identifier_map.get(parameter_name) {
        if identifier_info.declared_in_current_scope {
            panic!("Duplicate variable declaration!");
        }
    }

    let new_name = new_unique_variable_name(counter);
    identifier_map.insert(
        parameter_name.clone(),
        IdentifierInfo {
            new_name: new_name.clone(),
            declared_in_current_scope: true,
            has_linkage: false,
        },
    );

    new_name
}

pub fn resolve_statement(
    statement: &parser::Statement,
    variable_map: &mut IdentifierMap,
    counter: &mut usize,
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
            let then_branch = resolve_statement(then_branch.as_ref(), variable_map, counter);
            let else_branch = else_branch
                .as_ref()
                .map(|stmt| Box::new(resolve_statement(stmt, variable_map, counter)));

            parser::Statement::If(cond, Box::new(then_branch), else_branch)
        }
        parser::Statement::Compound(block) => {
            let mut new_variable_map = new_scope_identifier_map(variable_map);
            let resolved_block = resolve_block(block, &mut new_variable_map, counter);
            parser::Statement::Compound(resolved_block)
        }
        parser::Statement::While(cond, body, label) => {
            let cond = resolve_expr(cond, variable_map);
            let body = resolve_statement(body.as_ref(), variable_map, counter);
            parser::Statement::While(cond, Box::new(body), label.clone())
        }
        parser::Statement::DoWhile(body, cond, label) => {
            let body = resolve_statement(body.as_ref(), variable_map, counter);
            let cond = resolve_expr(cond, variable_map);
            parser::Statement::DoWhile(Box::new(body), cond, label.clone())
        }
        parser::Statement::For(for_init, opt_cond, opt_post, body, label) => {
            let mut new_variable_map = new_scope_identifier_map(variable_map);

            let for_init = resolve_for_init(for_init, &mut new_variable_map, counter);
            let opt_cond = resolve_optional_expr(opt_cond, &mut new_variable_map);
            let opt_post = resolve_optional_expr(opt_post, &mut new_variable_map);
            let body = resolve_statement(body.as_ref(), &mut new_variable_map, counter);

            parser::Statement::For(for_init, opt_cond, opt_post, Box::new(body), label.clone())
        }
        stmt => stmt.clone(),
    }
}

pub fn resolve_for_init(
    for_init: &parser::ForInit,
    variable_map: &mut IdentifierMap,
    counter: &mut usize,
) -> parser::ForInit {
    match for_init {
        parser::ForInit::InitDecl(decl) => parser::ForInit::InitDecl(
            resolve_block_scope_variable_declaration(decl, variable_map, counter),
        ),
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
        parser::Expr::Assignment(left, right, ty) => {
            let left = left.as_ref();
            let right = right.as_ref();

            parser::Expr::Assignment(
                Box::new(resolve_expr(left, identifier_map)),
                Box::new(resolve_expr(right, identifier_map)),
                ty.clone(),
            )
        }
        parser::Expr::Var(identifier, ty) => {
            if let Some(identifier_info) = identifier_map.get(identifier) {
                parser::Expr::Var(identifier_info.new_name.clone(), ty.clone())
            } else {
                panic!("Undeclared variable!");
            }
        }
        parser::Expr::Unary(op, e, ty) => {
            let e = e.as_ref();
            parser::Expr::Unary(
                op.clone(),
                Box::new(resolve_expr(e, identifier_map)),
                ty.clone(),
            )
        }
        parser::Expr::Binary(op, left, right, ty) => {
            let left = left.as_ref();
            let right = right.as_ref();

            parser::Expr::Binary(
                op.clone(),
                Box::new(resolve_expr(left, identifier_map)),
                Box::new(resolve_expr(right, identifier_map)),
                ty.clone(),
            )
        }
        parser::Expr::Conditional(cond, then_branch, else_branch, ty) => {
            let cond = cond.as_ref();
            let then_branch = then_branch.as_ref();
            let else_branch = else_branch.as_ref();

            parser::Expr::Conditional(
                Box::new(resolve_expr(cond, identifier_map)),
                Box::new(resolve_expr(then_branch, identifier_map)),
                Box::new(resolve_expr(else_branch, identifier_map)),
                ty.clone(),
            )
        }
        parser::Expr::FunctionCall(identifier, args, ty) => {
            if let Some(identifier_info) = identifier_map.get(identifier) {
                let new_func_name = identifier_info.new_name.clone();
                let mut new_args = Vec::new();
                for arg in args {
                    new_args.push(resolve_expr(arg, identifier_map));
                }
                parser::Expr::FunctionCall(new_func_name, new_args, ty.clone())
            } else {
                panic!("Undeclared function");
            }
        }
        parser::Expr::Cast(to_type, expr, ty) => {
            let expr = expr.as_ref();
            parser::Expr::Cast(
                to_type.clone(),
                Box::new(resolve_expr(expr, identifier_map)),
                ty.clone(),
            )
        }
        parser::Expr::AddressOf(expr, ty) => {
            let expr = expr.as_ref();
            parser::Expr::AddressOf(Box::new(resolve_expr(expr, identifier_map)), ty.clone())
        }
        parser::Expr::Dereference(expr, ty) => {
            let expr = expr.as_ref();
            parser::Expr::Dereference(Box::new(resolve_expr(expr, identifier_map)), ty.clone())
        }
        other => other.clone(),
    }
}
