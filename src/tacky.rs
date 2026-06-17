use crate::parser;
use crate::semantic::get_type;
use crate::symbol::{InitialValue, StaticInit, SymbolMetadata, SymbolTable, Type};

// program = Program(top_level*)
#[derive(Debug)]
pub struct Program(pub Vec<TopLevel>);

// top_level = Function(identifier, bool global, identifier* params, instruction* body)
//     | StaticVariable(identifier, bool global, type t, static_init init)
#[derive(Debug)]
pub enum TopLevel {
    Function(String, bool, Vec<String>, Vec<Instruction>),
    StaticVariable(String, bool, Type, StaticInit),
}

// instruction = Return(val)
//     | SignExtend(val src, val dst)
//     | Truncate(val src, val dst)
//     | ZeroExtend(val src, val dst)
//     | DoubleToInt(val src, val dst)
//     | DoubleToUInt(val src, val dst)
//     | IntToDouble(val src, val dst)
//     | UIntToDouble(val src, val dst)
//     | Unary(unary_operator, val src, val dst)
//     | Binary(binary_operator, val src1, val src2, val dst)
//     | Copy(val src, val dst)
//     | Jump(identifier target)
//     | JumpIfZero(val condition, identifier target)
//     | JumpIfNotZero(val condition, identifier target)
//     | Label(identifier)
//     | FunCall(identifier fun_name, val* args, val dst)
#[derive(Debug)]
pub enum Instruction {
    Return(Val),
    SignExtend(Val, Val),
    ZeroExtend(Val, Val),
    DoubleToInt(Val, Val),
    DoubleToUInt(Val, Val),
    IntToDouble(Val, Val),
    UIntToDouble(Val, Val),
    Truncate(Val, Val),
    Unary(UnaryOperator, Val, Val),
    Binary(BinaryOperator, Val, Val, Val),
    Copy(Val, Val),
    Jump(String),
    JumpIfZero(Val, String),
    JumpIfNotZero(Val, String),
    Label(String),
    FunCall(String, Vec<Val>, Val),
}

// val = Constant(const) | Var(identifier)
#[derive(Debug, Clone)]
pub enum Val {
    Constant(parser::Const),
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

pub fn generate_unique_variable(symbol_table: &mut SymbolTable, ty: &Type) -> String {
    let var_name = symbol_table.unique_var_name();
    symbol_table.insert_local_variable(&var_name, ty);
    var_name
}

pub fn ast_program_to_tacky(
    ast_program: &parser::Program,
    symbol_table: &mut SymbolTable,
) -> Program {
    let parser::Program(ast_declarations) = ast_program;
    let mut label_idx = 0;

    let mut tacky_top_defs: Vec<TopLevel> = Vec::new();
    for ast_decl in ast_declarations {
        let tacky_top_func_defs =
            ast_file_scope_declaration_to_tacky(ast_decl, symbol_table, &mut label_idx);

        if let Some(f) = tacky_top_func_defs {
            tacky_top_defs.push(f);
        }
    }

    for static_def in convert_symbols_to_tacky(symbol_table) {
        tacky_top_defs.push(static_def);
    }

    Program(tacky_top_defs)
}

pub fn convert_symbols_to_tacky(symbol_table: &mut SymbolTable) -> Vec<TopLevel> {
    let mut tacky_defs = Vec::new();

    for (name, info) in &symbol_table.map {
        let ty = info.ty.clone();

        if let SymbolMetadata::StaticVariable {
            global,
            initial_value,
        } = info.metadata.clone()
        {
            match initial_value {
                Some(InitialValue::Tentative) => {
                    let static_init = match ty {
                        Type::Int => StaticInit::IntInit(0),
                        Type::UInt => StaticInit::UIntInit(0),
                        Type::Long => StaticInit::LongInit(0),
                        Type::ULong => StaticInit::ULongInit(0),
                        Type::Double => StaticInit::DoubleInit(0.),
                        Type::FunType(_, _) => {
                            panic!("Function type cannot be used for static variable: {}", name)
                        }
                    };

                    tacky_defs.push(TopLevel::StaticVariable(
                        name.clone(),
                        global,
                        ty,
                        static_init,
                    ))
                }
                Some(InitialValue::Initial(static_init)) => tacky_defs.push(
                    TopLevel::StaticVariable(name.clone(), global, ty, static_init),
                ),
                None => {}
            }
        }
    }

    tacky_defs
}

pub fn ast_file_scope_declaration_to_tacky(
    ast_declaration: &parser::Declaration,
    symbol_table: &mut SymbolTable,
    label_idx: &mut usize,
) -> Option<TopLevel> {
    match ast_declaration {
        parser::Declaration::FunDecl(function_declaration) => {
            ast_function_declaration_to_tacky(function_declaration, symbol_table, label_idx)
        }
        _ => None, // Static variables are processed in a second pass
    }
}

pub fn ast_function_declaration_to_tacky(
    ast_function: &parser::FunctionDeclaration,
    symbol_table: &mut SymbolTable,
    label_idx: &mut usize,
) -> Option<TopLevel> {
    let parser::FunctionDeclaration(identifier, parameters, block, _, _) = ast_function;

    let mut instructions = Vec::new();

    let symbol_info = symbol_table
        .get(identifier)
        .expect("Function not found in symbol table");

    let SymbolMetadata::Function { global, .. } = symbol_info.metadata else {
        panic!("Expected function symbol metadata.");
    };

    // Only emit code for defined functions.
    if let Some(block) = block.as_ref() {
        ast_block_to_tacky(block, &mut instructions, symbol_table, label_idx);
        instructions.push(Instruction::Return(Val::Constant(parser::Const::ConstInt(
            0,
        ))));
        Some(TopLevel::Function(
            identifier.clone(),
            global.clone(),
            parameters.clone(),
            instructions,
        ))
    } else {
        None
    }
}

pub fn ast_block_scope_variable_declaration_to_tacky(
    ast_declaration: &parser::VariableDeclaration,
    instructions: &mut Vec<Instruction>,
    symbol_table: &mut SymbolTable,
    label_idx: &mut usize,
) {
    let parser::VariableDeclaration(name, init, _, storage_class) = ast_declaration;

    if storage_class.is_some() {
        return; // Static variables are processed in a second pass
    }

    if let Some(e) = init.as_ref() {
        let value = ast_expression_to_tacky(e, instructions, symbol_table, label_idx);
        let dst = Val::Var(name.clone());
        instructions.push(Instruction::Copy(value, dst.clone()));
    };
}

pub fn ast_block_scope_declaration_to_tacky(
    ast_declaration: &parser::Declaration,
    instructions: &mut Vec<Instruction>,
    symbol_table: &mut SymbolTable,
    label_idx: &mut usize,
) {
    match ast_declaration {
        parser::Declaration::VarDecl(variable_declaration) => {
            ast_block_scope_variable_declaration_to_tacky(
                variable_declaration,
                instructions,
                symbol_table,
                label_idx,
            )
        }
        _ => {}
    }
}

pub fn ast_block_to_tacky(
    ast_block: &parser::Block,
    instructions: &mut Vec<Instruction>,
    symbol_table: &mut SymbolTable,
    label_idx: &mut usize,
) {
    let parser::Block(block_items) = ast_block;

    for block_item in block_items {
        ast_block_item_to_tacky(block_item, instructions, symbol_table, label_idx);
    }
}

pub fn ast_block_item_to_tacky(
    ast_block: &parser::BlockItem,
    instructions: &mut Vec<Instruction>,
    symbol_table: &mut SymbolTable,
    label_idx: &mut usize,
) {
    match ast_block {
        parser::BlockItem::S(statement) => {
            ast_statement_to_tacky(statement, instructions, symbol_table, label_idx)
        }
        parser::BlockItem::D(declaration) => {
            ast_block_scope_declaration_to_tacky(declaration, instructions, symbol_table, label_idx)
        }
    }
}

pub fn ast_for_init_to_tacky(
    ast_init: &parser::ForInit,
    instructions: &mut Vec<Instruction>,
    symbol_table: &mut SymbolTable,
    label_idx: &mut usize,
) {
    match ast_init {
        parser::ForInit::InitDecl(declaration) => ast_block_scope_variable_declaration_to_tacky(
            declaration,
            instructions,
            symbol_table,
            label_idx,
        ),
        parser::ForInit::InitExp(exp) => {
            if let Some(e) = exp.as_ref() {
                ast_expression_to_tacky(e, instructions, symbol_table, label_idx);
            }
        }
    }
}

pub fn ast_statement_to_tacky(
    ast_statement: &parser::Statement,
    instructions: &mut Vec<Instruction>,
    symbol_table: &mut SymbolTable,
    label_idx: &mut usize,
) {
    match ast_statement {
        parser::Statement::Return(e) => {
            let value = ast_expression_to_tacky(e, instructions, symbol_table, label_idx);
            instructions.push(Instruction::Return(value));
        }
        parser::Statement::Expression(e) => {
            ast_expression_to_tacky(e, instructions, symbol_table, label_idx);
        }
        parser::Statement::If(cond_expr, then_stmt, else_stmt) => {
            *label_idx += 1;
            let current_label_idx = *label_idx;
            let cond_result =
                ast_expression_to_tacky(cond_expr, instructions, symbol_table, label_idx);

            if let Some(else_stmt) = else_stmt {
                instructions.push(Instruction::JumpIfZero(
                    cond_result,
                    format!("else.{}", current_label_idx),
                ));
                ast_statement_to_tacky(then_stmt.as_ref(), instructions, symbol_table, label_idx);
                instructions.push(Instruction::Jump(format!("end.{}", current_label_idx)));
                instructions.push(Instruction::Label(format!("else.{}", current_label_idx)));
                ast_statement_to_tacky(else_stmt.as_ref(), instructions, symbol_table, label_idx);
            } else {
                instructions.push(Instruction::JumpIfZero(
                    cond_result,
                    format!("end.{}", current_label_idx),
                ));
                ast_statement_to_tacky(then_stmt.as_ref(), instructions, symbol_table, label_idx);
            }

            instructions.push(Instruction::Label(format!("end.{}", current_label_idx)));
        }
        parser::Statement::Compound(block) => {
            ast_block_to_tacky(block, instructions, symbol_table, label_idx);
        }
        parser::Statement::Break(identifier) => {
            let jump_label = format!("break.{}", identifier.as_ref().unwrap());
            instructions.push(Instruction::Jump(jump_label))
        }
        parser::Statement::Continue(identifier) => {
            let jump_label = format!("continue.{}", identifier.as_ref().unwrap());
            instructions.push(Instruction::Jump(jump_label))
        }
        parser::Statement::While(cond_expr, body, identifier) => {
            let identifier = identifier.as_ref();
            let continue_label = format!("continue.{}", identifier.unwrap());
            let break_label = format!("break.{}", identifier.unwrap());

            instructions.push(Instruction::Label(continue_label.clone()));
            let cond_result =
                ast_expression_to_tacky(cond_expr, instructions, symbol_table, label_idx);
            instructions.push(Instruction::JumpIfZero(cond_result, break_label.clone()));
            ast_statement_to_tacky(body.as_ref(), instructions, symbol_table, label_idx);
            instructions.push(Instruction::Jump(continue_label));
            instructions.push(Instruction::Label(break_label));
        }
        parser::Statement::DoWhile(body, cond_expr, identifier) => {
            let identifier = identifier.as_ref();

            let start_label = format!("start.{}", identifier.unwrap());
            let continue_label = format!("continue.{}", identifier.unwrap());
            let break_label = format!("break.{}", identifier.unwrap());

            instructions.push(Instruction::Label(start_label.clone()));
            ast_statement_to_tacky(body.as_ref(), instructions, symbol_table, label_idx);
            instructions.push(Instruction::Label(continue_label.clone()));
            let cond_result =
                ast_expression_to_tacky(cond_expr, instructions, symbol_table, label_idx);
            instructions.push(Instruction::JumpIfNotZero(cond_result, start_label.clone()));
            instructions.push(Instruction::Label(break_label.clone()));
        }
        parser::Statement::For(init, cond_expr, post, body, identifier) => {
            let identifier = identifier.as_ref();

            let start_label = format!("start.{}", identifier.unwrap());
            let continue_label = format!("continue.{}", identifier.unwrap());
            let break_label = format!("break.{}", identifier.unwrap());

            ast_for_init_to_tacky(init, instructions, symbol_table, label_idx);
            instructions.push(Instruction::Label(start_label.clone()));
            if let Some(cond_expr) = cond_expr {
                let cond_result =
                    ast_expression_to_tacky(cond_expr, instructions, symbol_table, label_idx);
                instructions.push(Instruction::JumpIfZero(cond_result, break_label.clone()));
            }
            ast_statement_to_tacky(body.as_ref(), instructions, symbol_table, label_idx);
            instructions.push(Instruction::Label(continue_label.clone()));
            if let Some(post) = post {
                ast_expression_to_tacky(post, instructions, symbol_table, label_idx);
            }
            instructions.push(Instruction::Jump(start_label));
            instructions.push(Instruction::Label(break_label));
        }
        parser::Statement::Null => {}
    }
}

pub fn ast_expression_to_tacky(
    ast_expr: &parser::Expr,
    instructions: &mut Vec<Instruction>,
    symbol_table: &mut SymbolTable,
    label_idx: &mut usize,
) -> Val {
    let e_type = &get_type(ast_expr);

    match ast_expr {
        parser::Expr::Constant(i, _) => Val::Constant(i.clone()),
        parser::Expr::Unary(op, expr, _) => {
            let src = ast_expression_to_tacky(expr, instructions, symbol_table, label_idx);
            let dst_name = generate_unique_variable(symbol_table, e_type);
            let dst = Val::Var(dst_name);
            let tacky_op = ast_unop_to_tacky(op);
            instructions.push(Instruction::Unary(tacky_op, src, dst.clone()));
            dst
        }
        parser::Expr::Binary(op, left_expr, right_expr, _) => match op {
            // Short circuit binary operators
            parser::BinaryOperator::And | parser::BinaryOperator::Or => {
                *label_idx += 1;
                let current_label_idx = *label_idx;

                let val_1 =
                    ast_expression_to_tacky(left_expr, instructions, symbol_table, label_idx);
                if let parser::BinaryOperator::And = op {
                    instructions.push(Instruction::JumpIfZero(
                        val_1,
                        format!("false_result.{}", current_label_idx),
                    ));
                } else {
                    instructions.push(Instruction::JumpIfNotZero(
                        val_1,
                        format!("true_result.{}", current_label_idx),
                    ));
                }

                let val_2 =
                    ast_expression_to_tacky(right_expr, instructions, symbol_table, label_idx);
                instructions.push(Instruction::JumpIfZero(
                    val_2,
                    format!("false_result.{}", current_label_idx),
                ));

                let dst_name = generate_unique_variable(symbol_table, e_type);
                let dst = Val::Var(dst_name);

                instructions.push(Instruction::Label(format!(
                    "true_result.{}",
                    current_label_idx
                )));
                instructions.push(Instruction::Copy(
                    Val::Constant(parser::Const::ConstInt(1)),
                    dst.clone(),
                ));
                instructions.push(Instruction::Jump(format!("end.{}", current_label_idx)));
                instructions.push(Instruction::Label(format!(
                    "false_result.{}",
                    current_label_idx
                )));
                instructions.push(Instruction::Copy(
                    Val::Constant(parser::Const::ConstInt(0)),
                    dst.clone(),
                ));
                instructions.push(Instruction::Label(format!("end.{}", current_label_idx)));
                dst
            }
            _ => {
                let val_1 =
                    ast_expression_to_tacky(left_expr, instructions, symbol_table, label_idx);
                let val_2 =
                    ast_expression_to_tacky(right_expr, instructions, symbol_table, label_idx);
                let dst_name = generate_unique_variable(symbol_table, e_type);
                let dst = Val::Var(dst_name);
                let tacky_op = ast_binop_to_tacky(op);
                instructions.push(Instruction::Binary(tacky_op, val_1, val_2, dst.clone()));
                dst
            }
        },
        parser::Expr::Var(identifier, _) => Val::Var(identifier.to_string()),
        parser::Expr::Assignment(left, right, _) => {
            let parser::Expr::Var(v, _) = left.as_ref() else {
                panic!("Expected var on left hand side of assignment.")
            };

            let value = ast_expression_to_tacky(right, instructions, symbol_table, label_idx);
            let dst = Val::Var(v.clone());
            instructions.push(Instruction::Copy(value, dst.clone()));
            dst
        }
        parser::Expr::Conditional(cond_expr, then_expr, else_expr, _) => {
            *label_idx += 1;
            let current_label_idx = *label_idx;
            let dst_name = generate_unique_variable(symbol_table, e_type);
            let dst = Val::Var(dst_name);

            let cond_value =
                ast_expression_to_tacky(cond_expr, instructions, symbol_table, label_idx);
            instructions.push(Instruction::JumpIfZero(
                cond_value,
                format!("else.{}", current_label_idx),
            ));

            let then_value =
                ast_expression_to_tacky(then_expr, instructions, symbol_table, label_idx);
            instructions.push(Instruction::Copy(then_value, dst.clone()));
            instructions.push(Instruction::Jump(format!("end.{}", current_label_idx)));

            instructions.push(Instruction::Label(format!("else.{}", current_label_idx)));
            let else_value =
                ast_expression_to_tacky(else_expr, instructions, symbol_table, label_idx);
            instructions.push(Instruction::Copy(else_value, dst.clone()));

            instructions.push(Instruction::Label(format!("end.{}", current_label_idx)));
            dst
        }
        parser::Expr::FunctionCall(fun_name, args, _) => {
            let mut arg_values = Vec::new();
            for arg in args {
                arg_values.push(ast_expression_to_tacky(
                    arg,
                    instructions,
                    symbol_table,
                    label_idx,
                ))
            }

            let dst_name = generate_unique_variable(symbol_table, e_type);
            let dst = Val::Var(dst_name);
            instructions.push(Instruction::FunCall(
                fun_name.clone(),
                arg_values,
                dst.clone(),
            ));

            dst
        }
        parser::Expr::Cast(t, inner, _) => {
            let result = ast_expression_to_tacky(inner, instructions, symbol_table, label_idx);
            let inner_type = &get_type(inner);

            if t == inner_type {
                return result;
            };

            let dst_name = generate_unique_variable(symbol_table, e_type);
            let dst = Val::Var(dst_name);

            match (t, inner_type) {
                // double (float) conversion
                (Type::Int, Type::Double) => {
                    instructions.push(Instruction::DoubleToInt(result.clone(), dst.clone()))
                }
                (Type::UInt, Type::Double) => {
                    instructions.push(Instruction::DoubleToUInt(result.clone(), dst.clone()))
                }
                (Type::Double, Type::Int) => {
                    instructions.push(Instruction::IntToDouble(result.clone(), dst.clone()))
                }
                (Type::Double, Type::UInt) => {
                    instructions.push(Instruction::UIntToDouble(result.clone(), dst.clone()))
                }
                // integer conversion
                _ => {
                    if t.byte_size() == inner_type.byte_size() {
                        instructions.push(Instruction::Copy(result, dst.clone()));
                    } else if t.byte_size() < inner_type.byte_size() {
                        instructions.push(Instruction::Truncate(result, dst.clone()))
                    } else if inner_type.signed() {
                        instructions.push(Instruction::SignExtend(result, dst.clone()))
                    } else {
                        instructions.push(Instruction::ZeroExtend(result, dst.clone()))
                    }
                }
            }

            dst
        }
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
