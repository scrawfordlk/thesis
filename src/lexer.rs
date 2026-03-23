use crate::runtime::{CharList, OptionChar, OptionToken, TokenList};
use crate::token::Token;

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

fn next_token(chars: &CharList, i: usize) -> (OptionToken, usize) {
    let c_opt: OptionChar = chars.get(i);
    match c_opt {
        OptionChar::Some(c) => {
            if c.is_whitespace() {
                (OptionToken::None, i + 1)
            } else if c == '/' {
                scan_slash_or_comment(chars, i)
            } else if c.is_ascii_digit() {
                scan_integer(chars, i)
            } else if c.is_alphabetic() || c == '_' {
                scan_identifier(chars, i)
            } else if c == '"' {
                scan_string(chars, i)
            } else {
                scan_symbol(c, chars, i)
            }
        }
        OptionChar::None => (OptionToken::None, i + 1),
    }
}

fn scan_slash_or_comment(chars: &CharList, i: usize) -> (OptionToken, usize) {
    let next_i: usize = i + 1;
    let next: OptionChar = chars.get(next_i);
    match next {
        OptionChar::Some(nc) => {
            if nc == '/' {
                let end_comment_i: usize = skip_line_comment(chars, next_i + 1);
                (OptionToken::None, end_comment_i)
            } else {
                (OptionToken::Some(Token::Slash), next_i)
            }
        }
        OptionChar::None => (OptionToken::Some(Token::Slash), next_i),
    }
}

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
                if digit.is_ascii_digit() {
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

fn scan_identifier(chars: &CharList, start_i: usize) -> (OptionToken, usize) {
    let mut s: String = String::new();
    let mut i: usize = start_i;
    let len: usize = chars.len();

    while i < len {
        let ch_opt: OptionChar = chars.get(i);
        match ch_opt {
            OptionChar::Some(ch) => {
                if ch.is_alphanumeric() || ch == '_' {
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

fn keyword_or_id(s: String) -> Token {
    match s.as_str() {
        "fn" => Token::Fn,
        "let" => Token::Let,
        "if" => Token::If,
        "while" => Token::While,
        "return" => Token::Return,
        "enum" => Token::Enum,
        "match" => Token::Match,
        _ => Token::Identifier(s),
    }
}

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
                eprintln!("Syntax error: unterminated string literal");
                std::process::exit(1);
            }
        }
    }
    eprintln!("Syntax error: unterminated string literal");
    std::process::exit(1);
}

fn scan_symbol(c: char, chars: &CharList, i: usize) -> (OptionToken, usize) {
    match c {
        '{' => (OptionToken::Some(Token::LBrace), i + 1),
        '}' => (OptionToken::Some(Token::RBrace), i + 1),
        '(' => (OptionToken::Some(Token::LParen), i + 1),
        ')' => (OptionToken::Some(Token::RParen), i + 1),
        ':' => (OptionToken::Some(Token::Colon), i + 1),
        ';' => (OptionToken::Some(Token::SemiColon), i + 1),
        ',' => (OptionToken::Some(Token::Comma), i + 1),
        '+' => (OptionToken::Some(Token::Plus), i + 1),
        '*' => (OptionToken::Some(Token::Star), i + 1),
        '-' => {
            // Check for ->
            let next_i: usize = i + 1;
            match chars.get(next_i) {
                OptionChar::Some('>') => (OptionToken::Some(Token::Arrow), next_i + 1),
                _ => (OptionToken::Some(Token::Minus), next_i),
            }
        }
        '=' => {
            // Check for ==
            let next_i: usize = i + 1;
            match chars.get(next_i) {
                OptionChar::Some('=') => (OptionToken::Some(Token::Eq), next_i + 1),
                _ => (OptionToken::Some(Token::Assign), next_i),
            }
        }
        _ => {
            // Unknown character - print error and exit
            eprintln!("Syntax error: unexpected character '{}'", c);
            std::process::exit(1);
        }
    }
}
