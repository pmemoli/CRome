use crate::lexer::Token;
use std::collections::VecDeque;

// program = Program(function_definition)
#[derive(Debug)]
pub struct Program(pub Function);

// function_definition = Function(identifier name, block_item* body)
#[derive(Debug)]
pub struct Function(pub String, pub Vec<BlockItem>);

// block_item = S(statement) | D(declaration)
#[derive(Debug)]
pub enum BlockItem {
    S(Statement),
    D(Declaration),
}

// declaration = Declaration(identifier name, exp? init)
#[derive(Debug)]
pub struct Declaration(pub String, pub Option<Expr>);

// statement = Return(exp) | Expression(exp) | Null
#[derive(Debug)]
pub enum Statement {
    Return(Expr),
    Expression(Expr),
    Null,
}

// exp = Constant(int)
//     | Var(identifier)
//     | Unary(unary_operator, exp)
//     | Binary(binary_operator, exp, exp)
//     | Assignment(exp, exp)
#[derive(Debug)]
pub enum Expr {
    // factors
    Constant(i32),
    Var(String),
    Unary(UnaryOperator, Box<Expr>),
    // compound expressions
    Binary(BinaryOperator, Box<Expr>, Box<Expr>),
    Assignment(Box<Expr>, Box<Expr>),
}

// unary_operator = Complement | Negate | Not
#[derive(Debug)]
pub enum UnaryOperator {
    Complement,
    Negate,
    Not,
}

// binary_operator = Add | Subtract | Multiply | Divide | Remainder | And | Or
//     | Equal | NotEqual | LessThan | LessOrEqual
//     | GreaterThan | GreaterOrEqual
#[derive(Debug)]
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
    tokens.front().unwrap()
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
    )
}

// <program> ::= <function>
pub fn parse_program(tokens: &mut VecDeque<Token>) -> Program {
    let function = parse_function(tokens);

    if tokens.len() != 0 {
        panic!("Syntax Error: Parsed entire program but some tokens remain");
    }

    Program(function)
}

// <function> ::= "int" <identifier> "(" "void" ")" "{" { <block-item> } "}"
pub fn parse_function(tokens: &mut VecDeque<Token>) -> Function {
    expect(Token::IntKeyword, tokens);
    let identifier = parse_identifier(tokens);
    expect(Token::OpenParenthesis, tokens);
    expect(Token::VoidKeyword, tokens);
    expect(Token::CloseParenthesis, tokens);
    expect(Token::OpenBrace, tokens);

    let mut function_body = Vec::new();
    while !matches!(peek(tokens), Token::CloseBrace) {
        let next_block_item = parse_block_item(tokens);
        function_body.push(next_block_item);
    }
    take_token(tokens);

    Function(identifier, function_body)
}

// <block-item> ::= <statement> | <declaration>
pub fn parse_block_item(tokens: &mut VecDeque<Token>) -> BlockItem {
    match peek(tokens) {
        Token::IntKeyword => BlockItem::D(parse_declaration(tokens)),
        _ => BlockItem::S(parse_statement(tokens)),
    }
}

// <declaration> ::= "int" <identifier> [ "=" <exp> ] ";"
pub fn parse_declaration(tokens: &mut VecDeque<Token>) -> Declaration {
    expect(Token::IntKeyword, tokens);
    let identifier = parse_identifier(tokens);
    let init = if matches!(peek(tokens), Token::Equal) {
        take_token(tokens);
        Some(parse_expr(tokens, 0))
    } else {
        None
    };
    expect(Token::Semicolon, tokens);

    Declaration(identifier, init)
}

// <statement> ::= "return" <exp> ";" | <exp> ";" | ";"
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
        _ => {
            let expr = parse_expr(tokens, 0);
            expect(Token::Semicolon, tokens);
            Statement::Expression(expr)
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
        } else {
            let operator = parse_binop(tokens);
            let right_expr = parse_expr(tokens, precedence(&next_token) + 1);
            left_expr = Expr::Binary(operator, Box::new(left_expr), Box::new(right_expr));
        }
        next_token = peek(tokens).clone();
    }
    left_expr
}

// <factor> ::= <int> | <identifier> | <unop> <factor> | "(" <exp> ")"
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
        Token::Identifier(s) => {
            let expr = Expr::Var(s.to_string());
            take_token(tokens);
            expr
        }
        _ => panic!("Malformed Expression"),
    }
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
