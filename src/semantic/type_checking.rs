use super::*;

use crate::symbol::{InitialValue, SymbolMetadata, Type};

// Third pass (Type Checking):

// 1. Check that function declaration types and linkage are consistent everywhere
// 2. A function can't be defined more than once (not really type checking but easy to implement here)
// 3. A function can't be called with the wrong number of arguments

// 1. Check that an initialized file scope value is only init with a constant expression

// It also fills the symbol table with variable and function type, definition, storage duration and
// linkage information

pub fn typecheck_program(program: &parser::Program, symbol_table: &mut SymbolTable) {
    let parser::Program(declarations) = program;
    let mut new_declarations = Vec::new();
    for declaration in declarations {
        new_declarations.push(typecheck_file_scope_declaration(declaration, symbol_table))
    }
}

pub fn typecheck_file_scope_declaration(
    declaration: &parser::Declaration,
    symbol_table: &mut SymbolTable,
) {
    match declaration {
        parser::Declaration::VarDecl(var_decl) => {
            typecheck_file_scope_variable_declaration(var_decl, symbol_table);
        }
        parser::Declaration::FunDecl(func_decl) => {
            typecheck_function_declaration(func_decl, symbol_table);
        }
    }
}

pub fn typecheck_block_scope_declaration(
    declaration: &parser::Declaration,
    symbol_table: &mut SymbolTable,
) {
    match declaration {
        parser::Declaration::VarDecl(var_decl) => {
            typecheck_block_scope_variable_declaration(var_decl, symbol_table);
        }
        parser::Declaration::FunDecl(func_decl) => {
            typecheck_function_declaration(func_decl, symbol_table);
        }
    }
}

pub fn typecheck_block_scope_variable_declaration(
    variable_declaration: &parser::VariableDeclaration,
    symbol_table: &mut SymbolTable,
) {
    let parser::VariableDeclaration(identifier, init, storage_class) = variable_declaration;

    match storage_class {
        Some(parser::StorageClass::Extern) => {
            if init.is_some() {
                panic!("Local variable with extern storage duration can't be initialized");
            }
            if let Some(symbol_info) = symbol_table.map.get(identifier) {
                if symbol_info.ty != Type::Int {
                    panic!("Function redeclared as variable")
                }
            } else {
                symbol_table.insert_static_variable(identifier, true, None)
            }
        }
        Some(parser::StorageClass::Static) => {
            let initial_value = match init {
                Some(parser::Expr::Constant(i)) => InitialValue::Initial(*i),
                None => InitialValue::Initial(0),
                _ => panic!("Non-constant expression used to initialize local static variable"),
            };

            symbol_table.insert_static_variable(identifier, false, Some(initial_value));
        }
        None => {
            symbol_table.insert_local_variable(identifier);
            if let Some(e) = init.as_ref() {
                typecheck_expr(e, symbol_table);
            }
        }
    }
}

pub fn typecheck_file_scope_variable_declaration(
    variable_declaration: &parser::VariableDeclaration,
    symbol_table: &mut SymbolTable,
) {
    let parser::VariableDeclaration(identifier, init, storage_class) = variable_declaration;

    let mut initial_value = match init {
        Some(parser::Expr::Constant(i)) => Some(InitialValue::Initial(*i)),
        None => {
            if matches!(storage_class, Some(parser::StorageClass::Extern)) {
                None
            } else {
                Some(InitialValue::Tentative)
            }
        }
        _ => panic!("Non-constant expression used to initialize file scope variable"),
    };

    let static_specifier = matches!(storage_class, Some(parser::StorageClass::Static));
    let extern_specifier = matches!(storage_class, Some(parser::StorageClass::Extern));

    // Unless specifier is extern and previous specifier is static
    let mut global = !static_specifier;

    if let Some(symbol_info) = symbol_table.map.get(identifier) {
        if symbol_info.ty != Type::Int {
            panic!("Incompatible type declarations")
        }

        match symbol_info.metadata.clone() {
            SymbolMetadata::StaticVariable {
                global: prev_global,
                initial_value: prev_initial_value,
            } => {
                // extern specifier matches previous linkage
                if extern_specifier {
                    global = prev_global;
                } else if prev_global != global {
                    panic!("File scope variable declarations with different linkage");
                }

                // validates that initialization is consistent and resolves multiple declarations
                initial_value = match (prev_initial_value, initial_value) {
                    // Cases when at least one of them is initialized
                    (Some(InitialValue::Initial(_)), Some(InitialValue::Initial(_))) => {
                        panic!("Conflicting file scope variable definitions");
                    }
                    (Some(InitialValue::Initial(i)), _) => Some(InitialValue::Initial(i)),
                    (_, init @ Some(InitialValue::Initial(_))) => init,

                    // Cases when neither of them is initialized
                    (Some(InitialValue::Tentative), _) => Some(InitialValue::Tentative),
                    (_, init) => init,
                };
            }
            _ => panic!(
                "Ill parsing of file scope variable declarations (one of them as non-static storage duration)"
            ),
        }
    }

    symbol_table.insert_static_variable(identifier, global, initial_value);
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
    let static_specifier = matches!(storage_class, Some(parser::StorageClass::Static));

    // Unless previous declaration is static
    let mut global = !static_specifier;

    // 1. Check that function declarations types and linkage is consistent everywhere.
    // 2. Check that the function is not defined more than once.
    if let Some(symbol_info) = symbol_table.map.get(name) {
        if symbol_info.ty != Type::FunType(parameters.len()) {
            panic!("Incompatible declarations")
        }

        match symbol_info.metadata {
            SymbolMetadata::Function {
                defined: prev_defined,
                global: prev_global,
            } => {
                already_defined = prev_defined;
                if prev_defined && body.is_some() {
                    panic!("Function defined twice");
                }

                if prev_global && static_specifier {
                    panic!("Static function declaration follows non-static function declaration");
                }

                global = prev_defined;
            }
            _ => panic!("Previous non-function declaration with the same name"),
        }
    }

    symbol_table.insert_function(
        name,
        parameters.len(),
        already_defined || body.is_some(),
        global,
    );

    if let Some(block) = body.as_ref() {
        typecheck_block(block, symbol_table);
    }
}

pub fn typecheck_block(block: &parser::Block, symbol_table: &mut SymbolTable) {
    let parser::Block(block_items) = block;
    for block_item in block_items {
        match block_item {
            parser::BlockItem::D(declaration) => {
                typecheck_block_scope_declaration(declaration, symbol_table)
            }
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
