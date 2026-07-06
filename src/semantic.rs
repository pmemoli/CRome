use crate::{parser, symbol::SymbolTable, types::Type};

mod identifier_resolution;
mod loop_annotation;
mod type_checking;

pub fn semantic_analysis(ast: &parser::Program, symbol_table: &mut SymbolTable) -> parser::Program {
    let resolved_variable_ast = identifier_resolution::resolve_program(ast);
    let loop_labeled_ast = loop_annotation::label_program(&resolved_variable_ast);
    let type_checked_ast = type_checking::typecheck_program(&loop_labeled_ast, symbol_table);

    type_checked_ast
}

pub fn get_type(expr: &parser::Expr) -> Type {
    match expr {
        parser::Expr::FunctionCall(_, _, Some(ty))
        | parser::Expr::Var(_, Some(ty))
        | parser::Expr::Assignment(_, _, Some(ty))
        | parser::Expr::Unary(_, _, Some(ty))
        | parser::Expr::Binary(_, _, _, Some(ty))
        | parser::Expr::Conditional(_, _, _, Some(ty))
        | parser::Expr::Cast(_, _, Some(ty))
        | parser::Expr::AddressOf(_, Some(ty))
        | parser::Expr::Dereference(_, Some(ty))
        | parser::Expr::Constant(_, Some(ty)) => ty.clone(),
        _ => panic!("Expression without type annotation"),
    }
}
