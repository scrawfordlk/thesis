// TODO: You may not use derive macros, except for Debug (during development)
#[derive(Clone, Debug, PartialEq)]
pub enum Token {
    // Keywords
    Fn,
    Let,
    If,
    While,
    Return,
    Enum,
    Match,

    // Symbols
    LBrace,
    RBrace,
    LParen,
    RParen,
    Colon,
    SemiColon,
    Comma,
    Eq,
    Assign,
    Arrow, // ->
    Plus,
    Minus,
    Star,
    Slash,

    // Literals
    Identifier(String),
    Integer(u64),
    StringLiteral(String),

    // End of File
    EOF,
}

// TODO: remove unnecessary examples
impl Token {
    // Example method for enum (allowed now)
    pub fn is_eof(&self) -> bool {
        match self {
            Token::EOF => true,
            _ => false,
        }
    }
}
