use crate::lexer::Token;
use std::collections::VecDeque;

// ASDL Grammar
// program = Program(function_definition)
// function_definition = Function(identifier name, statement body)
// statement = Return(exp)
// exp = Constant(int)

// Formal Grammar
// <program> ::= <function>
// <function> ::= "int" <identifier> "(" "void" ")" "{" <statement> "}"
// <statement> ::= "return" <exp> ";"
// <exp> ::= <int>
// <identifier> ::= ? An identifier token ?
// <int> ::= ? A constant token ?

#[derive(Debug)]
pub struct Program(Function);

#[derive(Debug)]
pub struct Function(String, Statement);

#[derive(Debug)]
pub enum Statement {
    Return(Expr),
}

#[derive(Debug)]
pub enum Expr {
    Constant(i32),
}

fn expect(expected: Token, tokens: &mut VecDeque<Token>) {
    let actual = tokens.pop_front().unwrap();
    if actual != expected {
        panic!(
            "Syntax Error: Expected {:?} but found {:?}",
            expected, actual
        );
    }
}

pub fn parse_program(tokens: &mut VecDeque<Token>) -> Program {
    let function = parse_function(tokens);
    Program(function)
}

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

pub fn parse_statement(tokens: &mut VecDeque<Token>) -> Statement {
    expect(Token::ReturnKeyword, tokens);
    let expr = parse_expr(tokens);
    expect(Token::Semicolon, tokens);

    Statement::Return(expr)
}

pub fn parse_expr(tokens: &mut VecDeque<Token>) -> Expr {
    let int = parse_int(tokens);
    Expr::Constant(int)
}

pub fn parse_identifier(tokens: &mut VecDeque<Token>) -> String {
    let actual = tokens.pop_front().unwrap();
    let Token::Identifier(s) = actual else {
        panic!("Syntax Error: Can't parse {:?} as a string", actual);
    };

    s.to_string()
}

pub fn parse_int(tokens: &mut VecDeque<Token>) -> i32 {
    let actual = tokens.pop_front().unwrap();
    let Token::Constant(s) = actual else {
        panic!("Syntax Error: Can't parse {:?} as an integer", actual);
    };

    s
}
