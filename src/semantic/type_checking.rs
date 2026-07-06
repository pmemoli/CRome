use super::*;

use crate::{
    parser::{BinaryOperator, UnaryOperator},
    symbol::{InitialValue, StaticInit, SymbolMetadata},
};

// Third pass (Type Checking):

// Functions

// 1. Check that function declaration types and linkage are consistent everywhere
// 2. A function can't be defined more than once (not really type checking but easy to implement here)
// 3. A function can't be called with the wrong number of arguments

// Variables

// 1. Check that an initialized file scope value is only init with a constant expression
// 2. Check that declarations have consistent types (equal or coercible)

// Types (annotate, validate and coerce)

// 1. Annotate the AST with the type of each expression.
// 2. Cast sub expressions in binary expressions to the common type if possible
// 3. Cast the expression to the declaration/assignment type if possible
// 4. Raise error when operating with invalid types (like multiyplying pointers)
// 5. Not using an lvalue where one is required (such as the left side of an assignment)

// Symbol Table

// It also fills the symbol table with variable and function type, definition, storage duration and
// linkage information

pub fn typecheck_program(
    program: &parser::Program,
    symbol_table: &mut SymbolTable,
) -> parser::Program {
    let parser::Program(declarations) = program;

    let mut new_declarations = Vec::new();
    for declaration in declarations {
        new_declarations.push(typecheck_file_scope_declaration(declaration, symbol_table))
    }

    parser::Program(new_declarations)
}

pub fn typecheck_file_scope_declaration(
    declaration: &parser::Declaration,
    symbol_table: &mut SymbolTable,
) -> parser::Declaration {
    match declaration {
        parser::Declaration::VarDecl(var_decl) => parser::Declaration::VarDecl(
            typecheck_file_scope_variable_declaration(var_decl, symbol_table),
        ),
        parser::Declaration::FunDecl(func_decl) => {
            parser::Declaration::FunDecl(typecheck_function_declaration(func_decl, symbol_table))
        }
    }
}

pub fn typecheck_block_scope_declaration(
    declaration: &parser::Declaration,
    symbol_table: &mut SymbolTable,
) -> parser::Declaration {
    match declaration {
        parser::Declaration::VarDecl(var_decl) => parser::Declaration::VarDecl(
            typecheck_block_scope_variable_declaration(var_decl, symbol_table),
        ),
        parser::Declaration::FunDecl(func_decl) => {
            parser::Declaration::FunDecl(typecheck_function_declaration(func_decl, symbol_table))
        }
    }
}

pub fn typecheck_block_scope_variable_declaration(
    variable_declaration: &parser::VariableDeclaration,
    symbol_table: &mut SymbolTable,
) -> parser::VariableDeclaration {
    let parser::VariableDeclaration(identifier, init, ty, storage_class) = variable_declaration;

    // Populate the symbol table, type/specifier check and process the initializer
    let new_init = match storage_class {
        Some(parser::StorageClass::Extern) => {
            if init.is_some() {
                panic!("Local variable with extern storage duration can't be initialized");
            }

            // Only extern values can be redeclared in block scope
            if let Some(symbol_info) = symbol_table.get(identifier) {
                let prev_ty = &symbol_info.ty;
                if prev_ty != ty {
                    panic!("Incompatible type declarations");
                }
            } else {
                symbol_table.insert_static_variable(identifier, true, None, ty);
            }

            None
        }
        Some(parser::StorageClass::Static) => {
            // Convert the constant expression directly
            let new_init = init.as_ref().map(|e| static_convert_constant_expr(e, ty));

            let initial_value = match new_init.as_ref() {
                Some(parser::Expr::Constant(cons, _)) => {
                    InitialValue::Initial(constant_to_static_init(cons))
                }
                None => InitialValue::Initial(StaticInit::IntInit(0)),
                Some(_) => panic!("Non-constant expression used to initialize static variable"),
            };

            symbol_table.insert_static_variable(identifier, false, Some(initial_value), ty);

            new_init
        }
        None => {
            symbol_table.insert_local_variable(identifier, ty);

            // Cast the typed init to the variable type if it exists
            let typed_init = init.as_ref().map(|e| typecheck_expr(e, symbol_table));
            typed_init
                .as_ref()
                .map(|e| convert_by_assignment(e, ty.clone()))
        }
    };

    parser::VariableDeclaration(
        identifier.clone(),
        new_init,
        ty.clone(),
        storage_class.clone(),
    )
}

// Converts constant expression directly without casting
enum ConstVal {
    Float(f64),
    Integer(i128),
}

pub fn static_convert_constant_expr(expr: &parser::Expr, ty: &Type) -> parser::Expr {
    let parser::Expr::Constant(cons, _) = expr else {
        panic!("Expected constant expression for static variable initialization")
    };

    let val = match cons {
        parser::Const::ConstInt(i) => ConstVal::Integer(*i as i128),
        parser::Const::ConstLong(i) => ConstVal::Integer(*i as i128),
        parser::Const::ConstUInt(u) => ConstVal::Integer(*u as i128),
        parser::Const::ConstULong(u) => ConstVal::Integer(*u as i128),
        parser::Const::ConstDouble(f) => ConstVal::Float(*f as f64),
        parser::Const::ConstFloat(f) => ConstVal::Float(*f as f64),
    };

    // Rust preserves C standards for integer conversions
    let new_cons = match ty {
        Type::Int => parser::Const::ConstInt(match val {
            ConstVal::Integer(i) => i as i32,
            ConstVal::Float(f) => f as i32,
        }),
        Type::Long => parser::Const::ConstLong(match val {
            ConstVal::Integer(i) => i as i64,
            ConstVal::Float(f) => f as i64,
        }),
        Type::UInt => parser::Const::ConstUInt(match val {
            ConstVal::Integer(i) => i as u32,
            ConstVal::Float(f) => f as u32,
        }),
        Type::ULong => parser::Const::ConstULong(match val {
            ConstVal::Integer(i) => i as u64,
            ConstVal::Float(f) => f as u64,
        }),
        Type::Double => parser::Const::ConstDouble(match val {
            ConstVal::Integer(i) => i as f64,
            ConstVal::Float(f) => f as f64,
        }),
        Type::Float => parser::Const::ConstFloat(match val {
            ConstVal::Integer(i) => i as f32,
            ConstVal::Float(f) => f as f32,
        }),
        Type::Pointer(_) => match val {
            ConstVal::Integer(i) => {
                if i == 0 {
                    parser::Const::ConstULong(0)
                } else {
                    panic!(
                        "Type Error: Non-null pointer constant used to initialize static variable"
                    )
                }
            }
            ConstVal::Float(_) => {
                panic!("Type Error: Floating point constant used to initialize pointer")
            }
        },
        _ => panic!("Unsupported type for static variable initialization"),
    };

    parser::Expr::Constant(new_cons, Some(ty.clone()))
}

pub fn constant_to_static_init(cons: &parser::Const) -> StaticInit {
    match cons {
        parser::Const::ConstInt(i) => StaticInit::IntInit(*i),
        parser::Const::ConstLong(i) => StaticInit::LongInit(*i),
        parser::Const::ConstUInt(u) => StaticInit::UIntInit(*u),
        parser::Const::ConstULong(u) => StaticInit::ULongInit(*u),
        parser::Const::ConstDouble(f) => StaticInit::DoubleInit(*f),
        parser::Const::ConstFloat(f) => StaticInit::FloatInit(*f),
    }
}

pub fn typecheck_file_scope_variable_declaration(
    variable_declaration: &parser::VariableDeclaration,
    symbol_table: &mut SymbolTable,
) -> parser::VariableDeclaration {
    let parser::VariableDeclaration(identifier, init, ty, storage_class) = variable_declaration;

    // Convert the init directly
    let new_init = init.as_ref().map(|e| static_convert_constant_expr(e, ty));

    // Resolve initial constant value and convert it if necessary
    let mut initial_value = match new_init.as_ref() {
        Some(parser::Expr::Constant(cons, _)) => {
            Some(InitialValue::Initial(constant_to_static_init(cons)))
        }
        None => match storage_class {
            Some(parser::StorageClass::Extern) => None,
            _ => Some(InitialValue::Tentative),
        },
        Some(_) => panic!("Non-constant expression used to initialize static variable"),
    };

    // Populate the symbol table and type/specifier check
    let static_specifier = matches!(storage_class, Some(parser::StorageClass::Static));
    let extern_specifier = matches!(storage_class, Some(parser::StorageClass::Extern));
    let mut global = !static_specifier;

    if let Some(symbol_info) = symbol_table.map.get(identifier) {
        if symbol_info.ty != ty.clone() {
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
                    (init @ Some(InitialValue::Initial(_)), _) => init,
                    (_, init @ Some(InitialValue::Initial(_))) => init,

                    // Cases when neither of them is initialized
                    (init @ Some(InitialValue::Tentative), _) => init,
                    (_, init) => init,
                };
            }
            _ => panic!(
                "Ill parsing of file scope variable declarations (one of them as non-static storage duration)"
            ),
        }
    }

    symbol_table.insert_static_variable(identifier, global, initial_value, ty);

    parser::VariableDeclaration(
        identifier.clone(),
        new_init,
        ty.clone(),
        storage_class.clone(),
    )
}

pub fn typecheck_function_declaration(
    function_declaration: &parser::FunctionDeclaration,
    symbol_table: &mut SymbolTable,
) -> parser::FunctionDeclaration {
    let parser::FunctionDeclaration(name, parameters, body, ty, storage_class) =
        function_declaration;

    let mut already_defined = false;
    let static_specifier = matches!(storage_class, Some(parser::StorageClass::Static));

    // Unless previous declaration is static, the function is global w/o static
    let mut global = !static_specifier;

    // 1. Check that function declarations types and linkage is consistent everywhere.
    // 2. Check that the function is not defined more than once.
    if let Some(symbol_info) = symbol_table.map.get(name) {
        if symbol_info.ty != ty.clone() {
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

    let Type::FunType(param_types, ret_ty) = ty else {
        panic!("Function declaration with non-function type")
    };

    symbol_table.insert_function(name, ty, already_defined || body.is_some(), global);

    for (param, param_ty) in parameters.iter().zip(param_types.iter()) {
        symbol_table.insert_local_variable(param, param_ty);
    }

    let typechecked_body = body
        .as_ref()
        .map(|b| typecheck_block(b, symbol_table, &Some(ret_ty.as_ref().clone())));

    parser::FunctionDeclaration(
        name.clone(),
        parameters.clone(),
        typechecked_body,
        ty.clone(),
        storage_class.clone(),
    )
}

pub fn typecheck_block(
    block: &parser::Block,
    symbol_table: &mut SymbolTable,
    enclosing_func_ty: &Option<Type>,
) -> parser::Block {
    let parser::Block(block_items) = block;

    let mut new_block_items = Vec::new();
    for block_item in block_items {
        new_block_items.push(match block_item {
            parser::BlockItem::D(declaration) => {
                parser::BlockItem::D(typecheck_block_scope_declaration(declaration, symbol_table))
            }

            parser::BlockItem::S(statement) => parser::BlockItem::S(typecheck_statement(
                statement,
                symbol_table,
                enclosing_func_ty,
            )),
        });
    }

    parser::Block(new_block_items)
}

pub fn typecheck_statement(
    statement: &parser::Statement,
    symbol_table: &mut SymbolTable,
    enclosing_func_ty: &Option<Type>,
) -> parser::Statement {
    match statement {
        parser::Statement::Return(expr) => {
            if let Some(enclosing_func_ty) = enclosing_func_ty {
                let typed_expr = typecheck_expr(expr, symbol_table);
                let converted_expr = convert_by_assignment(&typed_expr, enclosing_func_ty.clone());
                parser::Statement::Return(converted_expr)
            } else {
                panic!("Return statement not inside a function")
            }
        }
        parser::Statement::Expression(expr) => {
            parser::Statement::Expression(typecheck_expr(expr, symbol_table))
        }
        parser::Statement::If(cond, then_branch, else_branch) => parser::Statement::If(
            typecheck_expr(cond, symbol_table),
            Box::new(typecheck_statement(
                then_branch.as_ref(),
                symbol_table,
                enclosing_func_ty,
            )),
            else_branch.as_ref().map(|s| {
                Box::new(typecheck_statement(
                    s.as_ref(),
                    symbol_table,
                    enclosing_func_ty,
                ))
            }),
        ),
        parser::Statement::Compound(block) => {
            parser::Statement::Compound(typecheck_block(block, symbol_table, enclosing_func_ty))
        }
        parser::Statement::While(cond, body, label) => parser::Statement::While(
            typecheck_expr(cond, symbol_table),
            Box::new(typecheck_statement(
                body.as_ref(),
                symbol_table,
                enclosing_func_ty,
            )),
            label.clone(),
        ),
        parser::Statement::DoWhile(body, cond, label) => parser::Statement::DoWhile(
            Box::new(typecheck_statement(
                body.as_ref(),
                symbol_table,
                enclosing_func_ty,
            )),
            typecheck_expr(cond, symbol_table),
            label.clone(),
        ),
        parser::Statement::For(init_1, init_2, init_3, body, label) => {
            let new_init_1 = match init_1 {
                parser::ForInit::InitDecl(decl) => {
                    let parser::VariableDeclaration(_, _, _, storage_class) = decl;
                    if storage_class.is_some() {
                        panic!("For loop initializer can't have a storage class specifier");
                    }

                    parser::ForInit::InitDecl(typecheck_block_scope_variable_declaration(
                        decl,
                        symbol_table,
                    ))
                }
                parser::ForInit::InitExp(opt_expr) => parser::ForInit::InitExp(
                    opt_expr.as_ref().map(|e| typecheck_expr(e, symbol_table)),
                ),
            };

            let new_init_2 = init_2.as_ref().map(|e| typecheck_expr(e, symbol_table));
            let new_init_3 = init_3.as_ref().map(|e| typecheck_expr(e, symbol_table));
            let new_body = Box::new(typecheck_statement(
                body.as_ref(),
                symbol_table,
                enclosing_func_ty,
            ));

            parser::Statement::For(new_init_1, new_init_2, new_init_3, new_body, label.clone())
        }
        parser::Statement::Null | parser::Statement::Break(_) | parser::Statement::Continue(_) => {
            statement.clone()
        }
    }
}

pub fn typecheck_expr(expr: &parser::Expr, symbol_table: &mut SymbolTable) -> parser::Expr {
    match expr {
        parser::Expr::FunctionCall(name, arguments, _) => {
            let identifier_info = symbol_table.get(name).expect("Undeclared function");

            let f_type = identifier_info.ty.clone();
            match f_type {
                Type::FunType(param_types, ret_type) => {
                    if param_types.len() != arguments.len() {
                        panic!("Function called with wrong amount of parameters")
                    };

                    let mut typed_arguments = Vec::new();
                    for (arg, param_ty) in arguments.iter().zip(param_types.iter()) {
                        let typed_arg = typecheck_expr(arg, symbol_table);
                        let converted_arg = convert_by_assignment(&typed_arg, param_ty.clone());
                        typed_arguments.push(converted_arg);
                    }

                    let new_expr = parser::Expr::FunctionCall(name.clone(), typed_arguments, None);
                    set_type(&new_expr, ret_type.as_ref().clone())
                }
                _ => panic!("Variable name used as function"),
            }
        }

        parser::Expr::Var(name, _) => {
            if let Some(info) = symbol_table.get(name) {
                let st_type = &info.ty;
                match st_type {
                    Type::FunType(_, _) => panic!("Function name used as variable"),
                    _ => set_type(expr, st_type.clone()),
                }
            } else {
                panic!("Undeclared variable");
            }
        }

        parser::Expr::Assignment(left, right, _) => {
            let typed_left = typecheck_expr(left.as_ref(), symbol_table);
            let typed_right = typecheck_expr(right.as_ref(), symbol_table);
            let left_ty = get_type(&typed_left);

            let converted_right = convert_by_assignment(&typed_right, left_ty.clone());

            let new_expr =
                parser::Expr::Assignment(Box::new(typed_left), Box::new(converted_right), None);

            set_type(&new_expr, left_ty)
        }

        parser::Expr::Unary(op, inner, _) => {
            let typed_inner = typecheck_expr(inner.as_ref(), symbol_table);
            let inner_ty = get_type(&typed_inner);

            match op {
                UnaryOperator::Complement | UnaryOperator::Negate => {
                    if inner_ty.is_floating_point() {
                        panic!("Can't take the bitwise complement of a float");
                    };

                    if inner_ty.is_pointer() {
                        panic!("Can't take the bitwise complement of a pointer");
                    };

                    let new_expr =
                        parser::Expr::Unary(op.clone(), Box::new(typed_inner.clone()), None);
                    set_type(&new_expr, inner_ty)
                }
                UnaryOperator::Not => {
                    let new_expr =
                        parser::Expr::Unary(op.clone(), Box::new(typed_inner.clone()), None);
                    set_type(&new_expr, Type::Int)
                }
            }
        }

        parser::Expr::Binary(op, left, right, _) => {
            let typed_left = typecheck_expr(left.as_ref(), symbol_table);
            let left_ty = get_type(&typed_left);
            let typed_right = typecheck_expr(right.as_ref(), symbol_table);
            let right_ty = get_type(&typed_right);

            let common_ty = if left_ty.is_pointer() || right_ty.is_pointer() {
                get_common_pointer_type(&typed_left, &typed_right)
            } else {
                get_common_type(&typed_left, &typed_right)
            };

            // Invalid operations
            if matches!(op, BinaryOperator::Remainder) && common_ty.is_floating_point() {
                panic!("Type Error: Can't take the remainder of a float");
            }

            if matches!(
                op,
                BinaryOperator::Divide | BinaryOperator::Multiply | BinaryOperator::Remainder
            ) && common_ty.is_pointer()
            {
                panic!("Type Error: Can't multiply, divide or take the remainder of a pointer");
            }

            match op {
                // Logical operators always return int and don't cast their operators
                parser::BinaryOperator::And | parser::BinaryOperator::Or => {
                    let new_expr = parser::Expr::Binary(
                        op.clone(),
                        Box::new(typed_left),
                        Box::new(typed_right),
                        None,
                    );
                    set_type(&new_expr, Type::Int)
                }
                _ => {
                    let cast_left = convert_to(&typed_left, common_ty.clone());
                    let cast_right = convert_to(&typed_right, common_ty.clone());
                    let new_expr = parser::Expr::Binary(
                        op.clone(),
                        Box::new(cast_left),
                        Box::new(cast_right),
                        None,
                    );

                    match op {
                        // Arithmetic operators return the common type of their operands
                        parser::BinaryOperator::Add
                        | parser::BinaryOperator::Subtract
                        | parser::BinaryOperator::Multiply
                        | parser::BinaryOperator::Divide
                        | parser::BinaryOperator::Remainder => set_type(&new_expr, common_ty),

                        // Comparison operators always return int, but need to be casted to the
                        // same type first
                        _ => set_type(&new_expr, Type::Int),
                    }
                }
            }
        }

        parser::Expr::Conditional(cond, then_branch, else_branch, _) => {
            let typed_cond = typecheck_expr(cond.as_ref(), symbol_table);
            let typed_then = typecheck_expr(then_branch.as_ref(), symbol_table);
            let typed_else = typecheck_expr(else_branch.as_ref(), symbol_table);

            let common_ty = get_common_type(&typed_then, &typed_else);
            let cast_then = convert_to(&typed_then, common_ty.clone());
            let cast_else = convert_to(&typed_else, common_ty.clone());

            let new_expr = parser::Expr::Conditional(
                Box::new(typed_cond),
                Box::new(cast_then),
                Box::new(cast_else),
                None,
            );

            set_type(&new_expr, common_ty)
        }
        parser::Expr::Constant(cons, _) => match cons {
            parser::Const::ConstInt(_) => set_type(expr, Type::Int),
            parser::Const::ConstUInt(_) => set_type(expr, Type::UInt),
            parser::Const::ConstLong(_) => set_type(expr, Type::Long),
            parser::Const::ConstULong(_) => set_type(expr, Type::ULong),
            parser::Const::ConstFloat(_) => set_type(expr, Type::Float),
            parser::Const::ConstDouble(_) => set_type(expr, Type::Double),
        },
        parser::Expr::Cast(ty, inner, _) => {
            let typed_inner = typecheck_expr(inner.as_ref(), symbol_table);
            let inner_ty = get_type(&typed_inner);

            if ty.is_floating_point() && inner_ty.is_pointer() {
                panic!("Cannot cast pointer to floating point type");
            }
            if ty.is_pointer() && inner_ty.is_floating_point() {
                panic!("Cannot cast floating point type to pointer");
            }

            let new_expr = parser::Expr::Cast(ty.clone(), Box::new(typed_inner), None);
            set_type(&new_expr, ty.clone())
        }
        parser::Expr::Dereference(inner, _) => {
            let typed_inner = typecheck_expr(inner.as_ref(), symbol_table);
            match get_type(&typed_inner) {
                Type::Pointer(inner_ty) => set_type(
                    &parser::Expr::Dereference(Box::new(typed_inner), None),
                    *inner_ty,
                ),
                _ => panic!("Type error: Dereference operator can only be applied to pointers"),
            }
        }
        parser::Expr::AddressOf(inner, _) => {
            let inner_expr = inner.as_ref();

            if inner_expr.is_lvalue() {
                let typed_inner = typecheck_expr(inner_expr, symbol_table);
                let inner_ty = get_type(&typed_inner);
                let new_expr = parser::Expr::AddressOf(Box::new(typed_inner), None);
                set_type(&new_expr, Type::Pointer(Box::new(inner_ty)))
            } else {
                panic!("Type error: Address-of operator can only be applied to lvalues")
            }
        }
    }
}

pub fn set_type(expr: &parser::Expr, ty: Type) -> parser::Expr {
    let mut expr = expr.clone();
    let some_ty = Some(ty);

    match &mut expr {
        parser::Expr::FunctionCall(_, _, t)
        | parser::Expr::Var(_, t)
        | parser::Expr::Assignment(_, _, t)
        | parser::Expr::Unary(_, _, t)
        | parser::Expr::Binary(_, _, _, t)
        | parser::Expr::Conditional(_, _, _, t)
        | parser::Expr::Cast(_, _, t)
        | parser::Expr::AddressOf(_, t)
        | parser::Expr::Dereference(_, t)
        | parser::Expr::Constant(_, t) => *t = some_ty,
    };

    expr
}

pub fn get_common_type(left: &parser::Expr, right: &parser::Expr) -> Type {
    let left_ty = get_type(left);
    let right_ty = get_type(right);

    if matches!(left_ty, Type::Double) | matches!(right_ty, Type::Double) {
        return Type::Double;
    }
    if matches!(left_ty, Type::Float) | matches!(right_ty, Type::Float) {
        return Type::Float;
    }

    // Implements C standards for common type coercion
    if left_ty == right_ty {
        left_ty
    } else if left_ty.byte_size() == right_ty.byte_size() {
        if left_ty.signed() { right_ty } else { left_ty }
    } else if left_ty.byte_size() > right_ty.byte_size() {
        left_ty
    } else {
        right_ty
    }
}

pub fn get_common_pointer_type(left: &parser::Expr, right: &parser::Expr) -> Type {
    let left_ty = get_type(left);
    let right_ty = get_type(right);

    if left_ty == right_ty {
        left_ty
    } else if is_null_pointer_constant(left) {
        right_ty
    } else if is_null_pointer_constant(right) {
        left_ty
    } else {
        panic!("Type error: Pointer arithmetic with incompatible pointer types");
    }
}

pub fn is_null_pointer_constant(expr: &parser::Expr) -> bool {
    match expr {
        parser::Expr::Constant(parser::Const::ConstInt(0), _) => true,
        parser::Expr::Constant(parser::Const::ConstLong(0), _) => true,
        parser::Expr::Constant(parser::Const::ConstUInt(0), _) => true,
        parser::Expr::Constant(parser::Const::ConstULong(0), _) => true,
        _ => false,
    }
}

pub fn convert_to(expr: &parser::Expr, target_ty: Type) -> parser::Expr {
    if get_type(expr) == target_ty {
        return expr.clone();
    }

    let cast_expr = parser::Expr::Cast(target_ty.clone(), Box::new(expr.clone()), None);
    set_type(&cast_expr, target_ty)
}

pub fn convert_by_assignment(expr: &parser::Expr, target_ty: Type) -> parser::Expr {
    let e_ty = get_type(expr);

    if e_ty == target_ty {
        expr.clone()
    } else if e_ty.is_arithmetic() && target_ty.is_arithmetic() {
        convert_to(expr, target_ty)
    } else if e_ty.is_pointer() && target_ty.is_pointer() {
        convert_to(expr, target_ty)
    } else if is_null_pointer_constant(expr) && target_ty.is_pointer() {
        convert_to(expr, target_ty)
    } else {
        panic!(
            "Type error: Cannot assign expression of type {:?} to variable of type {:?}",
            e_ty, target_ty
        );
    }
}
