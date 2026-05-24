use crate::{parser, symbol::SymbolTable};

mod identifier_resolution;
mod loop_annotation;
mod type_checking;

// Wrapper for semantic analysis passes
pub fn semantic_analysis(ast: &parser::Program, symbol_table: &mut SymbolTable) -> parser::Program {
    let resolved_variable_ast = identifier_resolution::resolve_program(ast);
    let loop_labeled_ast = loop_annotation::label_program(&resolved_variable_ast);
    let type_checked_ast = type_checking::typecheck_program(&loop_labeled_ast, symbol_table);

    type_checked_ast
}
