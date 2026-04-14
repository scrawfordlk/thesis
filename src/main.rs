#![allow(clippy::assign_op_pattern)]
#![allow(while_true)]

fn main() {
    let mut str: String = string_new();
    string_push_str(&mut str, "Hello, World!");
    string_push(&mut str, '\n');

    for i in 0..string_len(&str) {
        print!("{}", unwrap_char(string_get(&str, i)));
    }
}

// -----------------------------------------------------------------
// ------------------------- Compiler ------------------------------
// -----------------------------------------------------------------

// ---------------------- Lexical Analysis -------------------------

enum Token {
    Fn,                     // "fn"
    Enum,                   // "enum"
    Let,                    // "let"
    If,                     // "if"
    Else,                   // "else"
    While,                  // "while"
    Return,                 // "return"
    Match,                  // "match"
    As,                     // "as"
    Mut,                    // "mut"
    Ampersand,              // "&"
    LBrace,                 // "{"
    RBrace,                 // "}"
    LParen,                 // "("
    RParen,                 // ")"
    Colon,                  // ":"
    DoubleColon,            // "::"
    SemiColon,              // ";"
    Comma,                  // ","
    Assign,                 // "="
    Comparison(Comparison), // "==", ...
    ArmArrow,               // "=>"
    Plus,                   // "+"
    Minus,                  // "-"
    Star,                   // "*"
    Slash,                  // "/"
    Remainder,              // "%"
    Usize,                  // "usize"
    U8,                     // "u8"
    Char,                   // "char"
    Str,                    // "str"
    TypeArrow,              // "->"
    SizeOf(usize),          // TODO: probably unnecessary
    Identifier(String),
    Literal(Literal),
    Eof,
}

enum Comparison {
    Eq,  // "=="
    Neq, // "!="
    Gt,  // ">"
    Lt,  // "<"
    Geq, // ">="
    Leq, // "<="
}

enum Literal {
    Integer(usize),
    Character(char),
    String(String),
}

enum Lexer {
    // source file, current token
    Lexer(SourceFile, Token),
}

enum SourceFile {
    // content, current character index, current location
    SourceFile(String, usize, SourceLocation),
}

enum SourceLocation {
    // line, column
    Coords(usize, usize),
}

fn lexer_sourcefile(lexer: &Lexer) -> &SourceFile {
    let Lexer::Lexer(source, _): &Lexer = lexer;
    source
}

/// Get the current source location.
fn lexer_location(lexer: &Lexer) -> &SourceLocation {
    let SourceFile::SourceFile(_, _, location): &SourceFile = lexer_sourcefile(lexer);
    location
}

/// Peek at the current character without consuming it.
fn lexer_peek_char(lexer: &Lexer) -> CharOption {
    let SourceFile::SourceFile(content, index, _): &SourceFile = lexer_sourcefile(lexer);
    string_get(content, *index)
}

/// Consume and return the current character.
fn lexer_consume_char(lexer: &mut Lexer) -> CharOption {
    let Lexer::Lexer(source, _): &mut Lexer = lexer;
    let SourceFile::SourceFile(content, index, location): &mut SourceFile = source;

    let current: CharOption = string_get(content, *index);
    *index = *index + 1;

    match current {
        CharOption::Some(c) => {
            let SourceLocation::Coords(line, col): &mut SourceLocation = location;
            if c == '\n' {
                *line = *line + 1;
                *col = 1;
            } else {
                *col = *col + 1;
            }
        }
        CharOption::None => {}
    }
    current
}

/// Consume the next character, erroring if it doesn't match expected.
fn lexer_expect_char(lexer: &mut Lexer, expected: char) {
    match lexer_consume_char(lexer) {
        CharOption::Some(c) => {
            if c != expected {
                lexer_error(lexer, "unexpected character");
            }
        }
        CharOption::None => lexer_error(lexer, "unexpected end of input"),
    }
}

// ---------------------- Lexer ----------------------

/// Consume and return the next token.
fn next_token(lexer: &mut Lexer) -> Token {
    skip_whitespace(lexer);

    match lexer_peek_char(lexer) {
        CharOption::Some(c) => {
            if is_alpha(c) {
                let ident: String = scan_identifier(lexer);
                identifier_to_token(ident)
            } else if is_digit(c) {
                let value: usize = scan_integer(lexer);
                Token::Literal(Literal::Integer(value))
            } else if c == '\'' {
                let ch: char = scan_char_literal(lexer);
                Token::Literal(Literal::Character(ch))
            } else if c == '"' {
                let s: String = scan_string_literal(lexer);
                Token::Literal(Literal::String(s))
            } else {
                scan_symbol(lexer)
            }
        }
        CharOption::None => Token::Eof,
    }
}

/// Scan an identifier or keyword.
fn scan_identifier(lexer: &mut Lexer) -> String {
    let mut ident: String = string_new();
    while true {
        match lexer_peek_char(lexer) {
            CharOption::Some(c) => {
                if is_alphanumeric(c) {
                    lexer_consume_char(lexer);
                    string_push(&mut ident, c);
                } else {
                    return ident;
                }
            }
            CharOption::None => return ident,
        }
    }
    ident // satisfy compiler
}

/// Convert an identifier to a keyword token if applicable.
fn identifier_to_token(ident: String) -> Token {
    if string_eq(&ident, &string_from_str("fn")) {
        Token::Fn
    } else if string_eq(&ident, &string_from_str("enum")) {
        Token::Enum
    } else if string_eq(&ident, &string_from_str("let")) {
        Token::Let
    } else if string_eq(&ident, &string_from_str("if")) {
        Token::If
    } else if string_eq(&ident, &string_from_str("else")) {
        Token::Else
    } else if string_eq(&ident, &string_from_str("while")) {
        Token::While
    } else if string_eq(&ident, &string_from_str("return")) {
        Token::Return
    } else if string_eq(&ident, &string_from_str("match")) {
        Token::Match
    } else if string_eq(&ident, &string_from_str("as")) {
        Token::As
    } else if string_eq(&ident, &string_from_str("mut")) {
        Token::Mut
    } else if string_eq(&ident, &string_from_str("usize")) {
        Token::Usize
    } else if string_eq(&ident, &string_from_str("u8")) {
        Token::U8
    } else if string_eq(&ident, &string_from_str("char")) {
        Token::Char
    } else if string_eq(&ident, &string_from_str("str")) {
        Token::Str
    } else {
        Token::Identifier(ident)
    }
}

/// TODO: check for too large integer
fn scan_integer(lexer: &mut Lexer) -> usize {
    let mut value: usize = 0;
    while true {
        match lexer_peek_char(lexer) {
            CharOption::Some(c) => {
                if is_digit(c) {
                    value = value * 10 + (c as usize) - ('0' as usize);
                    lexer_consume_char(lexer);
                } else {
                    return value;
                }
            }
            CharOption::None => return value,
        }
    }
    value // satisfy compiler
}

fn scan_char_literal(lexer: &mut Lexer) -> char {
    lexer_expect_char(lexer, '\'');
    let c: char = match lexer_consume_char(lexer) {
        CharOption::Some('\\') => scan_escape_char(lexer),
        CharOption::Some(ch) => ch,
        CharOption::None => lexer_error(lexer, "unexpected end of char literal"),
    };
    lexer_expect_char(lexer, '\'');
    c
}

fn scan_string_literal(lexer: &mut Lexer) -> String {
    lexer_expect_char(lexer, '"');
    let mut s: String = string_new();
    while true {
        match lexer_consume_char(lexer) {
            CharOption::Some('"') => return s,
            CharOption::Some('\\') => string_push(&mut s, scan_escape_char(lexer)),
            CharOption::Some(c) => string_push(&mut s, c),
            CharOption::None => lexer_error(lexer, "unexpected end of string literal"),
        }
    }
    s // satisfy compiler
}

/// Scan an escape sequence after backslash.
fn scan_escape_char(lexer: &mut Lexer) -> char {
    match lexer_consume_char(lexer) {
        CharOption::Some('n') => '\n',
        CharOption::Some('t') => '\t',
        CharOption::Some('r') => '\r',
        CharOption::Some('0') => '\0',
        CharOption::Some(c) => c,
        CharOption::None => lexer_error(lexer, "unexpected end of escape sequence"),
    }
}

fn scan_symbol(lexer: &mut Lexer) -> Token {
    match unwrap_char(lexer_consume_char(lexer)) {
        '{' => Token::LBrace,
        '}' => Token::RBrace,
        '(' => Token::LParen,
        ')' => Token::RParen,
        ';' => Token::SemiColon,
        ',' => Token::Comma,
        '+' => Token::Plus,
        '*' => Token::Star,
        '/' => scan_slash(lexer),
        '%' => Token::Remainder,
        '&' => Token::Ampersand,
        ':' => scan_colon(lexer),
        '=' => scan_equals(lexer),
        '-' => scan_minus(lexer),
        '!' => scan_bang(lexer),
        '<' => scan_less(lexer),
        '>' => scan_greater(lexer),
        _ => lexer_error(lexer, "unexpected character"),
    }
}

fn scan_slash(lexer: &mut Lexer) -> Token {
    match lexer_peek_char(lexer) {
        CharOption::Some('/') => {
            lexer_consume_char(lexer);
            skip_line_comment(lexer);
            next_token(lexer)
        }
        _ => Token::Slash,
    }
}

fn scan_colon(lexer: &mut Lexer) -> Token {
    match lexer_peek_char(lexer) {
        CharOption::Some(':') => {
            lexer_consume_char(lexer);
            Token::DoubleColon
        }
        _ => Token::Colon,
    }
}

fn scan_equals(lexer: &mut Lexer) -> Token {
    match lexer_peek_char(lexer) {
        CharOption::Some('=') => {
            lexer_consume_char(lexer);
            Token::Comparison(Comparison::Eq)
        }
        CharOption::Some('>') => {
            lexer_consume_char(lexer);
            Token::ArmArrow
        }
        _ => Token::Assign,
    }
}

fn scan_minus(lexer: &mut Lexer) -> Token {
    match lexer_peek_char(lexer) {
        CharOption::Some('>') => {
            lexer_consume_char(lexer);
            Token::TypeArrow
        }
        _ => Token::Minus,
    }
}

fn scan_bang(lexer: &mut Lexer) -> Token {
    match lexer_peek_char(lexer) {
        CharOption::Some('=') => {
            lexer_consume_char(lexer);
            Token::Comparison(Comparison::Neq)
        }
        _ => lexer_error(lexer, "expected '=' after '!'"),
    }
}

fn scan_less(lexer: &mut Lexer) -> Token {
    match lexer_peek_char(lexer) {
        CharOption::Some('=') => {
            lexer_consume_char(lexer);
            Token::Comparison(Comparison::Leq)
        }
        _ => Token::Comparison(Comparison::Lt),
    }
}

fn scan_greater(lexer: &mut Lexer) -> Token {
    match lexer_peek_char(lexer) {
        CharOption::Some('=') => {
            lexer_consume_char(lexer);
            Token::Comparison(Comparison::Geq)
        }
        _ => Token::Comparison(Comparison::Gt),
    }
}

fn skip_whitespace(lexer: &mut Lexer) {
    while true {
        match lexer_peek_char(lexer) {
            CharOption::Some(c) => {
                if is_whitespace(c) {
                    lexer_consume_char(lexer);
                } else {
                    return;
                }
            }
            CharOption::None => return,
        }
    }
}

fn skip_line_comment(lexer: &mut Lexer) {
    while true {
        match lexer_consume_char(lexer) {
            CharOption::Some('\n') => return,
            CharOption::Some(_) => (),
            CharOption::None => return,
        }
    }
}

// -----------------------------------------------------------------
// ------------------------- Library -------------------------------
// -----------------------------------------------------------------

// -------------------------- error --------------------------------

/// Report an error message with source location and exit.
/// TODO: not subset-conform
fn lexer_error(lexer: &Lexer, message: &str) -> ! {
    let SourceLocation::Coords(line, col): &SourceLocation = lexer_location(lexer);
    eprintln!("error at {}:{}: {}", line, col, message);
    std::process::exit(1);
}

// -------------------------- bool ---------------------------------

/// Logical AND of two booleans.
fn and(a: bool, b: bool) -> bool {
    if a { b } else { false }
}

/// Logical OR of two booleans.
fn or(a: bool, b: bool) -> bool {
    if a { true } else { b }
}

// -------------------------- char ---------------------------------

enum CharOption {
    Some(char),
    None,
}

/// Returns the value wrapped in Some.
/// If the option is None, end the program.
fn unwrap_char(char_opt: CharOption) -> char {
    match char_opt {
        CharOption::Some(c) => c,
        CharOption::None => panic!("unwrap failed"),
    }
}

fn is_whitespace(c: char) -> bool {
    or(or(c == ' ', c == '\t'), or(c == '\n', c == '\r'))
}

fn is_digit(c: char) -> bool {
    and(c >= '0', c <= '9')
}

fn is_alpha(c: char) -> bool {
    let lower: bool = and(c >= 'a', c <= 'z');
    let upper: bool = and(c >= 'A', c <= 'Z');
    or(or(lower, upper), c == '_')
}

fn is_alphanumeric(c: char) -> bool {
    or(is_alpha(c), is_digit(c))
}

// ------------------------- String -------------------------------

enum String {
    // start, length, capacity
    String(*mut u8, usize, usize),
}

/// Get the pointer to the start of the string data.
fn string_ptr(string: &String) -> *mut u8 {
    let String::String(ptr, _, _): &String = string;
    *ptr
}

/// Get the length of the string.
fn string_len(string: &String) -> usize {
    let String::String(_, len, _): &String = string;
    *len
}

/// Get the capacity of the string.
fn string_capacity(string: &String) -> usize {
    let String::String(_, _, capacity): &String = string;
    *capacity
}

/// Create a new empty string.
fn string_new() -> String {
    string_with_capacity(10)
}

/// Create a new string with the specified capacity
fn string_with_capacity(initial_capacity: usize) -> String {
    let ptr: *mut u8 = alloc(initial_capacity, std::mem::size_of::<u8>());
    String::String(ptr, 0, initial_capacity)
}

/// Get the character at the given index.
fn string_get(string: &String, index: usize) -> CharOption {
    if index >= string_len(string) {
        CharOption::None
    } else {
        let ptr: *mut u8 = ptr_add(string_ptr(string), index);
        unsafe { CharOption::Some(*ptr as char) }
    }
}

/// Append a string slice to the string.
fn string_push_str(string: &mut String, str: &str) {
    let str_len: usize = str.len();
    string_accomodate_extra_space(string, str_len);

    let str_ptr: *mut u8 = str.as_ptr() as *mut u8;

    let String::String(string_ptr, len, _): &mut String = string;
    unsafe {
        let string_end: *mut u8 = ptr_add(*string_ptr, *len);
        memcopy(string_end, str_ptr, str_len)
    };

    *len = *len + str_len;
}

/// Append a character to the string.
fn string_push(string: &mut String, character: char) {
    string_accomodate_extra_space(string, 1);
    let String::String(ptr, len, _): &mut String = string;
    unsafe {
        *ptr_add(*ptr, *len) = character as u8;
    }
    *len = *len + 1;
}

fn string_clone(string: &String) -> String {
    let len: usize = string_len(string);

    let mut clone: String = string_with_capacity(len);
    let mut i: usize = 0;
    while i < len {
        let character: char = unwrap_char(string_get(string, i));
        string_push(&mut clone, character);
        i = i + 1;
    }
    clone
}

/// Ensure the string has space for additional bytes.
fn string_accomodate_extra_space(string: &mut String, space: usize) {
    let len: usize = string_len(string);
    let capacity: usize = string_capacity(string);
    if capacity < len + space {
        let String::String(string_ptr, len, capacity): &mut String = string;
        *capacity = *capacity * 2;
        let new_ptr: *mut u8 = alloc(*capacity, 1);
        unsafe { memcopy(new_ptr, *string_ptr, *len) };
        *string_ptr = new_ptr;
        string_accomodate_extra_space(string, space);
    }
}

/// Create a string from a string slice.
fn string_from_str(str: &str) -> String {
    let mut s: String = string_new();
    string_push_str(&mut s, str);
    s
}

/// Check if two strings are equal.
fn string_eq(s1: &String, s2: &String) -> bool {
    let len: usize = string_len(s1);
    if len != string_len(s2) {
        return false;
    }

    let mut i: usize = 0;
    while i < len {
        let c1: char = unwrap_char(string_get(s1, i));
        let c2: char = unwrap_char(string_get(s2, i));
        if c1 != c2 {
            return false;
        }

        i = i + 1;
    }

    true
}

// ------------------------- Memory -------------------------------

/// Copy n bytes from src to dest.
///
/// It must hold: forall 0 <= i < n, dest[i] can be written
/// and src[i] can be read safely.
unsafe fn memcopy(dest: *mut u8, src: *mut u8, n: usize) {
    let mut i: usize = 0;
    while i < n {
        unsafe {
            *ptr_add(dest, i) = *ptr_add(src, i);
        }
        i = i + 1;
    }
}

/// Increment a pointer by n. This is standard arithmetic, not pointer arithmetic.
fn ptr_add(ptr: *mut u8, n: usize) -> *mut u8 {
    (ptr as usize + n) as *mut u8
}

/// Heap-allocate memory for the given size and alignment.
///
/// The caller should cast the returned pointer to the desired type.
fn alloc(size: usize, align: usize) -> *mut u8 {
    // TODO:
    // is power of 2
    // isize::MAX as usize + 1
    unsafe { std::alloc::alloc_zeroed(std::alloc::Layout::from_size_align_unchecked(size, align)) }
}

// -----------------------------------------------------------------
// -------------------------- Tests --------------------------------
// -----------------------------------------------------------------

include!("tests.rs");
