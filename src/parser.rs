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

// exp = Constant(int) | Unary(unary_operator, exp)
#[derive(Debug)]
pub enum Expr {
    Constant(i32),
    Unary(UnaryOperator, Box<Expr>),
}

// unary_operator = Complement | Negate
#[derive(Debug)]
pub enum UnaryOperator {
    Complement,
    Negate,
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
    let expr = parse_expr(tokens);
    expect(Token::Semicolon, tokens);

    Statement::Return(expr)
}

// <exp> ::= <int> | <unop> <exp> | "(" <exp> ")"
pub fn parse_expr(tokens: &mut VecDeque<Token>) -> Expr {
    match peek(tokens) {
        Token::Constant(i) => {
            let expr = Expr::Constant(*i);
            take_token(tokens);
            expr
        }
        Token::Hyphen | Token::Tilde => {
            let operator = parse_unop(tokens);
            let inner_expr = parse_expr(tokens);
            Expr::Unary(operator, Box::new(inner_expr))
        }
        Token::OpenParenthesis => {
            take_token(tokens);
            let inner_expr = parse_expr(tokens);
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
