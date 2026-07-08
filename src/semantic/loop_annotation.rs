use super::*;

pub fn label_program(program: &parser::Program) -> parser::Program {
    let mut loop_idx = 0;
    let current_label = None;

    let parser::Program(declarations) = program;
    let mut new_declarations = Vec::new();
    for declaration in declarations {
        new_declarations.push(label_declaration(
            declaration,
            &mut loop_idx,
            &current_label,
        ));
    }

    parser::Program(new_declarations)
}

pub fn label_declaration(
    declaration: &parser::Declaration,
    loop_idx: &mut usize,
    current_label: &Option<String>,
) -> parser::Declaration {
    match declaration {
        parser::Declaration::VarDecl(var_decl) => parser::Declaration::VarDecl(var_decl.clone()),
        parser::Declaration::FunDecl(func_decl) => parser::Declaration::FunDecl(
            label_function_declaration(func_decl, loop_idx, current_label),
        ),
    }
}

pub fn label_function_declaration(
    function_declaration: &parser::FunctionDeclaration,
    loop_idx: &mut usize,
    current_label: &Option<String>,
) -> parser::FunctionDeclaration {
    let parser::FunctionDeclaration(identifier, parameters, body, ty, storage_class) =
        function_declaration;
    if let Some(block) = body.as_ref() {
        let new_block = label_block(block, loop_idx, current_label);
        parser::FunctionDeclaration(
            identifier.clone(),
            parameters.clone(),
            Some(new_block),
            ty.clone(),
            storage_class.clone(),
        )
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
