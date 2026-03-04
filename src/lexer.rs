use regex::Regex;

#[derive(PartialEq, Eq, PartialOrd, Ord, Copy, Clone, Debug)]
pub enum TokenId {
    Identifier,
    Constant,
    IntKeyword,
    VoidKeyword,
    ReturnKeyword,
    OpenParenthesis,
    CloseParenthesis,
    OpenBrace,
    CloseBrace,
    Semicolon,
}

#[derive(Debug)]
pub struct Token {
    id: TokenId,
    value: Option<String>,
}

pub fn lexical_analysis(content: &str) -> Vec<Token> {
    let mut tokens = Vec::new();

    let rules = vec![
        (TokenId::Identifier, Regex::new(r"^[a-zA-Z_]\w*\b").unwrap()),
        (TokenId::Constant, Regex::new(r"^[0-9]+\b").unwrap()),
        (TokenId::IntKeyword, Regex::new(r"^int\b").unwrap()),
        (TokenId::VoidKeyword, Regex::new(r"^void\b").unwrap()),
        (TokenId::ReturnKeyword, Regex::new(r"^return\b").unwrap()),
        (TokenId::OpenParenthesis, Regex::new(r"^\(").unwrap()),
        (TokenId::CloseParenthesis, Regex::new(r"^\)").unwrap()),
        (TokenId::OpenBrace, Regex::new(r"^\{").unwrap()),
        (TokenId::CloseBrace, Regex::new(r"^\}").unwrap()),
        (TokenId::Semicolon, Regex::new(r"^;").unwrap()),
    ];

    let mut i = 0;
    while i < content.len() {
        let remaining_content = &content[i..];

        if remaining_content.starts_with(' ') || remaining_content.starts_with("\n") {
            i += 1;
            continue;
        }

        let mut token_matches = Vec::new();
        for rule in &rules {
            let rule_id = &rule.0;
            let re = &rule.1;

            // Find all token matches
            if let Some(m) = re.find(remaining_content) {
                let token = (m.as_str(), rule_id);
                token_matches.push(token);
            }
        }

        // Select the maximum token by lexicographic order
        let max_match = *token_matches.iter().max().unwrap();
        i += max_match.0.len();

        let mut token = Token {
            id: *max_match.1,
            value: None,
        };

        if matches!(token.id, TokenId::Identifier | TokenId::Constant) {
            token.value = Some(max_match.0.to_string());
        }

        println!("{:?}", token);
        tokens.push(token);
    }

    tokens
}
