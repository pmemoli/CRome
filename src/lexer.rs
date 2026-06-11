use regex::Regex;
use std::collections::VecDeque;

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Hash)]
pub enum Token {
    Identifier(String),
    Constant(i32),
    UConstant(u32),
    LongConstant(i64),
    ULongConstant(u64),
    SignedKeyword,
    UnsignedKeyword,
    LongKeyword,
    IntKeyword,
    VoidKeyword,
    ReturnKeyword,
    OpenParenthesis,
    CloseParenthesis,
    OpenBrace,
    CloseBrace,
    Semicolon,
    Tilde,
    Hyphen,
    TwoHyphens,
    Plus,
    Asterisk,
    ForwardSlash,
    Percent,
    Exclamation,
    TwoAmpersand,
    TwoVerticalBar,
    TwoEqual,
    NotEqual,
    LessThan,
    LessThanOrEqual,
    GreaterThan,
    GreaterThanOrEqual,
    Equal,
    IfKeyword,
    ElseKeyword,
    QuestionMark,
    Colon,
    DoKeyword,
    WhileKeyword,
    ForKeyword,
    BreakKeyword,
    ContinueKeyword,
    Comma,
    Static,
    Extern,
}

pub fn lexical_analysis(content: &str) -> VecDeque<Token> {
    let mut tokens = VecDeque::new();

    let rules: Vec<(Regex, fn(&str) -> Token)> = vec![
        (Regex::new(r"^[a-zA-Z_]\w*\b").unwrap(), |s| {
            Token::Identifier(s.to_string())
        }),
        // Signed constants
        (Regex::new(r"^[0-9]+\b").unwrap(), |s| {
            match s.parse::<i32>() {
                Ok(num) => Token::Constant(num),
                Err(_) => Token::LongConstant(s.parse::<i64>().unwrap()),
            }
        }),
        (Regex::new(r"^[0-9]+[lL]\b").unwrap(), |s| {
            Token::LongConstant(s[..s.len() - 1].parse::<i64>().unwrap())
        }),
        // Unsigned constants
        (Regex::new(r"^[0-9]+[uU]\b").unwrap(), |s| {
            match s[..s.len() - 1].parse::<u32>() {
                Ok(num) => Token::UConstant(num),
                Err(_) => Token::ULongConstant(s[..s.len() - 1].parse::<u64>().unwrap()),
            }
        }),
        (Regex::new(r"^[0-9]+([lL][uU]|[uU][lL])\b").unwrap(), |s| {
            Token::ULongConstant(s[..s.len() - 2].parse::<u64>().unwrap())
        }),
        (Regex::new(r"^signed\b").unwrap(), |_| Token::SignedKeyword),
        (Regex::new(r"^unsigned\b").unwrap(), |_| {
            Token::UnsignedKeyword
        }),
        (Regex::new(r"^long\b").unwrap(), |_| Token::LongKeyword),
        (Regex::new(r"^int\b").unwrap(), |_| Token::IntKeyword),
        (Regex::new(r"^void\b").unwrap(), |_| Token::VoidKeyword),
        (Regex::new(r"^return\b").unwrap(), |_| Token::ReturnKeyword),
        (Regex::new(r"^\(").unwrap(), |_| Token::OpenParenthesis),
        (Regex::new(r"^\)").unwrap(), |_| Token::CloseParenthesis),
        (Regex::new(r"^\{").unwrap(), |_| Token::OpenBrace),
        (Regex::new(r"^\}").unwrap(), |_| Token::CloseBrace),
        (Regex::new(r"^;").unwrap(), |_| Token::Semicolon),
        (Regex::new(r"^~").unwrap(), |_| Token::Tilde),
        (Regex::new(r"^--").unwrap(), |_| Token::TwoHyphens),
        (Regex::new(r"^!").unwrap(), |_| Token::Exclamation),
        (Regex::new(r"^-").unwrap(), |_| Token::Hyphen),
        (Regex::new(r"^\+").unwrap(), |_| Token::Plus),
        (Regex::new(r"^\*").unwrap(), |_| Token::Asterisk),
        (Regex::new(r"^/").unwrap(), |_| Token::ForwardSlash),
        (Regex::new(r"^%").unwrap(), |_| Token::Percent),
        (Regex::new(r"^&&").unwrap(), |_| Token::TwoAmpersand),
        (Regex::new(r"^\|\|").unwrap(), |_| Token::TwoVerticalBar),
        (Regex::new(r"^==").unwrap(), |_| Token::TwoEqual),
        (Regex::new(r"^!=").unwrap(), |_| Token::NotEqual),
        (Regex::new(r"^<").unwrap(), |_| Token::LessThan),
        (Regex::new(r"^<=").unwrap(), |_| Token::LessThanOrEqual),
        (Regex::new(r"^>").unwrap(), |_| Token::GreaterThan),
        (Regex::new(r"^>=").unwrap(), |_| Token::GreaterThanOrEqual),
        (Regex::new(r"^=").unwrap(), |_| Token::Equal),
        (Regex::new(r"^if\b").unwrap(), |_| Token::IfKeyword),
        (Regex::new(r"^else\b").unwrap(), |_| Token::ElseKeyword),
        (Regex::new(r"^\?").unwrap(), |_| Token::QuestionMark),
        (Regex::new(r"^:").unwrap(), |_| Token::Colon),
        (Regex::new(r"^do\b").unwrap(), |_| Token::DoKeyword),
        (Regex::new(r"^while\b").unwrap(), |_| Token::WhileKeyword),
        (Regex::new(r"^for\b").unwrap(), |_| Token::ForKeyword),
        (Regex::new(r"^break\b").unwrap(), |_| Token::BreakKeyword),
        (Regex::new(r"^continue\b").unwrap(), |_| {
            Token::ContinueKeyword
        }),
        (Regex::new(r"^,").unwrap(), |_| Token::Comma),
        (Regex::new(r"^static\b").unwrap(), |_| Token::Static),
        (Regex::new(r"^extern\b").unwrap(), |_| Token::Extern),
    ];

    let mut i = 0;
    while i < content.len() {
        let remaining_content = &content[i..];

        if remaining_content.starts_with(' ') || remaining_content.starts_with("\n") {
            i += 1;
            continue;
        }

        // Find all token matches
        let mut token_matches = Vec::new();
        for (re, constructor) in &rules {
            if let Some(m) = re.find(remaining_content) {
                let m_str = m.as_str();
                token_matches.push((m_str, constructor(m_str)));
            }
        }

        // Select the maximum token by lexicographic order (length and enum)
        let max_match = token_matches.iter().max().unwrap();

        i += max_match.0.len();

        tokens.push_back(max_match.1.clone());
    }

    tokens
}
