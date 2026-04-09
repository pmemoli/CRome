use crate::lexer::Token;
use std::collections::VecDeque;

// program = Program(function_declaration*)
#[derive(Debug, Clone)]
pub struct Program(pub Vec<FunctionDeclaration>);

// declaration = FunDecl(function_declaration) | VarDecl(variable_declaration)
#[derive(Debug, Clone)]
pub enum Declaration {
    FunDecl(FunctionDeclaration),
    VarDecl(VariableDeclaration),
}

// function_declaration = (identifier name, identifier* params, block? body)
#[derive(Debug, Clone)]
pub struct FunctionDeclaration(pub String, pub Vec<String>, pub Option<Block>);

// variable_declaration = (identifier name, exp? init)
#[derive(Debug, Clone)]
pub struct VariableDeclaration(pub String, pub Option<Expr>);

// block = Block(block_item*)
#[derive(Debug, Clone)]
pub struct Block(pub Vec<BlockItem>);

// block_item = S(statement) | D(declaration)
#[derive(Debug, Clone)]
pub enum BlockItem {
    S(Statement),
    D(Declaration),
}

// for_init = InitDecl(variable_declaration) | InitExp(exp?)
#[derive(Debug, Clone)]
pub enum ForInit {
    InitDecl(VariableDeclaration),
    InitExp(Option<Expr>),
}

// statement = Return(exp)
//     | Expression(exp)
//     | If(exp condition, statement then, statement? else)
//     | Compound(block)
//     | Break(identifier label)
//     | Continue(identifier label)
//     | While(exp condition, statement body, identifier label)
//     | DoWhile(statement body, exp condition, identifier label)
//     | For(for_init init, exp? condition, exp? post, statement body, identifier label)
//     | Null
#[derive(Debug, Clone)]
pub enum Statement {
    Return(Expr),
    Expression(Expr),
    If(Expr, Box<Statement>, Option<Box<Statement>>),
    Compound(Block),
    Null,

    // Loop statements
    Break(Option<String>),
    Continue(Option<String>),
    While(Expr, Box<Statement>, Option<String>),
    DoWhile(Box<Statement>, Expr, Option<String>),
    For(
        ForInit,
        Option<Expr>,
        Option<Expr>,
        Box<Statement>,
        Option<String>,
    ),
}

// exp = Constant(int)
//     | Var(identifier)
//     | Unary(unary_operator, exp)
//     | Binary(binary_operator, exp, exp)
//     | Assignment(exp, exp)
//     | Conditional(exp condition, exp, exp)
//     | FunctionCall(identifier, exp* args)
#[derive(Debug, Clone)]
pub enum Expr {
    // factors
    Constant(i32),
    Var(String),
    Unary(UnaryOperator, Box<Expr>),

    // compound expressions
    Binary(BinaryOperator, Box<Expr>, Box<Expr>),
    Assignment(Box<Expr>, Box<Expr>),
    Conditional(Box<Expr>, Box<Expr>, Box<Expr>),
    FunctionCall(String, Vec<Expr>),
}

// unary_operator = Complement | Negate | Not
#[derive(Debug, Clone)]
pub enum UnaryOperator {
    Complement,
    Negate,
    Not,
}

// binary_operator = Add | Subtract | Multiply | Divide | Remainder | And | Or
//     | Equal | NotEqual | LessThan | LessOrEqual
//     | GreaterThan | GreaterOrEqual
#[derive(Debug, Clone)]
pub enum BinaryOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
    Remainder,
    And,
    Or,
    Equal,
    NotEqual,
    LessThan,
    LessThanOrEqual,
    GreaterThan,
    GreaterThanOrEqual,
}

// utils
fn expect(expected: Token, tokens: &mut VecDeque<Token>) {
    let actual = tokens.pop_front().unwrap();
    if actual != expected {
        panic!(
            "Syntax Error: Expected {:?} but found {:?}",
            expected, actual
        );
    }
}

fn peek(tokens: &VecDeque<Token>) -> &Token {
    &tokens[0]
}

fn peek_n(tokens: &VecDeque<Token>, n: usize) -> &Token {
    &tokens[n]
}

fn take_token(tokens: &mut VecDeque<Token>) -> Token {
    tokens.pop_front().unwrap()
}

fn precedence(operator: &Token) -> i32 {
    match operator {
        Token::TwoVerticalBar => 5,
        Token::TwoAmpersand => 10,
        Token::TwoEqual | Token::NotEqual => 30,
        Token::LessThan
        | Token::LessThanOrEqual
        | Token::GreaterThan
        | Token::GreaterThanOrEqual => 35,
        Token::Plus | Token::Hyphen => 45,
        Token::Asterisk | Token::ForwardSlash | Token::Percent => 50,
        Token::QuestionMark => 3,
        Token::Equal => 1,
        _ => panic!(
            "Syntax Error: Expected a binary operator but found {:?}",
            operator
        ),
    }
}

fn is_binop(token: &Token) -> bool {
    matches!(
        token,
        Token::Plus
            | Token::Hyphen
            | Token::Asterisk
            | Token::ForwardSlash
            | Token::Percent
            | Token::TwoAmpersand
            | Token::TwoVerticalBar
            | Token::TwoEqual
            | Token::NotEqual
            | Token::LessThan
            | Token::LessThanOrEqual
            | Token::GreaterThan
            | Token::GreaterThanOrEqual
            | Token::Equal
            | Token::QuestionMark
    )
}

// <program> ::= { <function-declaration> }
pub fn parse_program(tokens: &mut VecDeque<Token>) -> Program {
    let mut declarations = Vec::new();
    while !tokens.is_empty() {
        let func_declaration = parse_function_declaration(tokens);
        declarations.push(func_declaration);
    }

    if tokens.len() != 0 {
        panic!("Syntax Error: Parsed entire program but some tokens remain");
    }

    Program(declarations)
}

// <declaration> ::= <variable-declaration> | <function-declaration>
pub fn parse_declaration(tokens: &mut VecDeque<Token>) -> Declaration {
    match peek_n(tokens, 2) {
        Token::Equal | Token::Semicolon => {
            let declaration = parse_variable_declaration(tokens);
            Declaration::VarDecl(declaration)
        }
        Token::OpenParenthesis => {
            let declaration = parse_function_declaration(tokens);
            Declaration::FunDecl(declaration)
        }
        _ => panic!("Expected =, ; or ( after declaration"),
    }
}

// <variable-declaration> ::= "int" <identifier> [ "=" <exp> ] ";"
pub fn parse_variable_declaration(tokens: &mut VecDeque<Token>) -> VariableDeclaration {
    expect(Token::IntKeyword, tokens);
    let identifier = parse_identifier(tokens);
    let init = if matches!(peek(tokens), Token::Equal) {
        take_token(tokens);
        Some(parse_expr(tokens, 0))
    } else {
        None
    };
    expect(Token::Semicolon, tokens);

    VariableDeclaration(identifier, init)
}

// <function-declaration> ::= "int" <identifier> "(" <param-list> ")" ( <block> | ";")
pub fn parse_function_declaration(tokens: &mut VecDeque<Token>) -> FunctionDeclaration {
    expect(Token::IntKeyword, tokens);
    let identifier = parse_identifier(tokens);
    expect(Token::OpenParenthesis, tokens);
    let param_list = parse_param_list(tokens);
    expect(Token::CloseParenthesis, tokens);

    match peek(tokens) {
        Token::OpenBrace => {
            let block = parse_block(tokens);
            FunctionDeclaration(identifier, param_list, Some(block))
        }
        Token::Semicolon => {
            take_token(tokens);
            FunctionDeclaration(identifier, param_list, None)
        }
        _ => panic!("Expected semicolon or definition after function definition"),
    }
}

// <param-list> ::= eps | "void" | "int" <identifier> { "," "int" <identifier> }
pub fn parse_param_list(tokens: &mut VecDeque<Token>) -> Vec<String> {
    let mut param_list = Vec::new();
    match peek(tokens) {
        Token::IntKeyword => {
            take_token(tokens);
            param_list.push(parse_identifier(tokens));

            while matches!(peek(tokens), Token::Comma) {
                take_token(tokens);
                expect(Token::IntKeyword, tokens);
                param_list.push(parse_identifier(tokens));
            }
        }
        Token::VoidKeyword => {
            take_token(tokens);
        }
        Token::CloseParenthesis => {}
        _ => panic!("Expected 'void' or parameter list in function declaration"),
    }

    param_list
}

// <block> ::= "{" { <block-item> } "}"
pub fn parse_block(tokens: &mut VecDeque<Token>) -> Block {
    expect(Token::OpenBrace, tokens);

    let mut block_items = Vec::new();
    while !matches!(peek(tokens), Token::CloseBrace) {
        let next_block_item = parse_block_item(tokens);
        block_items.push(next_block_item);
    }
    expect(Token::CloseBrace, tokens);

    Block(block_items)
}

// <block-item> ::= <statement> | <declaration>
pub fn parse_block_item(tokens: &mut VecDeque<Token>) -> BlockItem {
    match peek(tokens) {
        Token::IntKeyword => BlockItem::D(parse_declaration(tokens)),
        _ => BlockItem::S(parse_statement(tokens)),
    }
}

// <statement> ::= "return" <exp> ";"
//     | <exp> ";" done
//     | "if" "(" <exp> ")" <statement> [ "else" <statement> ] done
//     | <block> done
//     | "break" ";" done
//     | "continue" ";" done
//     | "while" "(" <exp> ")" <statement>
//     | "do" <statement> "while" "(" <exp> ")" ";"
//     | "for" "(" <for-init> [ <exp> ] ";" [ <exp> ] ")" <statement>
//     | ";"
pub fn parse_statement(tokens: &mut VecDeque<Token>) -> Statement {
    match peek(tokens) {
        Token::ReturnKeyword => {
            take_token(tokens);
            let expr = parse_expr(tokens, 0);
            expect(Token::Semicolon, tokens);
            Statement::Return(expr)
        }
        Token::Semicolon => {
            take_token(tokens);
            Statement::Null
        }
        Token::IfKeyword => {
            take_token(tokens);

            expect(Token::OpenParenthesis, tokens);
            let condition = parse_expr(tokens, 0);
            expect(Token::CloseParenthesis, tokens);

            let then_branch = Box::new(parse_statement(tokens));
            let else_branch = if matches!(peek(tokens), Token::ElseKeyword) {
                take_token(tokens);
                Some(Box::new(parse_statement(tokens)))
            } else {
                None
            };

            Statement::If(condition, then_branch, else_branch)
        }
        Token::BreakKeyword => {
            take_token(tokens);
            expect(Token::Semicolon, tokens);
            Statement::Break(None)
        }
        Token::ContinueKeyword => {
            take_token(tokens);
            expect(Token::Semicolon, tokens);
            Statement::Continue(None)
        }
        Token::WhileKeyword => {
            take_token(tokens);
            expect(Token::OpenParenthesis, tokens);
            let condition_expr = parse_expr(tokens, 0);
            expect(Token::CloseParenthesis, tokens);
            let body_stmt = Box::new(parse_statement(tokens));
            Statement::While(condition_expr, body_stmt, None)
        }
        Token::DoKeyword => {
            take_token(tokens);
            let body_stmt = Box::new(parse_statement(tokens));
            expect(Token::WhileKeyword, tokens);
            expect(Token::OpenParenthesis, tokens);
            let condition_expr = parse_expr(tokens, 0);
            expect(Token::CloseParenthesis, tokens);
            expect(Token::Semicolon, tokens);
            Statement::DoWhile(body_stmt, condition_expr, None)
        }
        Token::ForKeyword => {
            take_token(tokens);
            expect(Token::OpenParenthesis, tokens);
            let init = parse_for_init(tokens);
            let condition = parse_optional_expr(tokens, 0, Token::Semicolon);
            expect(Token::Semicolon, tokens);
            let post = parse_optional_expr(tokens, 0, Token::CloseParenthesis);
            expect(Token::CloseParenthesis, tokens);
            let body_stmt = Box::new(parse_statement(tokens));
            Statement::For(init, condition, post, body_stmt, None)
        }
        Token::OpenBrace => Statement::Compound(parse_block(tokens)),
        _ => {
            let expr = parse_expr(tokens, 0);
            expect(Token::Semicolon, tokens);
            Statement::Expression(expr)
        }
    }
}

// <for-init> ::= <declaration> | [ <exp> ] ";"
pub fn parse_for_init(tokens: &mut VecDeque<Token>) -> ForInit {
    match peek(tokens) {
        Token::IntKeyword => ForInit::InitDecl(parse_variable_declaration(tokens)),
        _ => {
            let init_expr = parse_optional_expr(tokens, 0, Token::Semicolon);
            expect(Token::Semicolon, tokens);
            ForInit::InitExp(init_expr)
        }
    }
}

// <exp> ::= <factor> | <exp> <binop> <exp>
pub fn parse_expr(tokens: &mut VecDeque<Token>, min_prec: i32) -> Expr {
    let mut left_expr = parse_factor(tokens);
    let mut next_token = peek(tokens).clone();
    while is_binop(&next_token) && precedence(&next_token) >= min_prec {
        if next_token == Token::Equal {
            take_token(tokens);
            let right_expr = parse_expr(tokens, precedence(&next_token));
            left_expr = Expr::Assignment(Box::new(left_expr), Box::new(right_expr));
        } else if next_token == Token::QuestionMark {
            take_token(tokens);
            let middle_expr = parse_expr(tokens, 0);
            expect(Token::Colon, tokens);
            let right_expr = parse_expr(tokens, precedence(&next_token));
            left_expr = Expr::Conditional(
                Box::new(left_expr),
                Box::new(middle_expr),
                Box::new(right_expr),
            );
        } else {
            let operator = parse_binop(tokens);
            let right_expr = parse_expr(tokens, precedence(&next_token) + 1);
            left_expr = Expr::Binary(operator, Box::new(left_expr), Box::new(right_expr));
        }
        next_token = peek(tokens).clone();
    }
    left_expr
}

pub fn parse_optional_expr(
    tokens: &mut VecDeque<Token>,
    min_prec: i32,
    end_token: Token,
) -> Option<Expr> {
    if peek(tokens) == &end_token {
        None
    } else {
        Some(parse_expr(tokens, min_prec))
    }
}

// <factor> ::= <int> | <identifier> | <unop> <factor> | "(" <exp> ")"
//     | <identifier> "(" [ <argument-list> ] ")"
pub fn parse_factor(tokens: &mut VecDeque<Token>) -> Expr {
    match peek(tokens) {
        Token::Constant(i) => {
            let expr = Expr::Constant(*i);
            take_token(tokens);
            expr
        }
        Token::Hyphen | Token::Tilde | Token::Exclamation => {
            let operator = parse_unop(tokens);
            let inner_expr = parse_factor(tokens);
            Expr::Unary(operator, Box::new(inner_expr))
        }
        Token::OpenParenthesis => {
            take_token(tokens);
            let inner_expr = parse_expr(tokens, 0);
            expect(Token::CloseParenthesis, tokens);
            inner_expr
        }
        Token::Identifier(s) => match peek_n(tokens, 1) {
            Token::OpenParenthesis => {
                let func_name = s.to_string();
                take_token(tokens);
                take_token(tokens);
                let arguments = parse_argument_list(tokens);
                expect(Token::CloseParenthesis, tokens);
                Expr::FunctionCall(func_name, arguments)
            }
            _ => {
                let expr = Expr::Var(s.to_string());
                take_token(tokens);
                expr
            }
        },
        _ => panic!("Malformed Expression"),
    }
}

// <argument-list> ::= <exp> { "," <exp> }
pub fn parse_argument_list(tokens: &mut VecDeque<Token>) -> Vec<Expr> {
    let mut arguments = Vec::new();
    if matches!(peek(tokens), Token::CloseParenthesis) {
        return arguments;
    }

    arguments.push(parse_expr(tokens, 0));

    while matches!(peek(tokens), Token::Comma) {
        take_token(tokens);
        arguments.push(parse_expr(tokens, 0));
    }

    arguments
}

// <unop> ::= "-" | "~" | "!"
pub fn parse_unop(tokens: &mut VecDeque<Token>) -> UnaryOperator {
    match peek(tokens) {
        Token::Hyphen => {
            take_token(tokens);
            UnaryOperator::Negate
        }
        Token::Tilde => {
            take_token(tokens);
            UnaryOperator::Complement
        }
        Token::Exclamation => {
            take_token(tokens);
            UnaryOperator::Not
        }
        _ => panic!("Malformed Expression"),
    }
}

// <binop> ::= "-" | "+" | "*" | "/" | "%" | "&&" | "||"
//     | "==" | "!=" | "<" | "<=" | ">" | ">="
pub fn parse_binop(tokens: &mut VecDeque<Token>) -> BinaryOperator {
    match take_token(tokens) {
        Token::Hyphen => BinaryOperator::Subtract,
        Token::Plus => BinaryOperator::Add,
        Token::Asterisk => BinaryOperator::Multiply,
        Token::ForwardSlash => BinaryOperator::Divide,
        Token::Percent => BinaryOperator::Remainder,
        Token::TwoAmpersand => BinaryOperator::And,
        Token::TwoVerticalBar => BinaryOperator::Or,
        Token::TwoEqual => BinaryOperator::Equal,
        Token::NotEqual => BinaryOperator::NotEqual,
        Token::LessThan => BinaryOperator::LessThan,
        Token::LessThanOrEqual => BinaryOperator::LessThanOrEqual,
        Token::GreaterThan => BinaryOperator::GreaterThan,
        Token::GreaterThanOrEqual => BinaryOperator::GreaterThanOrEqual,
        _ => panic!("Malformed Expression"),
    }
}

// <identifier> ::= ? An identifier token ?
pub fn parse_identifier(tokens: &mut VecDeque<Token>) -> String {
    let actual = tokens.pop_front().unwrap();
    let Token::Identifier(s) = actual else {
        panic!("Syntax Error: Can't parse {:?} as a string", actual);
    };

    s.to_string()
}

// <int> ::= ? A constant token ? (a separate function is currently not needed...)
// pub fn parse_int(tokens: &mut VecDeque<Token>) -> i32 {
//     let actual = tokens.pop_front().unwrap();
//     let Token::Constant(s) = actual else {
//         panic!("Syntax Error: Can't parse {:?} as an integer", actual);
//     };
//
//     s
// }
