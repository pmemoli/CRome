use crate::lexer::Token;
use std::collections::VecDeque;

// program = Program(function_definition)
#[derive(Debug, Clone)]
pub struct Program(pub Function);

// function_definition = Function(identifier name, block body)
#[derive(Debug, Clone)]
pub struct Function(pub String, pub Block);

// block = Block(block_item*)
#[derive(Debug, Clone)]
pub struct Block(pub Vec<BlockItem>);

// block_item = S(statement) | D(declaration)
#[derive(Debug, Clone)]
pub enum BlockItem {
    S(Statement),
    D(Declaration),
}

// declaration = Declaration(identifier name, exp? init)
#[derive(Debug, Clone)]
pub struct Declaration(pub String, pub Option<Expr>);

// for_init = InitDecl(declaration) | InitExp(exp?)
#[derive(Debug, Clone)]
pub enum ForInit {
    InitDecl(Declaration),
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

// <program> ::= <function>
pub fn parse_program(tokens: &mut VecDeque<Token>) -> Program {
    let function = parse_function(tokens);

    if tokens.len() != 0 {
        panic!("Syntax Error: Parsed entire program but some tokens remain");
    }

    Program(function)
}

// <function> ::= "int" <identifier> "(" "void" ")" <block>
pub fn parse_function(tokens: &mut VecDeque<Token>) -> Function {
    expect(Token::IntKeyword, tokens);
    let identifier = parse_identifier(tokens);
    expect(Token::OpenParenthesis, tokens);
    expect(Token::VoidKeyword, tokens);
    expect(Token::CloseParenthesis, tokens);

    let function_body = parse_block(tokens);

    Function(identifier, function_body)
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
        Token::IntKeyword => ForInit::InitDecl(parse_declaration(tokens)),
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
