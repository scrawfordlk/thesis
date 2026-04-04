#![allow(clippy::assign_op_pattern)]

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
    Type(Type),             // "usize", ...
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

enum Type {
    Usize, // "usize"
    U8,    // "u8"
    Char,  // "char"
    Str,   // "str"
}

enum Literal {
    Integer(usize),
    Character(char),
    String(String),
}

// -----------------------------------------------------------------
// ------------------------- Library -------------------------------
// -----------------------------------------------------------------

// -------------------------- char ---------------------------------

enum CharOption {
    Some(char),
    None,
}

fn unwrap_char(char_opt: CharOption) -> char {
    match char_opt {
        CharOption::Some(c) => c,
        CharOption::None => panic!("unwrap failed"),
    }
}

// ------------------------- String -------------------------------

enum String {
    // start, length, capacity
    String(*mut u8, usize, usize),
}

/// Get the pointer to the start of the string data.
fn string_ptr(String::String(ptr, _, _): &String) -> *mut u8 {
    *ptr
}

/// Get the length of the string.
fn string_len(String::String(_, len, _): &String) -> usize {
    *len
}

/// Get the capacity of the string.
fn string_capacity(String::String(_, _, capacity): &String) -> usize {
    *capacity
}

/// Create a new empty string.
fn string_new() -> String {
    let ptr: *mut u8 = alloc(1, size_of::<u8>());
    String::String(ptr, 0, 1)
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
