use std::collections::HashMap;

use crate::{
    parser,
    symbol::{SymbolMetadata, SymbolTable, Type},
};

// First pass: Variable resolution
#[derive(Debug, Clone)]
pub struct IdentifierInfo {
    new_name: String,
    declared_in_scope: bool,
    has_linkage: bool,
}

type IdentifierMap = HashMap<String, IdentifierInfo>;

pub fn new_scope_identifier_map(identifier_map: &IdentifierMap) -> IdentifierMap {
    let mut new_identifier_map = identifier_map.clone();

    for identifier_info in new_identifier_map.values_mut() {
        identifier_info.declared_in_scope = false;
    }

    new_identifier_map
}

pub fn resolve_program(
    program: &parser::Program,
    symbol_table: &mut SymbolTable,
) -> parser::Program {
    let mut identifier_map = IdentifierMap::new();

    let parser::Program(declarations) = program;
    let mut new_declarations = Vec::new();
    for declaration in declarations {
        new_declarations.push(resolve_function_declaration(
            declaration,
            &mut identifier_map,
            symbol_table,
        ))
    }

    parser::Program(new_declarations)
}

pub fn resolve_declaration(
    declaration: &parser::Declaration,
    identifier_map: &mut IdentifierMap,
    symbol_table: &mut SymbolTable,
) -> parser::Declaration {
    match declaration {
        parser::Declaration::VarDecl(var_decl) => parser::Declaration::VarDecl(
            resolve_variable_declaration(var_decl, identifier_map, symbol_table),
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
                symbol_table,
            ))
        }
    }
}

pub fn resolve_block(
    block: &parser::Block,
    identifier_map: &mut IdentifierMap,
    symbol_table: &mut SymbolTable,
) -> parser::Block {
    let parser::Block(block_items) = block;
    let mut resolved_block_items = Vec::new();
    for item in block_items {
        resolved_block_items.push(resolve_block_item(item, identifier_map, symbol_table));
    }

    parser::Block(resolved_block_items)
}

pub fn resolve_block_item(
    block_item: &parser::BlockItem,
    identifier_map: &mut IdentifierMap,
    symbol_table: &mut SymbolTable,
) -> parser::BlockItem {
    match block_item {
        parser::BlockItem::D(declaration) => {
            parser::BlockItem::D(resolve_declaration(declaration, identifier_map, symbol_table))
        }
        parser::BlockItem::S(statement) => {
            parser::BlockItem::S(resolve_statement(statement, identifier_map, symbol_table))
        }
    }
}

pub fn resolve_variable_declaration(
    declaration: &parser::VariableDeclaration,
    identifier_map: &mut IdentifierMap,
    symbol_table: &mut SymbolTable,
) -> parser::VariableDeclaration {
    let parser::VariableDeclaration(name, init) = declaration;

    if let Some(identifier_info) = identifier_map.get(name) {
        if identifier_info.declared_in_scope {
            panic!("Duplicate variable declaration!");
        }
    }

    let new_name = symbol_table.generate_variable();
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
    symbol_table: &mut SymbolTable,
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
        new_params.push(resolve_parameter(parameter, &mut inner_map, symbol_table));
    }

    match body {
        Some(block) => {
            let new_body = resolve_block(block, &mut inner_map, symbol_table);
            parser::FunctionDeclaration(name.clone(), new_params, Some(new_body))
        }
        None => parser::FunctionDeclaration(name.clone(), new_params, None),
    }
}

pub fn resolve_parameter(
    parameter_name: &String,
    identifier_map: &mut IdentifierMap,
    symbol_table: &mut SymbolTable,
) -> String {
    if let Some(identifier_info) = identifier_map.get(parameter_name) {
        if identifier_info.declared_in_scope {
            panic!("Duplicate variable declaration!");
        }
    }

    let new_name = symbol_table.generate_variable();
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
            let mut new_variable_map = new_scope_identifier_map(variable_map);
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
            let mut new_variable_map = new_scope_identifier_map(variable_map);

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
    variable_map: &mut IdentifierMap,
    symbol_table: &mut SymbolTable,
) -> parser::ForInit {
    match for_init {
        parser::ForInit::InitDecl(decl) => {
            parser::ForInit::InitDecl(resolve_variable_declaration(decl, variable_map, symbol_table))
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
pub fn typecheck_program(program: &parser::Program, symbol_table: &mut SymbolTable) {
    let parser::Program(declarations) = program;
    let mut new_declarations = Vec::new();
    for declaration in declarations {
        new_declarations.push(typecheck_function_declaration(declaration, symbol_table))
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
    let parser::VariableDeclaration(_name, init) = variable_declaration;
    if let Some(e) = init.as_ref() {
        typecheck_expr(e, symbol_table);
    }
}

pub fn typecheck_function_declaration(
    function_declaration: &parser::FunctionDeclaration,
    symbol_table: &mut SymbolTable,
) {
    let parser::FunctionDeclaration(name, parameters, body) = function_declaration;
    let mut already_defined = false;

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

    symbol_table.generate_function(name, parameters.len(), body.is_some() || already_defined);

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

// Wrapper for semantic analysis passes
pub fn semantic_analysis(ast: &parser::Program, symbol_table: &mut SymbolTable) -> parser::Program {
    let identifier_resolution_ast = resolve_program(ast, symbol_table);
    let loop_labeling_ast = label_program(&identifier_resolution_ast);
    typecheck_program(&loop_labeling_ast, symbol_table);

    loop_labeling_ast
}
