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
