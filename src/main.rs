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
    Fn,            // "fn"
    Enum,          // "enum"
    Let,           // "let"
    If,            // "if"
    Else,          // "else"
    While,         // "while"
    Return,        // "return"
    Match,         // "match"
    As,            // "as"
    Unsafe,        // "unsafe"
    Mut,           // "mut"
    Ampersand,     // "&"
    LBrace,        // "{"
    RBrace,        // "}"
    LParen,        // "("
    RParen,        // ")"
    Colon,         // ":"
    DoubleColon,   // "::"
    SemiColon,     // ";"
    Comma,         // ","
    Assign,        // "="
    Eq,            // "=="
    Neq,           // "!="
    Bang,          // "!"
    Gt,            // ">"
    Lt,            // "<"
    Geq,           // ">="
    Leq,           // "<="
    ArmArrow,      // "=>"
    Plus,          // "+"
    Minus,         // "-"
    Star,          // "*"
    Slash,         // "/"
    Remainder,     // "%"
    Usize,         // "usize"
    U8,            // "u8"
    Bool,          // "bool"
    Char,          // "char"
    Str,           // "str"
    TypeArrow,     // "->"
    Boolean(bool), // "true", "false"
    Integer(usize),
    String(String),
    Character(char),
    Identifier(String),
    SizeOf(usize), // TODO: probably unnecessary
    Eof,
}

fn token_clone(token: &Token) -> Token {
    match token {
        Token::Unsafe => Token::Unsafe,
        Token::Fn => Token::Fn,
        Token::Enum => Token::Enum,
        Token::Let => Token::Let,
        Token::If => Token::If,
        Token::Else => Token::Else,
        Token::While => Token::While,
        Token::Return => Token::Return,
        Token::Match => Token::Match,
        Token::As => Token::As,
        Token::Mut => Token::Mut,
        Token::Ampersand => Token::Ampersand,
        Token::LBrace => Token::LBrace,
        Token::RBrace => Token::RBrace,
        Token::LParen => Token::LParen,
        Token::RParen => Token::RParen,
        Token::Colon => Token::Colon,
        Token::DoubleColon => Token::DoubleColon,
        Token::SemiColon => Token::SemiColon,
        Token::Comma => Token::Comma,
        Token::Assign => Token::Assign,
        Token::Eq => Token::Eq,
        Token::Neq => Token::Neq,
        Token::Bang => Token::Bang,
        Token::Gt => Token::Gt,
        Token::Lt => Token::Lt,
        Token::Geq => Token::Geq,
        Token::Leq => Token::Leq,
        Token::ArmArrow => Token::ArmArrow,
        Token::Plus => Token::Plus,
        Token::Minus => Token::Minus,
        Token::Star => Token::Star,
        Token::Slash => Token::Slash,
        Token::Remainder => Token::Remainder,
        Token::Usize => Token::Usize,
        Token::U8 => Token::U8,
        Token::Bool => Token::Bool,
        Token::Char => Token::Char,
        Token::Str => Token::Str,
        Token::TypeArrow => Token::TypeArrow,
        Token::Identifier(value) => Token::Identifier(string_clone(value)),
        Token::Integer(value) => Token::Integer(*value),
        Token::String(value) => Token::String(string_clone(value)),
        Token::Character(value) => Token::Character(*value),
        Token::Boolean(value) => Token::Boolean(*value),
        Token::SizeOf(value) => Token::SizeOf(*value),
        Token::Eof => Token::Eof,
    }
}
/// A type that encapsulates the state of the lexer
enum Lexer {
    // source file, current token
    Lexer(SourceFile, Token),
}

/// A type that manages the source file
enum SourceFile {
    // content, current character index, current location
    SourceFile(String, usize, SourceLocation),
}

/// A type that tracks the location in the source code
enum SourceLocation {
    // line, column
    Coords(usize, usize),
}

fn lexer_new(source: String) -> Lexer {
    let source_file: SourceFile = SourceFile::SourceFile(source, 0, SourceLocation::Coords(1, 1));
    let mut lexer: Lexer = Lexer::Lexer(source_file, Token::Eof);
    lexer_next_token(&mut lexer);
    lexer
}

fn lexer_sourcefile(lexer: &Lexer) -> &SourceFile {
    let Lexer::Lexer(source, _): &Lexer = lexer;
    source
}

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
fn lexer_next_token(lexer: &mut Lexer) -> Token {
    skip_whitespace(lexer);

    let token: Token = match lexer_peek_char(lexer) {
        CharOption::Some(c) => {
            if is_alpha(c) {
                let ident: String = scan_identifier(lexer);
                identifier_to_token(ident)
            } else if is_digit(c) {
                let value: usize = scan_integer(lexer);
                Token::Integer(value)
            } else if c == '\'' {
                let ch: char = scan_char_literal(lexer);
                Token::Character(ch)
            } else if c == '"' {
                let s: String = scan_string_literal(lexer);
                Token::String(s)
            } else {
                scan_symbol(lexer)
            }
        }
        CharOption::None => Token::Eof,
    };

    let returned_token: Token = token_clone(&token);
    let Lexer::Lexer(_, current_token) = lexer;
    *current_token = token;
    returned_token
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
    } else if string_eq(&ident, &string_from_str("unsafe")) {
        Token::Unsafe
    } else if string_eq(&ident, &string_from_str("mut")) {
        Token::Mut
    } else if string_eq(&ident, &string_from_str("usize")) {
        Token::Usize
    } else if string_eq(&ident, &string_from_str("u8")) {
        Token::U8
    } else if string_eq(&ident, &string_from_str("bool")) {
        Token::Bool
    } else if string_eq(&ident, &string_from_str("char")) {
        Token::Char
    } else if string_eq(&ident, &string_from_str("str")) {
        Token::Str
    } else if string_eq(&ident, &string_from_str("true")) {
        Token::Boolean(true)
    } else if string_eq(&ident, &string_from_str("false")) {
        Token::Boolean(false)
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
            lexer_next_token(lexer)
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
            Token::Eq
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
            Token::Neq
        }
        _ => Token::Bang,
    }
}

fn scan_less(lexer: &mut Lexer) -> Token {
    match lexer_peek_char(lexer) {
        CharOption::Some('=') => {
            lexer_consume_char(lexer);
            Token::Leq
        }
        _ => Token::Lt,
    }
}

fn scan_greater(lexer: &mut Lexer) -> Token {
    match lexer_peek_char(lexer) {
        CharOption::Some('=') => {
            lexer_consume_char(lexer);
            Token::Geq
        }
        _ => Token::Gt,
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

// -------------------------- Parser -------------------------------

/// Data structure that manages a global symbol table and (multiple) local symbol tables.
enum SymTable {
    Table(GlobalSymTable, LocalSymTableStack),
}

/// Create an empty symbol table.
fn symtable_new() -> SymTable {
    let global: GlobalSymTable = GlobalSymTable::Nil;
    let local: LocalSymTableStack = LocalSymTableStack::Nil;
    SymTable::Table(global, local)
}

/// Check whether a symbol exists in local scopes or globals.
fn symtable_contains(symtable: &SymTable, name: &String) -> bool {
    let SymTable::Table(global, local): &SymTable = symtable;
    or(
        local_symtable_stack_contains(local, name),
        global_symtable_contains(global, name),
    )
}

/// Enter a new local scope.
fn symtable_enter_scope(symtable: &mut SymTable) {
    let SymTable::Table(_, local): &mut SymTable = symtable;
    local_symtable_stack_push(local);
}

/// Leave the current local scope.
fn symtable_leave_scope(symtable: &mut SymTable) -> bool {
    let SymTable::Table(_, local_stack): &mut SymTable = symtable;
    local_symtable_stack_pop(local_stack)
}

/// Insert a function into the global table.
fn symtable_insert_function(
    symtable: &mut SymTable,
    name: String,
    parameter_types: Types,
    return_type: Type,
) -> bool {
    let SymTable::Table(global, _) = symtable;
    global_symtable_insert_function(global, name, parameter_types, return_type)
}

/// Insert an enum into the global table.
fn symtable_insert_enum(symtable: &mut SymTable, name: String, variants: Types) -> bool {
    let SymTable::Table(global, _) = symtable;
    global_symtable_insert_enum(global, name, variants)
}

/// Insert a variable into the current local scope.
fn symtable_insert_variable(
    symtable: &mut SymTable,
    name: String,
    variable_type: Type,
    mutable: bool,
) {
    let SymTable::Table(_, local_stack): &mut SymTable = symtable;
    match local_stack {
        LocalSymTableStack::Cons(local, _) => {
            local_symtable_insert_variable(local, name, variable_type, mutable)
        }
        LocalSymTableStack::Nil => (),
    };
}

/// Global symbol table represented as a linked cons list.
enum GlobalSymTable {
    Cons(SymTableEntry, GlobalSymTableBox),
    Nil,
}

/// Prepend an entry to the global table.
fn global_symtable_prepend(symtable: &mut GlobalSymTable, entry: SymTableEntry) {
    let old_copy: GlobalSymTable = global_symtable_clone(symtable);
    let tail: GlobalSymTableBox = global_symtable_box_new(old_copy);
    *symtable = GlobalSymTable::Cons(entry, tail);
}

/// Check whether a name exists in the global table.
fn global_symtable_contains(symtable: &GlobalSymTable, name: &String) -> bool {
    match symtable {
        GlobalSymTable::Cons(head, tail) => {
            let entry_name: &String = symtable_entry_name(&head);
            let matches: bool = string_eq(entry_name, name);
            or(
                matches,
                global_symtable_contains(global_symtable_box_deref(tail), name),
            )
        }
        GlobalSymTable::Nil => false,
    }
}

/// Insert a function entry into globals, returning false on duplicate name.
fn global_symtable_insert_function(
    symtable: &mut GlobalSymTable,
    name: String,
    parameter_types: Types,
    return_type: Type,
) -> bool {
    if global_symtable_contains(symtable, &name) {
        return false;
    }

    let entry: SymTableEntry = SymTableEntry::Function(name, parameter_types, return_type);
    global_symtable_prepend(symtable, entry);
    true
}

/// Insert an enum entry into globals, returning false on duplicate name.
fn global_symtable_insert_enum(
    symtable: &mut GlobalSymTable,
    name: String,
    variants: Types,
) -> bool {
    if global_symtable_contains(symtable, &name) {
        return false;
    }

    let entry: SymTableEntry = SymTableEntry::Enum(name, variants);
    global_symtable_prepend(symtable, entry);
    true
}

/// Stack of local scopes represented as a linked cons list.
enum LocalSymTableStack {
    Cons(LocalSymTable, LocalSymTableStackBox),
    Nil,
}

/// Push a new empty local scope onto the stack.
fn local_symtable_stack_push(stack: &mut LocalSymTableStack) {
    let old_copy: LocalSymTableStack = local_symtable_stack_clone(stack);
    let tail: LocalSymTableStackBox = local_symtable_stack_box_new(old_copy);
    *stack = LocalSymTableStack::Cons(LocalSymTable::Nil, tail);
}

/// Pop the top local scope from the stack.
fn local_symtable_stack_pop(stack: &mut LocalSymTableStack) -> bool {
    match stack {
        LocalSymTableStack::Cons(_, tail) => {
            *stack = local_symtable_stack_clone(local_symtable_stack_box_deref(tail));
            true
        }
        LocalSymTableStack::Nil => false,
    }
}

/// Check whether a name exists in any local scope.
fn local_symtable_stack_contains(stack: &LocalSymTableStack, name: &String) -> bool {
    match stack {
        LocalSymTableStack::Cons(local, tail) => or(
            local_symtable_contains(local, name),
            local_symtable_stack_contains(local_symtable_stack_box_deref(tail), name),
        ),
        LocalSymTableStack::Nil => false,
    }
}

/// Single local scope represented as a linked cons list.
enum LocalSymTable {
    Cons(SymTableEntry, LocalSymTableBox),
    Nil,
}

/// Prepend an entry to a local scope.
fn local_symtable_prepend(symtable: &mut LocalSymTable, entry: SymTableEntry) {
    let old_copy: LocalSymTable = local_symtable_clone(symtable);
    let tail: LocalSymTableBox = local_symtable_box_new(old_copy);
    *symtable = LocalSymTable::Cons(entry, tail);
}

/// Check whether a name exists in a local scope.
fn local_symtable_contains(symtable: &LocalSymTable, name: &String) -> bool {
    match symtable {
        LocalSymTable::Cons(head, tail) => {
            let entry_name: &String = symtable_entry_name(head);
            let matches: bool = string_eq(entry_name, name);
            or(
                matches,
                local_symtable_contains(local_symtable_box_deref(tail), name),
            )
        }
        LocalSymTable::Nil => false,
    }
}

/// Insert a variable entry into a single local scope.
fn local_symtable_insert_variable(
    symtable: &mut LocalSymTable,
    name: String,
    variable_type: Type,
    mutable: bool,
) {
    let entry: SymTableEntry = SymTableEntry::Variable(name, variable_type, mutable);
    local_symtable_prepend(symtable, entry);
}

/// Symbol table entry for functions, enums, and variables.
enum SymTableEntry {
    // name, parameter types, return type
    Function(String, Types, Type),
    // name, variants
    Enum(String, Types),
    // name, type, mutable
    Variable(String, Type, bool),
}

/// Get the name associated with a symbol table entry.
fn symtable_entry_name(entry: &SymTableEntry) -> &String {
    match entry {
        SymTableEntry::Function(name, _, _) => name,
        SymTableEntry::Enum(name, _) => name,
        SymTableEntry::Variable(name, _, _) => name,
    }
}

enum Type {
    U8,
    Usize,
    Bool,
    Char,
    Unit, // ()
    Never,
    Custom(String),
}

fn type_clone(t: &Type) -> Type {
    match t {
        Type::U8 => Type::U8,
        Type::Usize => Type::Usize,
        Type::Bool => Type::Bool,
        Type::Char => Type::Char,
        Type::Unit => Type::Unit,
        Type::Never => Type::Never,
        Type::Custom(name) => Type::Custom(string_clone(name)),
    }
}

enum Types {
    Cons(Type, TypesBox),
    Nil,
}

/// Clone a Types linked list.
fn types_clone(types: &Types) -> Types {
    match types {
        Types::Nil => Types::Nil,
        Types::Cons(head, tail) => Types::Cons(type_clone(head), types_box_clone(tail)),
    }
}

fn symtable_entry_clone(entry: &SymTableEntry) -> SymTableEntry {
    match entry {
        SymTableEntry::Function(name, parameter_types, return_type) => SymTableEntry::Function(
            string_clone(name),
            types_clone(parameter_types),
            type_clone(return_type),
        ),
        SymTableEntry::Enum(name, variants) => {
            SymTableEntry::Enum(string_clone(name), types_clone(variants))
        }
        SymTableEntry::Variable(name, variable_type, mutable) => {
            SymTableEntry::Variable(string_clone(name), type_clone(variable_type), *mutable)
        }
    }
}

fn global_symtable_clone(symtable: &GlobalSymTable) -> GlobalSymTable {
    match symtable {
        GlobalSymTable::Nil => GlobalSymTable::Nil,
        GlobalSymTable::Cons(head, tail) => {
            GlobalSymTable::Cons(symtable_entry_clone(head), global_symtable_box_clone(tail))
        }
    }
}

fn local_symtable_clone(symtable: &LocalSymTable) -> LocalSymTable {
    match symtable {
        LocalSymTable::Nil => LocalSymTable::Nil,
        LocalSymTable::Cons(head, tail) => {
            LocalSymTable::Cons(symtable_entry_clone(head), local_symtable_box_clone(tail))
        }
    }
}

fn local_symtable_stack_clone(stack: &LocalSymTableStack) -> LocalSymTableStack {
    match stack {
        LocalSymTableStack::Nil => LocalSymTableStack::Nil,
        LocalSymTableStack::Cons(local, tail) => LocalSymTableStack::Cons(
            local_symtable_clone(local),
            local_symtable_stack_box_clone(tail),
        ),
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
    a as u8 + b as u8 == 2
}

/// Logical OR of two booleans.
fn or(a: bool, b: bool) -> bool {
    a as u8 + b as u8 > 0
}

/// Logical NOT of one boolean.
fn not(a: bool) -> bool {
    a as u8 == 0
}

// -------------------------- char ---------------------------------

/// Option type for the type char
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

// ------------------------ Pointers ------------------------------

/// Box-like type that is a pointer to an owned heap-allocated GlobalSymTable.
enum GlobalSymTableBox {
    Ptr(*mut GlobalSymTable),
}

/// Allocate and box a GlobalSymTable value on the heap.
fn global_symtable_box_new(symtable: GlobalSymTable) -> GlobalSymTableBox {
    let ptr_u8: *mut u8 = alloc(
        std::mem::size_of::<GlobalSymTable>(),
        std::mem::size_of::<usize>(),
    );
    let ptr: *mut GlobalSymTable = ptr_u8 as *mut GlobalSymTable;
    unsafe { *ptr = symtable };
    GlobalSymTableBox::Ptr(ptr)
}

/// Dereference a GlobalSymTable box.
fn global_symtable_box_deref(ptr_wrap: &GlobalSymTableBox) -> &GlobalSymTable {
    let GlobalSymTableBox::Ptr(ptr): &GlobalSymTableBox = ptr_wrap;
    unsafe { &**ptr }
}

/// Clone a GlobalSymTable box and its heap-owned value.
fn global_symtable_box_clone(ptr: &GlobalSymTableBox) -> GlobalSymTableBox {
    let cloned: GlobalSymTable = global_symtable_clone(global_symtable_box_deref(ptr));
    global_symtable_box_new(cloned)
}

/// Box-like type that is a pointer to an owned heap-allocated LocalSymTableStack.
enum LocalSymTableStackBox {
    Ptr(*mut LocalSymTableStack),
}

/// Allocate and box a LocalSymTableStack value on the heap.
fn local_symtable_stack_box_new(stack: LocalSymTableStack) -> LocalSymTableStackBox {
    let ptr_u8: *mut u8 = alloc(
        std::mem::size_of::<LocalSymTableStack>(),
        std::mem::size_of::<usize>(),
    );
    let ptr: *mut LocalSymTableStack = ptr_u8 as *mut LocalSymTableStack;
    unsafe { *ptr = stack };
    LocalSymTableStackBox::Ptr(ptr)
}

/// Dereference a LocalSymTableStack box.
fn local_symtable_stack_box_deref(ptr_wrap: &LocalSymTableStackBox) -> &LocalSymTableStack {
    let LocalSymTableStackBox::Ptr(ptr): &LocalSymTableStackBox = ptr_wrap;
    unsafe { &**ptr }
}

/// Clone a LocalSymTableStack box and its heap-owned value.
fn local_symtable_stack_box_clone(ptr: &LocalSymTableStackBox) -> LocalSymTableStackBox {
    let cloned: LocalSymTableStack =
        local_symtable_stack_clone(local_symtable_stack_box_deref(ptr));
    local_symtable_stack_box_new(cloned)
}

/// Box-like type that is a pointer to an owned heap-allocated LocalSymTable.
enum LocalSymTableBox {
    Ptr(*mut LocalSymTable),
}

/// Allocate and box a LocalSymTable value on the heap.
fn local_symtable_box_new(symtable: LocalSymTable) -> LocalSymTableBox {
    let ptr_u8: *mut u8 = alloc(
        std::mem::size_of::<LocalSymTable>(),
        std::mem::size_of::<usize>(),
    );
    let ptr: *mut LocalSymTable = ptr_u8 as *mut LocalSymTable;
    unsafe { *ptr = symtable };
    LocalSymTableBox::Ptr(ptr)
}

/// Dereference a LocalSymTable box.
fn local_symtable_box_deref(ptr_wrap: &LocalSymTableBox) -> &LocalSymTable {
    let LocalSymTableBox::Ptr(ptr): &LocalSymTableBox = ptr_wrap;
    unsafe { &**ptr }
}

/// Clone a LocalSymTable box and its heap-owned value.
fn local_symtable_box_clone(ptr: &LocalSymTableBox) -> LocalSymTableBox {
    let cloned: LocalSymTable = local_symtable_clone(local_symtable_box_deref(ptr));
    local_symtable_box_new(cloned)
}

/// Box-like type that is a pointer to an owned heap-allocated Type.
enum TypeBox {
    Ptr(*mut Type),
}

/// Allocate and box a Type value on the heap.
fn type_box_new(ty: Type) -> TypeBox {
    let ptr_u8: *mut u8 = alloc(std::mem::size_of::<Type>(), std::mem::size_of::<usize>());
    let ptr: *mut Type = ptr_u8 as *mut Type;
    unsafe { *ptr = ty };
    TypeBox::Ptr(ptr)
}

/// Dereference a Type box.
fn type_box_deref(ptr_wrap: &TypeBox) -> &Type {
    let TypeBox::Ptr(ptr): &TypeBox = ptr_wrap;
    unsafe { &**ptr }
}

/// Clone a Type box and its heap-owned value.
fn type_box_clone(ptr: &TypeBox) -> TypeBox {
    let cloned: Type = type_clone(type_box_deref(ptr));
    type_box_new(cloned)
}

/// Box-like type that is a pointer to an owned heap-allocated Types.
enum TypesBox {
    Ptr(*mut Types),
}

/// Allocate and box a Types value on the heap.
fn types_box_new(types: Types) -> TypesBox {
    let ptr_u8: *mut u8 = alloc(std::mem::size_of::<Types>(), std::mem::size_of::<usize>());
    let ptr: *mut Types = ptr_u8 as *mut Types;
    unsafe { *ptr = types };
    TypesBox::Ptr(ptr)
}

/// Dereference a Types box.
fn types_box_deref(ptr_wrap: &TypesBox) -> &Types {
    let TypesBox::Ptr(ptr): &TypesBox = ptr_wrap;
    unsafe { &**ptr }
}

/// Clone a Types box and its heap-owned value.
fn types_box_clone(ptr: &TypesBox) -> TypesBox {
    let cloned: Types = types_clone(types_box_deref(ptr));
    types_box_new(cloned)
}

// ------------------------- String -------------------------------

/// A growable ASCII string.
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
