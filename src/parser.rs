use crate::lexer::Token;

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

struct Program(Function);

struct Function(String, Statement);

enum Statement {
    Return(Expr),
}

enum Expr {
    Constant(i32),
}

fn expect(expected: Token, tokens: &mut Vec<Token>) {}

fn parse_program(tokens: &mut Vec<Token>) -> Program {
    let function = parse_function(tokens);
    Program(function)
}

fn parse_function(tokens: &mut Vec<Token>) -> Function {
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

fn parse_statement(tokens: &mut Vec<Token>) -> Statement {
    expect(Token::ReturnKeyword, tokens);
    let expr = parse_expr(tokens);
    expect(Token::Semicolon, tokens);

    Statement::Return(expr)
}

fn parse_expr(tokens: &mut Vec<Token>) -> Expr {
    let int = parse_int(tokens);
    Expr::Constant(int)
}

fn parse_identifier(tokens: &mut Vec<Token>) -> String {}

fn parse_int(tokens: &mut Vec<Token>) -> i32 {}
