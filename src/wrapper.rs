pub use std::string::String;

#[unsafe(no_mangle)]
pub fn string_new() -> String {
    String::new()
}

#[unsafe(no_mangle)]
pub fn string_push(s: &mut String, c: char) {
    s.push(c);
}

#[unsafe(no_mangle)]
pub fn string_as_str(s: &String) -> &str {
    s.as_str()
}

#[unsafe(no_mangle)]
pub fn string_from_str(s: &str) -> String {
    String::from(s)
}

fn string_clone(s: &String) -> String {
    s.clone()
}
fn string_eq(a: &String, b: &String) -> bool {
    a == b
}

pub struct VecChar {
    inner: Vec<char>,
}

#[unsafe(no_mangle)]
pub fn vec_char_from_str(s: &str) -> VecChar {
    VecChar {
        inner: s.chars().collect(),
    }
}

#[unsafe(no_mangle)]
pub fn vec_char_len(v: &VecChar) -> usize {
    v.inner.len()
}

#[unsafe(no_mangle)]
pub fn vec_char_get(v: &VecChar, i: usize) -> OptionChar {
    match v.inner.get(i) {
        Some(c) => OptionChar::Some(*c),
        None => OptionChar::None,
    }
}

pub enum OptionChar {
    Some(char),
    None,
}

pub struct VecToken {
    inner: Vec<Token>,
}

#[unsafe(no_mangle)]
pub fn vec_token_new() -> VecToken {
    VecToken { inner: Vec::new() }
}

#[unsafe(no_mangle)]
pub fn vec_token_push(v: &mut VecToken, t: Token) {
    v.inner.push(t);
}

#[unsafe(no_mangle)]
pub fn vec_token_len(v: &VecToken) -> usize {
    v.inner.len()
}

#[unsafe(no_mangle)]
pub fn vec_token_get(v: &VecToken, i: usize) -> OptionToken {
    match v.inner.get(i) {
        Some(t) => OptionToken::Some(token_clone(t)),
        None => OptionToken::None,
    }
}

#[unsafe(no_mangle)]
pub fn char_is_whitespace(c: char) -> bool {
    c.is_ascii_whitespace()
}

#[unsafe(no_mangle)]
pub fn char_is_digit(c: char) -> bool {
    c.is_ascii_digit()
}

#[unsafe(no_mangle)]
pub fn char_is_alphabetic(c: char) -> bool {
    c.is_alphabetic()
}

#[unsafe(no_mangle)]
pub fn char_is_alphanumeric(c: char) -> bool {
    c.is_alphanumeric()
}

#[unsafe(no_mangle)]
pub fn str_eq(a: &str, b: &str) -> bool {
    a == b
}

// === I/O ===

#[unsafe(no_mangle)]
pub fn panic_exit(msg: &str) -> ! {
    eprintln!("Error: {}", msg);
    std::process::exit(1)
}

pub enum Token {
    Fn,
    Let,
    If,
    While,
    Return,
    Enum,
    Match,
    LBrace,
    RBrace,
    LParen,
    RParen,
    Colon,
    SemiColon,
    Comma,
    Eq,
    Assign,
    Arrow,
    Plus,
    Minus,
    Star,
    Slash,
    Identifier(String),
    Integer(u64),
    StringLiteral(String),
    Eof,
}

pub enum OptionToken {
    Some(Token),
    None,
}

/// Manual clone
pub fn token_clone(t: &Token) -> Token {
    match t {
        Token::Fn => Token::Fn,
        Token::Let => Token::Let,
        Token::If => Token::If,
        Token::While => Token::While,
        Token::Return => Token::Return,
        Token::Enum => Token::Enum,
        Token::Match => Token::Match,
        Token::LBrace => Token::LBrace,
        Token::RBrace => Token::RBrace,
        Token::LParen => Token::LParen,
        Token::RParen => Token::RParen,
        Token::Colon => Token::Colon,
        Token::SemiColon => Token::SemiColon,
        Token::Comma => Token::Comma,
        Token::Eq => Token::Eq,
        Token::Assign => Token::Assign,
        Token::Arrow => Token::Arrow,
        Token::Plus => Token::Plus,
        Token::Minus => Token::Minus,
        Token::Star => Token::Star,
        Token::Slash => Token::Slash,
        Token::Identifier(s) => Token::Identifier(string_clone(s)),
        Token::Integer(n) => Token::Integer(*n),
        Token::StringLiteral(s) => Token::StringLiteral(string_clone(s)),
        Token::Eof => Token::Eof,
    }
}

/// Manual equality
pub fn token_eq(a: &Token, b: &Token) -> bool {
    match (a, b) {
        (Token::Fn, Token::Fn) => true,
        (Token::Let, Token::Let) => true,
        (Token::If, Token::If) => true,
        (Token::While, Token::While) => true,
        (Token::Return, Token::Return) => true,
        (Token::Enum, Token::Enum) => true,
        (Token::Match, Token::Match) => true,
        (Token::LBrace, Token::LBrace) => true,
        (Token::RBrace, Token::RBrace) => true,
        (Token::LParen, Token::LParen) => true,
        (Token::RParen, Token::RParen) => true,
        (Token::Colon, Token::Colon) => true,
        (Token::SemiColon, Token::SemiColon) => true,
        (Token::Comma, Token::Comma) => true,
        (Token::Eq, Token::Eq) => true,
        (Token::Assign, Token::Assign) => true,
        (Token::Arrow, Token::Arrow) => true,
        (Token::Plus, Token::Plus) => true,
        (Token::Minus, Token::Minus) => true,
        (Token::Star, Token::Star) => true,
        (Token::Slash, Token::Slash) => true,
        (Token::Identifier(a), Token::Identifier(b)) => string_eq(a, b),
        (Token::Integer(a), Token::Integer(b)) => *a == *b,
        (Token::StringLiteral(a), Token::StringLiteral(b)) => string_eq(a, b),
        (Token::Eof, Token::Eof) => true,
        _ => false,
    }
}
