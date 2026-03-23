use crate::runtime::{CharList, OptionChar, OptionToken, TokenList};
use crate::token::Token;

// TODO: If there are suitable functions or methods in the std library, then you may use them
// Helper functions (not methods)
fn is_whitespace(c: char) -> bool {
    c == ' ' || c == '\n' || c == '\t' || c == '\r'
}

fn is_digit(c: char) -> bool {
    c >= '0' && c <= '9'
}

fn is_alphabetic(c: char) -> bool {
    (c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z') || c == '_'
}

fn is_alphanumeric(c: char) -> bool {
    is_alphabetic(c) || is_digit(c)
}

pub fn lex(source: String) -> TokenList {
    let chars: CharList = CharList::from_string(source);
    let mut tokens: TokenList = TokenList::new();
    let mut i: usize = 0;
    let len: usize = chars.len();

    while i < len {
        let (token_opt, next_i): (OptionToken, usize) = next_token(&chars, i);

        // Ensure progress
        if next_i <= i {
            // Should not happen, force advance
            i = i + 1;
        } else {
            i = next_i;
        }

        match token_opt {
            OptionToken::Some(t) => tokens.push(t),
            OptionToken::None => {} // Skip (whitespace/comments)
        }
    }
    tokens.push(Token::EOF);
    tokens
}

// TODO: Use match on the character instead of if statements
fn next_token(chars: &CharList, i: usize) -> (OptionToken, usize) {
    let c_opt: OptionChar = chars.get(i);
    match c_opt {
        OptionChar::Some(c) => {
            if is_whitespace(c) {
                return (OptionToken::None, i + 1);
            }
            if c == '/' {
                return scan_slash_or_comment(chars, i);
            }
            if is_digit(c) {
                return scan_integer(chars, i);
            }
            if is_alphabetic(c) {
                return scan_identifier(chars, i);
            }
            if c == '"' {
                return scan_string(chars, i);
            }
            scan_symbol(c, chars, i)
        }
        OptionChar::None => (OptionToken::None, i + 1), // Should be handled by loop condition
    }
}

fn scan_slash_or_comment(chars: &CharList, i: usize) -> (OptionToken, usize) {
    let next_i: usize = i + 1;
    let next: OptionChar = chars.get(next_i);
    match next {
        OptionChar::Some(nc) => {
            if nc == '/' {
                let end_comment_i: usize = skip_line_comment(chars, next_i + 1);
                return (OptionToken::None, end_comment_i);
            } else {
                return (OptionToken::Some(Token::Slash), next_i);
            }
        }
        OptionChar::None => (OptionToken::Some(Token::Slash), next_i),
    }
}

// TODO: If there is a suitable function/method that shortens this, you may use it
fn skip_line_comment(chars: &CharList, mut i: usize) -> usize {
    let len: usize = chars.len();
    while i < len {
        let c_opt: OptionChar = chars.get(i);
        match c_opt {
            OptionChar::Some(c) => {
                if c == '\n' {
                    return i; // Stop AT newline, lexer loop handles newline as whitespace
                }
                i = i + 1;
            }
            OptionChar::None => return i,
        }
    }
    i
}

fn scan_integer(chars: &CharList, start_i: usize) -> (OptionToken, usize) {
    let mut val: u64 = 0;
    let mut i: usize = start_i;
    let len: usize = chars.len();

    while i < len {
        let digit_opt: OptionChar = chars.get(i);
        match digit_opt {
            OptionChar::Some(digit) => {
                if is_digit(digit) {
                    val = val * 10 + (digit as u64 - '0' as u64);
                    i = i + 1;
                } else {
                    return (OptionToken::Some(Token::Integer(val)), i);
                }
            }
            OptionChar::None => {
                return (OptionToken::Some(Token::Integer(val)), i);
            }
        }
    }
    (OptionToken::Some(Token::Integer(val)), i)
}

// TODO: Identifiers may also contain _
fn scan_identifier(chars: &CharList, start_i: usize) -> (OptionToken, usize) {
    let mut s: String = String::new();
    let mut i: usize = start_i;
    let len: usize = chars.len();

    while i < len {
        let ch_opt: OptionChar = chars.get(i);
        match ch_opt {
            OptionChar::Some(ch) => {
                if is_alphanumeric(ch) {
                    s.push(ch);
                    i = i + 1;
                } else {
                    return (OptionToken::Some(keyword_or_id(s)), i);
                }
            }
            OptionChar::None => {
                return (OptionToken::Some(keyword_or_id(s)), i);
            }
        }
    }
    (OptionToken::Some(keyword_or_id(s)), i)
}

// TODO: rewrite this using match and no return keyword (functional style)
fn keyword_or_id(s: String) -> Token {
    if s == "fn" {
        return Token::Fn;
    }
    if s == "let" {
        return Token::Let;
    }
    if s == "if" {
        return Token::If;
    }
    if s == "while" {
        return Token::While;
    }
    if s == "return" {
        return Token::Return;
    }
    if s == "enum" {
        return Token::Enum;
    }
    if s == "match" {
        return Token::Match;
    }
    Token::Identifier(s)
}

// TODO: In the event of failure you should throw a syntax error (print and exit)
fn scan_string(chars: &CharList, start_i: usize) -> (OptionToken, usize) {
    let mut i: usize = start_i + 1; // skip opening quote
    let mut s: String = String::new();
    let len: usize = chars.len();

    while i < len {
        let ch_opt: OptionChar = chars.get(i);
        match ch_opt {
            OptionChar::Some(ch) => {
                if ch == '"' {
                    return (OptionToken::Some(Token::StringLiteral(s)), i + 1); // +1 to skip closing quote
                }
                s.push(ch);
                i = i + 1;
            }
            OptionChar::None => {
                // Unterminated string, ideally error, but here just return what we have?
                // Or consume until EOF.
                return (OptionToken::Some(Token::StringLiteral(s)), i);
            }
        }
    }
    (OptionToken::Some(Token::StringLiteral(s)), i)
}

// TODO: Rewrite this using match and without explicit return
// In the event of unknown you should throw a syntax error (print and exit)
fn scan_symbol(c: char, chars: &CharList, i: usize) -> (OptionToken, usize) {
    if c == '{' {
        return (OptionToken::Some(Token::LBrace), i + 1);
    }
    if c == '}' {
        return (OptionToken::Some(Token::RBrace), i + 1);
    }
    if c == '(' {
        return (OptionToken::Some(Token::LParen), i + 1);
    }
    if c == ')' {
        return (OptionToken::Some(Token::RParen), i + 1);
    }
    if c == ':' {
        return (OptionToken::Some(Token::Colon), i + 1);
    }
    if c == ';' {
        return (OptionToken::Some(Token::SemiColon), i + 1);
    }
    if c == ',' {
        return (OptionToken::Some(Token::Comma), i + 1);
    }
    if c == '+' {
        return (OptionToken::Some(Token::Plus), i + 1);
    }
    if c == '*' {
        return (OptionToken::Some(Token::Star), i + 1);
    }

    if c == '-' {
        // Check for ->
        let next_i: usize = i + 1;
        match chars.get(next_i) {
            OptionChar::Some('>') => return (OptionToken::Some(Token::Arrow), next_i + 1),
            _ => return (OptionToken::Some(Token::Minus), next_i),
        }
    }

    if c == '=' {
        // Check for ==
        let next_i: usize = i + 1;
        match chars.get(next_i) {
            OptionChar::Some('=') => return (OptionToken::Some(Token::Eq), next_i + 1),
            _ => return (OptionToken::Some(Token::Assign), next_i),
        }
    }

    // Unknown char
    (OptionToken::None, i + 1)
}
