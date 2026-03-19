use crate::lexer::Token;
use std::collections::VecDeque;

// program = Program(function_definition)
#[derive(Debug)]
pub struct Program(pub Function);

// function_definition = Function(identifier name, statement body)
#[derive(Debug)]
pub struct Function(pub String, pub Statement);

// statement = Return(exp)
#[derive(Debug)]
pub enum Statement {
    Return(Expr),
}

// exp = Constant(int)
//     | Unary(unary_operator, exp)
//     | Binary(binary_operator, exp, exp)
#[derive(Debug)]
pub enum Expr {
    Constant(i32),
    Unary(UnaryOperator, Box<Expr>),
    Binary(BinaryOperator, Box<Expr>, Box<Expr>),
}

// unary_operator = Complement | Negate
#[derive(Debug)]
pub enum UnaryOperator {
    Complement,
    Negate,
}

// binary_operator = Add | Subtract | Multiply | Divide | Remainder
#[derive(Debug)]
pub enum BinaryOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
    Remainder,
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
        Token::Plus | Token::Hyphen => 45,
        Token::Asterisk | Token::ForwardSlash | Token::Percent => 50,
        _ => panic!(
            "Syntax Error: Expected a binary operator but found {:?}",
            operator
        ),
    }
}

fn is_binop(token: &Token) -> bool {
    matches!(
        token,
        Token::Plus | Token::Hyphen | Token::Asterisk | Token::ForwardSlash | Token::Percent
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

// <function> ::= "int" <identifier> "(" "void" ")" "{" <statement> "}"
pub fn parse_function(tokens: &mut VecDeque<Token>) -> Function {
    expect(Token::IntKeyword, tokens);
    let identifier = parse_identifier(tokens);
    expect(Token::OpenParenthesis, tokens);
    expect(Token::VoidKeyword, tokens);
    expect(Token::CloseParenthesis, tokens);
    expect(Token::OpenBrace, tokens);
    let statement = parse_statement(tokens);
    expect(Token::CloseBrace, tokens);

    Function(identifier, statement)
}

// <statement> ::= "return" <exp> ";"
pub fn parse_statement(tokens: &mut VecDeque<Token>) -> Statement {
    expect(Token::ReturnKeyword, tokens);
    let expr = parse_expr(tokens, 0);
    expect(Token::Semicolon, tokens);

    Statement::Return(expr)
}

// <exp> ::= <factor> | <exp> <binop> <exp>
pub fn parse_expr(tokens: &mut VecDeque<Token>, min_prec: i32) -> Expr {
    // We implement left associativity
    let mut left_expr = parse_factor(tokens);
    let mut next_token = peek(tokens).clone();
    while is_binop(&next_token) && precedence(&next_token) >= min_prec {
        let operator = parse_binop(tokens);
        let right_expr = parse_expr(tokens, precedence(&next_token) + 1);
        left_expr = Expr::Binary(operator, Box::new(left_expr), Box::new(right_expr));
        next_token = peek(tokens).clone();
    }
    left_expr
}

// <factor> ::= <int> | <unop> <factor> | "(" <exp> ")"
pub fn parse_factor(tokens: &mut VecDeque<Token>) -> Expr {
    match peek(tokens) {
        Token::Constant(i) => {
            let expr = Expr::Constant(*i);
            take_token(tokens);
            expr
        }
        Token::Hyphen | Token::Tilde => {
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
        _ => panic!("Malformed Expression"),
    }
}

// <unop> ::= "-" | "~"
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
        _ => panic!("Malformed Expression"),
    }
}

// <binop> ::= "-" | "+" | "*" | "/" | "%"
pub fn parse_binop(tokens: &mut VecDeque<Token>) -> BinaryOperator {
    match take_token(tokens) {
        Token::Hyphen => BinaryOperator::Subtract,
        Token::Plus => BinaryOperator::Add,
        Token::Asterisk => BinaryOperator::Multiply,
        Token::ForwardSlash => BinaryOperator::Divide,
        Token::Percent => BinaryOperator::Remainder,
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
