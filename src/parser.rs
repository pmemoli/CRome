use ordered_float::OrderedFloat;

use crate::lexer::Token;
use crate::types::Type;
use std::{
    collections::{HashMap, VecDeque},
    panic,
};

// program = Program(declaration*)
#[derive(Debug, Clone, PartialEq)]
pub struct Program(pub Vec<Declaration>);

// declaration = FunDecl(function_declaration) | VarDecl(variable_declaration)
#[derive(Debug, Clone, PartialEq)]
pub enum Declaration {
    FunDecl(FunctionDeclaration),
    VarDecl(VariableDeclaration),
}

// function_declaration = (identifier name, identifier* params, block? body, type fun_type, storage_class?)
#[derive(Debug, Clone, PartialEq)]
pub struct FunctionDeclaration(
    pub String,
    pub Vec<String>,
    pub Option<Block>,
    pub Type,
    pub Option<StorageClass>,
);

// variable_declaration = (identifier name, exp? init, type var_type, storage_class?)
#[derive(Debug, Clone, PartialEq)]
pub struct VariableDeclaration(
    pub String,
    pub Option<Expr>,
    pub Type,
    pub Option<StorageClass>,
);

// storage_class = Static | Extern
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StorageClass {
    Static,
    Extern,
}

// block = Block(block_item*)
#[derive(Debug, Clone, PartialEq)]
pub struct Block(pub Vec<BlockItem>);

// block_item = S(statement) | D(declaration)
#[derive(Debug, Clone, PartialEq)]
pub enum BlockItem {
    S(Statement),
    D(Declaration),
}

// for_init = InitDecl(variable_declaration) | InitExp(exp?)
#[derive(Debug, Clone, PartialEq)]
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
#[derive(Debug, Clone, PartialEq)]
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

// exp = Constant(const)
//     | Var(identifier)
//     | Cast(type target_type, exp)
//     | Unary(unary_operator, exp)
//     | Binary(binary_operator, exp, exp)
//     | Assignment(exp, exp)
//     | Conditional(exp condition, exp, exp)
//     | FunctionCall(identifier, exp* args)
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    // factors
    Constant(Const, Option<Type>),
    Var(String, Option<Type>),
    Unary(UnaryOperator, Box<Expr>, Option<Type>),
    Cast(Type, Box<Expr>, Option<Type>),

    // compound expressions
    Binary(BinaryOperator, Box<Expr>, Box<Expr>, Option<Type>),
    Assignment(Box<Expr>, Box<Expr>, Option<Type>),
    Conditional(Box<Expr>, Box<Expr>, Box<Expr>, Option<Type>),
    FunctionCall(String, Vec<Expr>, Option<Type>),
    Dereference(Box<Expr>, Option<Type>), // *
    AddressOf(Box<Expr>, Option<Type>),   // &
}

impl Expr {
    pub fn is_lvalue(&self) -> bool {
        matches!(self, Expr::Var(_, _) | Expr::Dereference(_, _))
    }
}

// unary_operator = Complement | Negate | Not
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UnaryOperator {
    Complement,
    Negate,
    Not,
}

// binary_operator = Add | Subtract | Multiply | Divide | Remainder | And | Or
//     | Equal | NotEqual | LessThan | LessOrEqual
//     | GreaterThan | GreaterOrEqual
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BinaryOperator {
    // Arithmetic
    Add,
    Subtract,
    Multiply,
    Divide,
    Remainder,

    // Logical
    And,
    Or,

    // Comparison
    Equal,
    NotEqual,
    LessThan,
    LessThanOrEqual,
    GreaterThan,
    GreaterThanOrEqual,
}

// const = ConstInt(int) | ConstLong(int)
//     | ConstUInt(int) | ConstULong(int)
//     | ConstDouble(double) | ConstFloat(float)
#[derive(Debug, Clone, PartialEq)]
pub enum Const {
    ConstInt(i32),
    ConstUInt(u32),
    ConstLong(i64),
    ConstULong(u64),
    ConstFloat(f32),
    ConstDouble(f64),
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

// Token utilities for pattern matching
impl Token {
    pub fn is_binop(&self) -> bool {
        matches!(
            self,
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

    pub fn is_constant_token(&self) -> bool {
        matches!(
            self,
            Token::Constant(_)
                | Token::LongConstant(_)
                | Token::UConstant(_)
                | Token::ULongConstant(_)
                | Token::DFloatConstant(_)
                | Token::SFloatConstant(_)
        )
    }

    pub fn is_type_specifier(&self) -> bool {
        matches!(
            self,
            Token::IntKeyword
                | Token::LongKeyword
                | Token::UnsignedKeyword
                | Token::SignedKeyword
                | Token::FloatKeyword
                | Token::DoubleKeyword
        )
    }

    pub fn is_storage_class_specifier(&self) -> bool {
        matches!(self, Token::Static | Token::Extern)
    }

    pub fn is_specifier(&self) -> bool {
        self.is_type_specifier() || self.is_storage_class_specifier()
    }

    pub fn precedence(&self) -> i32 {
        match self {
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
                self
            ),
        }
    }
}

// <program> ::= { <declaration> }
pub fn parse_program(tokens: &mut VecDeque<Token>) -> Program {
    let mut declarations = Vec::new();
    while !tokens.is_empty() {
        let declaration = parse_declaration(tokens);
        declarations.push(declaration);
    }

    if tokens.len() != 0 {
        panic!("Syntax Error: Parsed entire program but some tokens remain");
    }

    Program(declarations)
}

// <declaration> ::= <variable-declaration> | <function-declaration>
// <function-declaration> ::= { <specifier> }+ <declarator> ( <block> | ";")
// <variable-declaration> ::= { <specifier> }+ <declarator> [ "=" <exp> ] ";"
pub fn parse_declaration(tokens: &mut VecDeque<Token>) -> Declaration {
    let (ty, storage_class) = parse_type_and_storage_class(tokens);
    let declarator = parse_declarator(tokens);
    let (name, derived_type, param_names) = process_declarator(declarator, ty);

    match derived_type {
        Type::FunType(_, _) => {
            let body = match peek(tokens) {
                Token::OpenBrace => Some(parse_block(tokens)),
                Token::Semicolon => {
                    take_token(tokens);
                    None
                }
                _ => panic!(
                    "Syntax Error: Expected function body or ';' but found {:?}",
                    peek(tokens)
                ),
            };

            Declaration::FunDecl(FunctionDeclaration(
                name,
                param_names,
                body,
                derived_type.clone(),
                storage_class,
            ))
        }
        _ => {
            let init_expr = match peek(tokens) {
                Token::Semicolon => None,
                Token::Equal => {
                    take_token(tokens);
                    Some(parse_expr(tokens, 0))
                }
                _ => panic!(
                    "Syntax Error: Expected '=' or ';' but found {:?}",
                    peek(tokens)
                ),
            };
            expect(Token::Semicolon, tokens);

            Declaration::VarDecl(VariableDeclaration(
                name,
                init_expr,
                derived_type,
                storage_class,
            ))
        }
    }
}

// <specifier> ::= <type-specifier> | "static" | "extern"
// parses { <specifier> }+ into a type and storage class tuple
// can be heavily optimized but whatever, lists are small
pub fn parse_type_and_storage_class(tokens: &mut VecDeque<Token>) -> (Type, Option<StorageClass>) {
    let mut storage_specifiers: Vec<Token> = Vec::new();
    let mut type_specifiers: Vec<Token> = Vec::new();

    loop {
        let next_token = peek(tokens);
        match next_token {
            t if t.is_type_specifier() => {
                type_specifiers.push(t.clone());
                take_token(tokens);
            }
            t if t.is_storage_class_specifier() => {
                storage_specifiers.push(t.clone());
                take_token(tokens);
            }
            _ => break,
        }
    }

    let ty = parse_type_from_specifiers(&mut type_specifiers);
    let storage_class = parse_storage_class_from_specifiers(&mut storage_specifiers);

    (ty, storage_class)
}

// parses { <specifier> }+ into a single type
pub fn parse_type_from_specifiers(specifier_tokens: &mut Vec<Token>) -> Type {
    // Specifier list must be non-empty
    if specifier_tokens.is_empty() {
        panic!("Syntax Error: Expected at least one type specifier");
    }

    // Doubles can't be combined with other types
    if specifier_tokens == &vec![Token::DoubleKeyword] {
        return Type::Double;
    }
    if specifier_tokens == &vec![Token::FloatKeyword] {
        return Type::Float;
    }
    if specifier_tokens.contains(&Token::DoubleKeyword)
        || specifier_tokens.contains(&Token::FloatKeyword)
    {
        panic!("Syntax Error: Can't combine 'double' with other type specifiers")
    }

    // Specifiers can't be repeated
    let mut seen_specifiers = HashMap::new();
    for token in specifier_tokens.iter() {
        if seen_specifiers.contains_key(token) {
            panic!("Syntax Error: Specifier {:?} is repeated", token);
        }
        seen_specifiers.insert(token.clone(), true);
    }

    // Sign specification must be consistent
    if specifier_tokens.contains(&Token::UnsignedKeyword)
        && specifier_tokens.contains(&Token::SignedKeyword)
    {
        panic!("Syntax Error: Can't specify both 'signed' and 'unsigned'");
    }

    // Simple specification rules
    let unsigned = specifier_tokens.contains(&Token::UnsignedKeyword);
    let long = specifier_tokens.contains(&Token::LongKeyword);

    if unsigned {
        if long { Type::ULong } else { Type::UInt }
    } else {
        if long { Type::Long } else { Type::Int }
    }
}

// parses { <specifier> }+ into a single storage class
pub fn parse_storage_class_from_specifiers(
    specifier_tokens: &mut Vec<Token>,
) -> Option<StorageClass> {
    if specifier_tokens.len() > 1 {
        panic!(
            "Syntax Error: Multiple specifiers found where only one storage class specifier is allowed"
        );
    }

    if specifier_tokens.contains(&Token::Static) {
        Some(StorageClass::Static)
    } else if specifier_tokens.contains(&Token::Extern) {
        Some(StorageClass::Extern)
    } else {
        None
    }
}

// <declarator> ::= "*" <declarator> | <direct-declarator>
// <direct-declarator> ::= <simple-declarator> [ <param-list> ]
// <param-list> ::= "(" "void" ")" | "(" <param> { "," <param> } ")"
// <param> ::= { <type-specifier> }+ <declarator>
// <simple-declarator> ::= <identifier> | "(" <declarator> ")"
#[derive(Debug, Clone, PartialEq, Eq)]
enum Declarator {
    Ident(String),
    PointerDeclarator(Box<Declarator>),
    FunDeclarator(Vec<ParamInfo>, Box<Declarator>),
}
#[derive(Debug, Clone, PartialEq, Eq)]
struct ParamInfo(Type, Declarator);

pub fn parse_declarator(tokens: &mut VecDeque<Token>) -> Declarator {
    match peek(tokens) {
        Token::Asterisk => {
            take_token(tokens);
            let inner_declarator = parse_declarator(tokens);
            Declarator::PointerDeclarator(Box::new(inner_declarator))
        }
        _ => parse_direct_declarator(tokens),
    }
}

pub fn parse_direct_declarator(tokens: &mut VecDeque<Token>) -> Declarator {
    let simple_declarator = parse_simple_declarator(tokens);

    match peek(tokens) {
        Token::OpenParenthesis => {
            take_token(tokens);
            let params = parse_param_list(tokens);
            expect(Token::CloseParenthesis, tokens);
            Declarator::FunDeclarator(params, Box::new(simple_declarator))
        }
        _ => simple_declarator,
    }
}

pub fn parse_simple_declarator(tokens: &mut VecDeque<Token>) -> Declarator {
    match peek(tokens) {
        Token::Identifier(_) => {
            let ident = parse_identifier(tokens);
            Declarator::Ident(ident)
        }
        Token::OpenParenthesis => {
            take_token(tokens);
            let inner_declarator = parse_declarator(tokens);
            expect(Token::CloseParenthesis, tokens);
            inner_declarator
        }
        _ => panic!("Expected a declarator but found {:?}", peek(tokens)),
    }
}

// <param-list> ::= "(" "void" ")" | "(" <param> { "," <param> } ")"
pub fn parse_param_list(tokens: &mut VecDeque<Token>) -> Vec<ParamInfo> {
    let mut params = Vec::new();

    match peek(tokens) {
        Token::CloseParenthesis => return params,
        Token::VoidKeyword => {
            take_token(tokens);
            if !matches!(peek(tokens), Token::CloseParenthesis) {
                panic!("Expected ')' after 'void' in parameter list");
            }
        }
        t if t.is_type_specifier() => {
            let (ty, _) = parse_type_and_storage_class(tokens);
            let declarator = parse_declarator(tokens);
            params.push(ParamInfo(ty, declarator));

            while matches!(peek(tokens), Token::Comma) {
                take_token(tokens);
                let (ty, _) = parse_type_and_storage_class(tokens);
                let declarator = parse_declarator(tokens);
                params.push(ParamInfo(ty, declarator));
            }
        }
        _ => panic!(
            "Expected a parameter or 'void' in parameter list but found {:?}",
            peek(tokens)
        ),
    }

    params
}

pub fn process_declarator(declarator: Declarator, base_ty: Type) -> (String, Type, Vec<String>) {
    // returns declarator name, derived type, and parameter names if it's a function
    match declarator {
        Declarator::Ident(name) => (name, base_ty, Vec::new()),
        Declarator::PointerDeclarator(inner) => {
            let derived_ty = Type::Pointer(Box::new(base_ty));
            process_declarator(*inner, derived_ty)
        }
        Declarator::FunDeclarator(params_info, inner) => match *inner {
            Declarator::Ident(name) => {
                let mut param_names = Vec::new();
                let mut param_types = Vec::new();
                for ParamInfo(param_ty, param_declarator) in params_info {
                    let (param_name, derived_param_ty, _) =
                        process_declarator(param_declarator, param_ty);

                    param_names.push(param_name);
                    param_types.push(derived_param_ty);
                }
                let derived_ty = Type::FunType(param_types, Box::new(base_ty));
                (name, derived_ty, param_names)
            }
            _ => panic!("Function declarator must have an identifier"),
        },
    }
}

// <block-item> ::= <statement> | <declaration>
pub fn parse_block_item(tokens: &mut VecDeque<Token>) -> BlockItem {
    match peek(tokens) {
        t if t.is_specifier() => BlockItem::D(parse_declaration(tokens)),
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
        t if t.is_specifier() => {
            let decl = parse_declaration(tokens);
            match decl {
                Declaration::VarDecl(var_decl) => ForInit::InitDecl(var_decl),
                _ => panic!("Expected variable declaration in for loop initialization"),
            }
        }
        _ => {
            let init_expr = parse_optional_expr(tokens, 0, Token::Semicolon);
            expect(Token::Semicolon, tokens);
            ForInit::InitExp(init_expr)
        }
    }
}

// <exp> ::= <factor> | <exp> <binop> <exp>
pub fn parse_expr(tokens: &mut VecDeque<Token>, min_prec: i32) -> Expr {
    // implemented with precedence climbing
    let mut left_expr = parse_factor(tokens);
    let mut next_token = peek(tokens).clone();

    while next_token.is_binop() && next_token.precedence() >= min_prec {
        // Right associative
        if next_token == Token::Equal {
            take_token(tokens);
            let right_expr = parse_expr(tokens, next_token.precedence());
            left_expr = Expr::Assignment(Box::new(left_expr), Box::new(right_expr), None);
        } else if next_token == Token::QuestionMark {
            take_token(tokens);
            let middle_expr = parse_expr(tokens, 0);
            expect(Token::Colon, tokens);
            let right_expr = parse_expr(tokens, next_token.precedence());
            left_expr = Expr::Conditional(
                Box::new(left_expr),
                Box::new(middle_expr),
                Box::new(right_expr),
                None,
            );
        // Left associative
        } else {
            let operator = parse_binop(tokens);
            let right_expr = parse_expr(tokens, next_token.precedence() + 1);
            left_expr = Expr::Binary(operator, Box::new(left_expr), Box::new(right_expr), None);
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

// <factor> ::= <const> | <identifier>
//     | "(" { <type-specifier> }+ [ <abstract-declarator> ] ")" <factor>
//     | <unop> <factor> | "(" <exp> ")"
//     | <identifier> "(" [ <argument-list> ] ")"
pub fn parse_factor(tokens: &mut VecDeque<Token>) -> Expr {
    match peek(tokens) {
        e if e.is_constant_token() => {
            let cons = parse_constant(tokens);
            let expr = Expr::Constant(cons, None);
            expr
        }
        Token::Hyphen | Token::Tilde | Token::Exclamation => {
            let operator = parse_unop(tokens);
            let inner_expr = parse_factor(tokens);
            Expr::Unary(operator, Box::new(inner_expr), None)
        }
        Token::Asterisk => {
            take_token(tokens);
            let inner_expr = parse_factor(tokens);
            Expr::Dereference(Box::new(inner_expr), None)
        }
        Token::Ampersand => {
            take_token(tokens);
            let inner_expr = parse_factor(tokens);
            Expr::AddressOf(Box::new(inner_expr), None)
        }
        Token::OpenParenthesis => match peek_n(tokens, 1).is_type_specifier() {
            true => {
                take_token(tokens);
                let (ty, _) = parse_type_and_storage_class(tokens);
                let abstract_declarator = parse_abstract_declarator(tokens);
                let derived_ty = process_abstract_declarator(abstract_declarator, ty);
                expect(Token::CloseParenthesis, tokens);
                let factor = parse_factor(tokens);
                Expr::Cast(derived_ty, Box::new(factor), None)
            }
            false => {
                take_token(tokens);
                let inner_expr = parse_expr(tokens, 0);
                expect(Token::CloseParenthesis, tokens);
                inner_expr
            }
        },
        Token::Identifier(s) => match peek_n(tokens, 1) {
            Token::OpenParenthesis => {
                let func_name = s.to_string();
                take_token(tokens);
                take_token(tokens);
                let arguments = parse_argument_list(tokens);
                expect(Token::CloseParenthesis, tokens);
                Expr::FunctionCall(func_name, arguments, None)
            }
            _ => {
                let expr = Expr::Var(s.to_string(), None);
                take_token(tokens);
                expr
            }
        },
        _ => panic!("Malformed Expression"),
    }
}

// <abstract-declarator> ::= "*" [ <abstract-declarator> ]
//     | <direct-abstract-declarator>
// <direct-abstract-declarator> ::= "(" <abstract-declarator> ")"
#[derive(Debug, Clone, PartialEq, Eq)]
enum AbstractDeclarator {
    AbstractPointer(Box<AbstractDeclarator>),
    AbstractBase,
}
pub fn parse_abstract_declarator(tokens: &mut VecDeque<Token>) -> Option<AbstractDeclarator> {
    match peek(tokens) {
        Token::Asterisk => {
            take_token(tokens);
            let inner_declarator = parse_abstract_declarator(tokens);
            match inner_declarator {
                Some(inner) => Some(AbstractDeclarator::AbstractPointer(Box::new(inner))),
                None => Some(AbstractDeclarator::AbstractPointer(Box::new(
                    AbstractDeclarator::AbstractBase,
                ))),
            }
        }
        Token::OpenParenthesis => {
            take_token(tokens);
            let inner_declarator = parse_abstract_declarator(tokens);
            expect(Token::CloseParenthesis, tokens);
            match inner_declarator {
                Some(inner) => Some(inner),
                None => Some(AbstractDeclarator::AbstractBase),
            }
        }
        _ => None,
    }
}

pub fn process_abstract_declarator(
    abstract_declarator: Option<AbstractDeclarator>,
    base_ty: Type,
) -> Type {
    match abstract_declarator {
        Some(AbstractDeclarator::AbstractPointer(inner)) => {
            let derived_ty = Type::Pointer(Box::new(base_ty));
            process_abstract_declarator(Some(*inner), derived_ty)
        }
        Some(AbstractDeclarator::AbstractBase) => base_ty,
        None => base_ty,
    }
}

pub fn parse_constant(tokens: &mut VecDeque<Token>) -> Const {
    match peek(tokens) {
        Token::Constant(i) => {
            let cons = Const::ConstInt(*i);
            take_token(tokens);
            cons
        }
        Token::UConstant(i) => {
            let cons = Const::ConstUInt(*i);
            take_token(tokens);
            cons
        }
        Token::LongConstant(i) => {
            let cons = Const::ConstLong(*i);
            take_token(tokens);
            cons
        }
        Token::ULongConstant(i) => {
            let cons = Const::ConstULong(*i);
            take_token(tokens);
            cons
        }
        Token::DFloatConstant(OrderedFloat(f)) => {
            let cons = Const::ConstDouble(*f);
            take_token(tokens);
            cons
        }
        Token::SFloatConstant(OrderedFloat(f)) => {
            let cons = Const::ConstFloat(*f);
            take_token(tokens);
            cons
        }
        _ => panic!("Expected a constant but found {:?}", peek(tokens)),
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
