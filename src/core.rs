use wrapper::{
    OptionChar, OptionToken, String, Token, VecChar, VecToken, char_is_alphabetic,
    char_is_alphanumeric, char_is_digit, char_is_whitespace, panic_exit, str_eq, string_as_str,
    string_from_str, string_new, string_push, vec_char_from_str, vec_char_get, vec_char_len,
    vec_token_get, vec_token_new, vec_token_push,
};

enum LexResult {
    Token(Token),
    Skip,
}

fn main() {}

/// Lexes source into tokens.
pub fn lex(source: &str) -> VecToken {
    let chars: VecChar = vec_char_from_str(source);
    let mut tokens: VecToken = vec_token_new();
    let mut i: usize = 0;
    let len: usize = vec_char_len(&chars);

    while i < len {
        let c_opt: OptionChar = vec_char_get(&chars, i);
        match c_opt {
            OptionChar::Some(c) => {
                if char_is_whitespace(c) {
                    i = i + 1;
                } else {
                    let result: (LexResult, usize) = scan_token(&chars, i, len);
                    match result {
                        (LexResult::Token(t), next_i) => {
                            vec_token_push(&mut tokens, t);
                            i = next_i;
                        }
                        (LexResult::Skip, next_i) => {
                            i = next_i;
                        }
                    }
                }
            }
            OptionChar::None => {
                i = i + 1;
            }
        }
    }

    vec_token_push(&mut tokens, Token::Eof);
    tokens
}

/// Scans a single token.
fn scan_token(chars: &VecChar, i: usize, len: usize) -> (LexResult, usize) {
    let c: char = unwrap_char(vec_char_get(chars, i));

    if c == '/' {
        return scan_slash_or_comment(chars, i, len);
    }
    if char_is_digit(c) {
        return scan_integer(chars, i, len);
    }
    if char_is_alphabetic(c) {
        return scan_identifier(chars, i, len);
    }
    if c == '_' {
        return scan_identifier(chars, i, len);
    }
    if c == '"' {
        return scan_string(chars, i, len);
    }

    scan_symbol(chars, i)
}

/// Scans '/' or comment.
fn scan_slash_or_comment(chars: &VecChar, i: usize, len: usize) -> (LexResult, usize) {
    let next_opt: OptionChar = vec_char_get(chars, i + 1);
    match next_opt {
        OptionChar::Some(next) => {
            if next == '/' {
                let end: usize = skip_line_comment(chars, i + 2, len);
                return (LexResult::Skip, end);
            }
            (LexResult::Token(Token::Slash), i + 1)
        }
        OptionChar::None => (LexResult::Token(Token::Slash), i + 1),
    }
}

/// Skips to end of line.
fn skip_line_comment(chars: &VecChar, start: usize, len: usize) -> usize {
    let mut i: usize = start;
    while i < len {
        let c_opt: OptionChar = vec_char_get(chars, i);
        match c_opt {
            OptionChar::Some(c) => {
                if c == '\n' {
                    return i;
                }
                i = i + 1;
            }
            OptionChar::None => {
                return i;
            }
        }
    }
    i
}

/// Scans integer literal.
fn scan_integer(chars: &VecChar, start: usize, len: usize) -> (LexResult, usize) {
    let mut val: u64 = 0;
    let mut i: usize = start;

    while i < len {
        let c_opt: OptionChar = vec_char_get(chars, i);
        match c_opt {
            OptionChar::Some(c) => {
                if char_is_digit(c) {
                    val = val * 10 + char_digit_to_u64(c);
                    i = i + 1;
                } else {
                    return (LexResult::Token(Token::Integer(val)), i);
                }
            }
            OptionChar::None => {
                return (LexResult::Token(Token::Integer(val)), i);
            }
        }
    }
    (LexResult::Token(Token::Integer(val)), i)
}

/// Converts digit char to u64.
fn char_digit_to_u64(c: char) -> u64 {
    let c_val: u64 = c as u64;
    let zero_val: u64 = '0' as u64;
    c_val - zero_val
}

/// Scans identifier or keyword.
fn scan_identifier(chars: &VecChar, start: usize, len: usize) -> (LexResult, usize) {
    let mut s: String = string_new();
    let mut i: usize = start;

    while i < len {
        let c_opt: OptionChar = vec_char_get(chars, i);
        match c_opt {
            OptionChar::Some(c) => {
                if char_is_alphanumeric(c) {
                    string_push(&mut s, c);
                    i = i + 1;
                } else if c == '_' {
                    string_push(&mut s, c);
                    i = i + 1;
                } else {
                    return (LexResult::Token(keyword_or_identifier(s)), i);
                }
            }
            OptionChar::None => {
                return (LexResult::Token(keyword_or_identifier(s)), i);
            }
        }
    }
    (LexResult::Token(keyword_or_identifier(s)), i)
}

/// Converts string to keyword or identifier.
fn keyword_or_identifier(s: String) -> Token {
    let text: &str = string_as_str(&s);

    if str_eq(text, "fn") {
        return Token::Fn;
    }
    if str_eq(text, "let") {
        return Token::Let;
    }
    if str_eq(text, "if") {
        return Token::If;
    }
    if str_eq(text, "while") {
        return Token::While;
    }
    if str_eq(text, "return") {
        return Token::Return;
    }
    if str_eq(text, "enum") {
        return Token::Enum;
    }
    if str_eq(text, "match") {
        return Token::Match;
    }

    Token::Identifier(s)
}

/// Scans string literal.
fn scan_string(chars: &VecChar, start: usize, len: usize) -> (LexResult, usize) {
    let mut s: String = string_new();
    let mut i: usize = start + 1;

    while i < len {
        let c_opt: OptionChar = vec_char_get(chars, i);
        match c_opt {
            OptionChar::Some(c) => {
                if c == '"' {
                    return (LexResult::Token(Token::StringLiteral(s)), i + 1);
                }
                string_push(&mut s, c);
                i = i + 1;
            }
            OptionChar::None => {
                panic_exit("unterminated string literal");
            }
        }
    }
    panic_exit("unterminated string literal")
}

/// Scans symbol token.
fn scan_symbol(chars: &VecChar, i: usize) -> (LexResult, usize) {
    let c: char = unwrap_char(vec_char_get(chars, i));

    match c {
        '{' => (LexResult::Token(Token::LBrace), i + 1),
        '}' => (LexResult::Token(Token::RBrace), i + 1),
        '(' => (LexResult::Token(Token::LParen), i + 1),
        ')' => (LexResult::Token(Token::RParen), i + 1),
        ':' => (LexResult::Token(Token::Colon), i + 1),
        ';' => (LexResult::Token(Token::SemiColon), i + 1),
        ',' => (LexResult::Token(Token::Comma), i + 1),
        '+' => (LexResult::Token(Token::Plus), i + 1),
        '*' => (LexResult::Token(Token::Star), i + 1),
        '-' => scan_minus_or_arrow(chars, i),
        '=' => scan_assign_or_eq(chars, i),
        _ => panic_exit("unexpected character"),
    }
}

/// Scans '-' or '->'.
fn scan_minus_or_arrow(chars: &VecChar, i: usize) -> (LexResult, usize) {
    let next_opt: OptionChar = vec_char_get(chars, i + 1);
    match next_opt {
        OptionChar::Some(next) => {
            if next == '>' {
                return (LexResult::Token(Token::Arrow), i + 2);
            }
            (LexResult::Token(Token::Minus), i + 1)
        }
        OptionChar::None => (LexResult::Token(Token::Minus), i + 1),
    }
}

/// Scans '=' or '=='.
fn scan_assign_or_eq(chars: &VecChar, i: usize) -> (LexResult, usize) {
    let next_opt: OptionChar = vec_char_get(chars, i + 1);
    match next_opt {
        OptionChar::Some(next) => {
            if next == '=' {
                return (LexResult::Token(Token::Eq), i + 2);
            }
            (LexResult::Token(Token::Assign), i + 1)
        }
        OptionChar::None => (LexResult::Token(Token::Assign), i + 1),
    }
}

/// Unwraps OptionChar or panics.
fn unwrap_char(opt: OptionChar) -> char {
    match opt {
        OptionChar::Some(c) => c,
        OptionChar::None => panic_exit("unexpected end of input"),
    }
}

// === Test helpers (used by tests module) ===

/// Gets token or panics.
pub fn get_token(tokens: &VecToken, i: usize) -> Token {
    let opt: OptionToken = vec_token_get(tokens, i);
    match opt {
        OptionToken::Some(t) => t,
        OptionToken::None => panic_exit("token index out of bounds"),
    }
}

/// Creates string from &str.
pub fn make_string(s: &str) -> String {
    string_from_str(s)
}
