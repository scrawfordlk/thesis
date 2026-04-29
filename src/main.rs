#![allow(clippy::assign_op_pattern, while_true, non_snake_case)]

fn main() {
    let str: String = parse_to_llvm(
        &std::fs::read_to_string("tests/empty.rs").unwrap_or(std::string::String::new()),
    );

    let mut i: usize = 0;
    while i < string_len(&str) {
        print!("{}", unwrap_char(string_get(&str, i)));
        i = i + 1;
    }
}

// -----------------------------------------------------------------
// -----------------------------------------------------------------
// ------------------------- Compiler ------------------------------
// -----------------------------------------------------------------
// -----------------------------------------------------------------

// -----------------------------------------------------------------
// ---------------------- Lexical Analysis -------------------------
// -----------------------------------------------------------------

#[derive(Debug)]
enum Token {
    Fn,              // "fn"
    Enum,            // "enum"
    Let,             // "let"
    If,              // "if"
    Else,            // "else"
    While,           // "while"
    Return,          // "return"
    Match,           // "match"
    As,              // "as"
    Unsafe,          // "unsafe"
    Mut,             // "mut"
    Ampersand,       // "&"
    LBrace,          // "{"
    RBrace,          // "}"
    LParen,          // "("
    RParen,          // ")"
    Colon,           // ":"
    DoubleColon,     // "::"
    SemiColon,       // ";"
    Comma,           // ","
    Assign,          // "="
    Bang,            // "!"
    Cmp(Comparison), // ==, !=, <, <=, >, >=
    ArmArrow,        // "=>"
    Plus,            // "+"
    Minus,           // "-"
    Star,            // "*"
    Slash,           // "/"
    Remainder,       // "%"
    Usize,           // "usize"
    U8,              // "u8"
    Bool,            // "bool"
    Char,            // "char"
    Str,             // "str"
    TypeArrow,       // "->"
    Literal(Literal),
    Identifier(String),
    SizeOf(usize), // TODO: probably unnecessary
    Eof,
}

/// Comparison tokens
#[derive(Debug)]
enum Comparison {
    Eq,
    Neq,
    Gt,
    Lt,
    Geq,
    Leq,
}

/// Literal tokens.
#[derive(Debug)]
enum Literal {
    Int(usize),
    String(String),
    Char(char),
    Bool(bool),
}

/// A type that encapsulates the state of the lexer
enum Lexer {
    /// source file, current token
    Lexer(SourceFile, Token),
}

/// A type that manages the source file
enum SourceFile {
    /// content, current character index, current location
    SourceFile(String, usize, SourceLocation),
}

/// A type that tracks the location in the source code
enum SourceLocation {
    /// line, column
    Coords(usize, usize),
}

/// Create a lexer and prime it with the first token.
fn lexer_new(source: String) -> Lexer {
    let source_file: SourceFile = SourceFile::SourceFile(source, 0, SourceLocation::Coords(1, 1));
    let mut lexer: Lexer = Lexer::Lexer(source_file, Token::Eof);
    lexer_next_token(&mut lexer);
    lexer
}

/// Get immutable access to the lexer source file state.
fn lexer_sourcefile(lexer: &Lexer) -> &SourceFile {
    let Lexer::Lexer(source, _): &Lexer = lexer;
    source
}

/// Get mutable access to the lexer source file state.
fn lexer_sourcefile_mut(lexer: &mut Lexer) -> &mut SourceFile {
    let Lexer::Lexer(source, _): &mut Lexer = lexer;
    source
}

/// Get the current token from the lexer.
fn lexer_current_token(lexer: &Lexer) -> &Token {
    let Lexer::Lexer(_, token): &Lexer = lexer;
    token
}

/// Get mutable access to the current lexer token slot.
fn lexer_set_current_token(lexer: &mut Lexer, token: Token) {
    let Lexer::Lexer(_, old_token): &mut Lexer = lexer;
    *old_token = token;
}

/// Get the current source location tracked by the lexer.
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
    let SourceFile::SourceFile(source, index, location): &mut SourceFile =
        lexer_sourcefile_mut(lexer);

    let current: CharOption = string_get(source, *index);
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
                let ident: String = lexer_scan_identifier(lexer);
                identifier_to_token(ident)
            } else if is_digit(c) {
                let value: usize = lexer_scan_integer(lexer);
                Token::Literal(Literal::Int(value))
            } else if c == '\'' {
                let ch: char = lexer_scan_char_literal(lexer);
                Token::Literal(Literal::Char(ch))
            } else if c == '"' {
                let s: String = lexer_scan_string_literal(lexer);
                Token::Literal(Literal::String(s))
            } else {
                lexer_scan_symbol(lexer)
            }
        }
        CharOption::None => Token::Eof,
    };

    lexer_set_current_token(lexer, token_clone(&token));
    token
}

/// Scan an identifier or keyword.
fn lexer_scan_identifier(lexer: &mut Lexer) -> String {
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
        Token::Literal(Literal::Bool(true))
    } else if string_eq(&ident, &string_from_str("false")) {
        Token::Literal(Literal::Bool(false))
    } else {
        Token::Identifier(ident)
    }
}

// TODO: check for too large integer
fn lexer_scan_integer(lexer: &mut Lexer) -> usize {
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

fn lexer_scan_char_literal(lexer: &mut Lexer) -> char {
    lexer_expect_char(lexer, '\'');
    let c: char = match lexer_consume_char(lexer) {
        CharOption::Some('\\') => lexer_scan_escape_char(lexer),
        CharOption::Some(ch) => ch,
        CharOption::None => lexer_error(lexer, "unexpected end of char literal"),
    };
    lexer_expect_char(lexer, '\'');
    c
}

fn lexer_scan_string_literal(lexer: &mut Lexer) -> String {
    lexer_expect_char(lexer, '"');
    let mut s: String = string_new();
    while true {
        match lexer_consume_char(lexer) {
            CharOption::Some('"') => return s,
            CharOption::Some('\\') => string_push(&mut s, lexer_scan_escape_char(lexer)),
            CharOption::Some(c) => string_push(&mut s, c),
            CharOption::None => lexer_error(lexer, "unexpected end of string literal"),
        }
    }
    s // satisfy compiler
}

/// Scan an escape sequence after backslash.
fn lexer_scan_escape_char(lexer: &mut Lexer) -> char {
    match lexer_consume_char(lexer) {
        CharOption::Some('n') => '\n',
        CharOption::Some('t') => '\t',
        CharOption::Some('r') => '\r',
        CharOption::Some('0') => '\0',
        CharOption::Some(c) => c,
        CharOption::None => lexer_error(lexer, "unexpected end of escape sequence"),
    }
}

fn lexer_scan_symbol(lexer: &mut Lexer) -> Token {
    match unwrap_char(lexer_consume_char(lexer)) {
        '{' => Token::LBrace,
        '}' => Token::RBrace,
        '(' => Token::LParen,
        ')' => Token::RParen,
        ';' => Token::SemiColon,
        ',' => Token::Comma,
        '+' => Token::Plus,
        '*' => Token::Star,
        '/' => lexer_scan_slash(lexer),
        '%' => Token::Remainder,
        '&' => Token::Ampersand,
        ':' => lexer_scan_colon(lexer),
        '=' => lexer_scan_equals(lexer),
        '-' => lexer_scan_minus(lexer),
        '!' => lexer_scan_bang(lexer),
        '<' => lexer_scan_less(lexer),
        '>' => lexer_scan_greater(lexer),
        _ => lexer_error(lexer, "unexpected character"),
    }
}

fn lexer_scan_slash(lexer: &mut Lexer) -> Token {
    match lexer_peek_char(lexer) {
        CharOption::Some('/') => {
            lexer_consume_char(lexer);
            skip_line_comment(lexer);
            lexer_next_token(lexer)
        }
        _ => Token::Slash,
    }
}

fn lexer_scan_colon(lexer: &mut Lexer) -> Token {
    match lexer_peek_char(lexer) {
        CharOption::Some(':') => {
            lexer_consume_char(lexer);
            Token::DoubleColon
        }
        _ => Token::Colon,
    }
}

fn lexer_scan_equals(lexer: &mut Lexer) -> Token {
    match lexer_peek_char(lexer) {
        CharOption::Some('=') => {
            lexer_consume_char(lexer);
            Token::Cmp(Comparison::Eq)
        }
        CharOption::Some('>') => {
            lexer_consume_char(lexer);
            Token::ArmArrow
        }
        _ => Token::Assign,
    }
}

fn lexer_scan_minus(lexer: &mut Lexer) -> Token {
    match lexer_peek_char(lexer) {
        CharOption::Some('>') => {
            lexer_consume_char(lexer);
            Token::TypeArrow
        }
        _ => Token::Minus,
    }
}

fn lexer_scan_bang(lexer: &mut Lexer) -> Token {
    match lexer_peek_char(lexer) {
        CharOption::Some('=') => {
            lexer_consume_char(lexer);
            Token::Cmp(Comparison::Neq)
        }
        _ => Token::Bang,
    }
}

fn lexer_scan_less(lexer: &mut Lexer) -> Token {
    match lexer_peek_char(lexer) {
        CharOption::Some('=') => {
            lexer_consume_char(lexer);
            Token::Cmp(Comparison::Leq)
        }
        _ => Token::Cmp(Comparison::Lt),
    }
}

fn lexer_scan_greater(lexer: &mut Lexer) -> Token {
    match lexer_peek_char(lexer) {
        CharOption::Some('=') => {
            lexer_consume_char(lexer);
            Token::Cmp(Comparison::Geq)
        }
        _ => Token::Cmp(Comparison::Gt),
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

/// Type that encapsulates the parser's state..
enum Parser {
    /// lexer, llvm code, symbol table, current function return type
    Parser(Lexer, String, SymTable, Type),
}

/// Create a parser from a String.
fn parser_new(source: String) -> Parser {
    Parser::Parser(lexer_new(source), string_new(), symTable_new(), Type::Unit)
}

/// Create a parser from a string slice.
fn parser_from_str(source: &str) -> Parser {
    parser_new(string_from_str(source))
}

/// Get immutable access to the parser lexer.
fn parser_lexer(parser: &Parser) -> &Lexer {
    let Parser::Parser(lexer, _, _, _): &Parser = parser;
    lexer
}

/// Get mutable access to the parser lexer.
fn parser_lexer_mut(parser: &mut Parser) -> &mut Lexer {
    let Parser::Parser(lexer, _, _, _): &mut Parser = parser;
    lexer
}

/// Get immutable access to the parser LLVM output buffer.
fn parser_llvm(parser: &Parser) -> &String {
    let Parser::Parser(_, llvm, _, _): &Parser = parser;
    llvm
}

/// Get mutable access to the parser LLVM output buffer.
fn parser_llvm_mut(parser: &mut Parser) -> &mut String {
    let Parser::Parser(_, llvm, _, _): &mut Parser = parser;
    llvm
}

/// Get immutable access to the parser symbol table.
fn parser_symtable(parser: &Parser) -> &SymTable {
    let Parser::Parser(_, _, symTable, _): &Parser = parser;
    symTable
}

/// Get mutable access to the parser symbol table.
fn parser_symtable_mut(parser: &mut Parser) -> &mut SymTable {
    let Parser::Parser(_, _, symTable, _): &mut Parser = parser;
    symTable
}

/// Get the expected return type of the current function.
fn parser_current_fn_return_type(parser: &Parser) -> &Type {
    let Parser::Parser(_, _, _, return_type): &Parser = parser;
    return_type
}

/// Update the expected return type of the current function.
fn parser_set_current_fn_return_type(parser: &mut Parser, ty: Type) {
    let Parser::Parser(_, _, _, return_type): &mut Parser = parser;
    *return_type = ty;
}

/// Get the parser current token.
fn parser_current_token(parser: &Parser) -> &Token {
    lexer_current_token(parser_lexer(parser))
}

/// Advance to and return the next token.
fn parser_next_token(parser: &mut Parser) -> Token {
    lexer_next_token(parser_lexer_mut(parser))
}

/// Check whether parser current token equals `token`.
fn parser_current_token_eq(parser: &Parser, token: &Token) -> bool {
    token_eq(parser_current_token(parser), token)
}

fn parse_to_llvm(source: &str) -> String {
    let mut parser: Parser = parser_from_str(source);
    parse_language(&mut parser);
    string_clone(parser_llvm(&parser))
}

/// Consume `token` when present and report success.
fn parser_try_consume(parser: &mut Parser, token: &Token) -> bool {
    if parser_current_token_eq(parser, token) {
        parser_next_token(parser);
        true
    } else {
        false
    }
}

/// Require and consume the given token.
fn parser_expect_token(parser: &mut Parser, token: &Token) {
    if not(parser_try_consume(parser, token)) {
        parser_error(parser, "unexpected token");
    }
}

/// Require both types to be equal.
fn parser_expect_same_type(parser: &Parser, left: &Type, right: &Type) {
    if not(type_eq(left, right)) {
        parser_error(parser, "type mismatch");
    }
}

/// Require a numeric type.
fn parser_expect_numeric_type(parser: &Parser, ty: &Type) {
    if not(type_is_numeric(ty)) {
        parser_error(parser, "expected numeric type");
    }
}

/// Require a boolean type.
fn parser_expect_bool_type(parser: &Parser, ty: &Type) {
    if not(type_eq(ty, &Type::Bool)) {
        parser_error(parser, "expected bool type");
    }
}

/// Read and consume the current identifier token.
fn parser_expect_identifier(parser: &mut Parser) -> String {
    match parser_current_token(parser) {
        Token::Identifier(name) => {
            let name: String = string_clone(name);
            parser_next_token(parser);
            name
        }
        _ => parser_error(parser, "expected identifier"),
    }
}

fn parse_language(parser: &mut Parser) {
    while true {
        match parser_current_token(parser) {
            Token::Fn => parse_function(parser),
            Token::Unsafe => parse_function(parser),
            Token::Enum => parse_enum(parser),
            Token::Eof => return,
            _ => parser_error(parser, "expected top-level item"),
        }
    }
}

fn parse_function(parser: &mut Parser) {
    if parser_try_consume(parser, &Token::Unsafe) {
        // TODO: handle unsafe function
    }

    parser_expect_token(parser, &Token::Fn);

    let function_name: String = parser_expect_identifier(parser);
    symTable_enter_scope(parser_symtable_mut(parser));
    let mut parameter_types: TypeList = typeList_new();

    parser_expect_token(parser, &Token::LParen);
    if not(parser_current_token_eq(parser, &Token::RParen)) {
        let first_parameter: Variable = parse_variable(parser);
        let Variable::Var(pattern, param_type, is_mutable): Variable = first_parameter;
        typeList_append(&mut parameter_types, type_clone(&param_type));

        match pattern {
            Pattern::Identifier(name) => {
                let first_type_name: String = type_to_llvm_name(&param_type);
                llvm_emit_let_comment(parser_llvm_mut(parser), &name, &first_type_name, is_mutable);
                symTable_insert_variable(parser_symtable_mut(parser), name, param_type, is_mutable);
            }
            _ => (), // only allow irrefutable pattern
        }

        while and(
            parser_try_consume(parser, &Token::Comma),
            not(parser_current_token_eq(parser, &Token::RParen)),
        ) {
            let parameter: Variable = parse_variable(parser);
            let Variable::Var(pattern, param_type, is_mutable): Variable = parameter;
            typeList_append(&mut parameter_types, type_clone(&param_type));

            match pattern {
                Pattern::Identifier(name) => {
                    let type_name: String = type_to_llvm_name(&param_type);
                    llvm_emit_let_comment(parser_llvm_mut(parser), &name, &type_name, is_mutable);

                    if not(symTable_insert_variable(
                        parser_symtable_mut(parser),
                        name,
                        param_type,
                        is_mutable,
                    )) {
                        parser_error(parser, "duplicate parameter name");
                    }
                }
                _ => (), // only allow irrefutable pattern
            }
        }
    }
    parser_expect_token(parser, &Token::RParen);

    let function_return_type: Type = if parser_try_consume(parser, &Token::TypeArrow) {
        parse_type(parser)
    } else {
        Type::Unit
    };
    parser_set_current_fn_return_type(parser, type_clone(&function_return_type));

    let llvm_return_type_name: String = type_to_llvm_name(&function_return_type);
    llvm_emit_function_header(
        parser_llvm_mut(parser),
        &function_name,
        &llvm_return_type_name,
    );

    if not(symTable_insert_function(
        parser_symtable_mut(parser),
        function_name,
        parameter_types,
        type_clone(&function_return_type),
    )) {
        parser_error(parser, "duplicate function name");
    }

    match parse_block(parser) {
        Type::Never => (),
        block_type => parser_expect_same_type(parser, &block_type, &function_return_type),
    }

    match function_return_type {
        Type::Unit => llvm_emit_line(parser_llvm_mut(parser), "  ret void"),
        Type::Never => llvm_emit_line(parser_llvm_mut(parser), "  unreachable"),
        _ => llvm_emit_line(parser_llvm_mut(parser), "  ret i64 0"),
    }
    llvm_emit_line(parser_llvm_mut(parser), "}");

    symTable_leave_scope(parser_symtable_mut(parser));
    parser_set_current_fn_return_type(parser, Type::Unit);
}

fn parse_enum(parser: &mut Parser) {
    parser_expect_token(parser, &Token::Enum);
    let enum_name: String = parser_expect_identifier(parser);
    parser_expect_token(parser, &Token::LBrace);

    let mut variants: TypeList = typeList_new();
    let first_variant_type: Type = parse_variant(parser);
    typeList_append(&mut variants, first_variant_type);
    parser_expect_token(parser, &Token::Comma);

    while not(parser_current_token_eq(parser, &Token::RBrace)) {
        let variant_type: Type = parse_variant(parser);
        // TODO: check for duplicate variants
        typeList_append(&mut variants, variant_type);
        parser_expect_token(parser, &Token::Comma);
    }
    parser_expect_token(parser, &Token::RBrace);

    llvm_emit_enum_comment(parser_llvm_mut(parser), &enum_name);

    if not(symTable_insert_enum(
        parser_symtable_mut(parser),
        enum_name,
        variants,
    )) {
        parser_error(parser, "duplicate enum name");
    }
}

fn parse_variant(parser: &mut Parser) -> Type {
    let variant_name: String = parser_expect_identifier(parser);

    if parser_try_consume(parser, &Token::LParen) {
        parse_type(parser);

        while parser_try_consume(parser, &Token::Comma) {
            parse_type(parser);
        }

        parser_expect_token(parser, &Token::RParen);
    }

    Type::Custom(variant_name)
}

// TODO: should introduce a new symbol table
fn parse_block(parser: &mut Parser) -> Type {
    parser_expect_token(parser, &Token::LBrace);

    while not(parser_current_token_eq(parser, &Token::RBrace)) {
        match parser_current_token(parser) {
            Token::Let => {
                parse_binding(parser);
                parser_expect_token(parser, &Token::SemiColon);
            }
            _ => {
                let expression_type: Type = parse_expression(parser);
                if parser_try_consume(parser, &Token::SemiColon) {
                    llvm_emit_line(parser_llvm_mut(parser), "  ; expression statement");
                } else {
                    parser_expect_token(parser, &Token::RBrace);
                    return expression_type;
                }
            }
        }
    }

    parser_expect_token(parser, &Token::RBrace);
    Type::Unit
}

fn parse_binding(parser: &mut Parser) {
    parser_expect_token(parser, &Token::Let);
    let variable: Variable = parse_variable(parser);
    parser_expect_token(parser, &Token::Assign);
    let value_type: Type = parse_expression(parser);

    let Variable::Var(pattern, binding_type, mutable): Variable = variable;
    parser_expect_same_type(parser, &binding_type, &value_type);

    match pattern {
        Pattern::Identifier(name) => {
            symTable_insert_variable(
                parser_symtable_mut(parser),
                string_clone(&name),
                type_clone(&binding_type),
                mutable,
            );
            let binding_type_name: String = type_to_llvm_name(&binding_type);
            llvm_emit_let_comment(parser_llvm_mut(parser), &name, &binding_type_name, mutable);
        }
        // TODO: handle other patterns
        _ => llvm_emit_line(parser_llvm_mut(parser), "  ; let pattern"),
    }
}

/// Variable declaration payload parsed from source.
enum Variable {
    /// pattern, type, is mutable
    Var(Pattern, Type, bool),
}

fn parse_variable(parser: &mut Parser) -> Variable {
    let mutable: bool = parser_try_consume(parser, &Token::Mut);
    let pattern: Pattern = parse_pattern(parser);
    parser_expect_token(parser, &Token::Colon);
    let ty: Type = parse_type(parser);
    Variable::Var(pattern, ty, mutable)
}

fn parse_type(parser: &mut Parser) -> Type {
    match parser_current_token(parser) {
        Token::U8 => {
            parser_next_token(parser);
            Type::U8
        }
        Token::Usize => {
            parser_next_token(parser);
            Type::Usize
        }
        Token::Char => {
            parser_next_token(parser);
            Type::Char
        }
        Token::Bool => {
            parser_next_token(parser);
            Type::Bool
        }
        Token::LParen => {
            parser_expect_token(parser, &Token::RParen);
            Type::Unit
        }
        Token::Ampersand => {
            parser_next_token(parser);
            if parser_try_consume(parser, &Token::Mut) {
                let inner: Type = parse_type(parser);
                Type::ReferenceMut(typeBox_new(inner))
            } else if parser_try_consume(parser, &Token::Str) {
                Type::Reference(typeBox_new(Type::Custom(string_from_str("str"))))
            } else {
                let inner: Type = parse_type(parser);
                Type::Reference(typeBox_new(inner))
            }
        }
        Token::Star => {
            parser_next_token(parser);
            parser_expect_token(parser, &Token::Mut);
            let inner: Type = parse_type(parser);
            Type::RawPointerMut(typeBox_new(inner))
        }
        Token::Identifier(_) => Type::Custom(parser_expect_identifier(parser)),
        _ => parser_error(parser, "expected type"),
    }
}

fn parse_expression(parser: &mut Parser) -> Type {
    match parser_current_token(parser) {
        Token::Return => {
            parser_next_token(parser);

            let returned_type: Type = match parser_current_token(parser) {
                Token::SemiColon => Type::Unit,
                Token::RBrace => Type::Unit,
                _ => parse_expression(parser),
            };

            let expected: &Type = parser_current_fn_return_type(parser);
            parser_expect_same_type(parser, &returned_type, expected);

            Type::Never
        }
        _ => parse_assignment(parser),
    }
}

fn parse_assignment(parser: &mut Parser) -> Type {
    let left_type: Type = parse_factor(parser);

    if parser_try_consume(parser, &Token::Assign) {
        let right_type: Type = parse_comparison(parser);
        parser_expect_same_type(parser, &left_type, &right_type);

        llvm_emit_line(parser_llvm_mut(parser), "  ; assignment");

        right_type
    } else {
        left_type
    }
}

fn parse_comparison(parser: &mut Parser) -> Type {
    let left_type: Type = parse_arithmetic(parser);

    match parser_current_token(parser) {
        Token::Cmp(operator) => {
            match operator {
                Comparison::Lt => (),
                Comparison::Gt => (),
                Comparison::Leq => (),
                Comparison::Geq => (),
                _ => (),
            }
            parser_next_token(parser);

            let right_type: Type = parse_arithmetic(parser);

            parser_expect_same_type(parser, &left_type, &right_type);
            if not(or(
                type_is_numeric(&left_type),
                type_eq(&left_type, &Type::Char),
            )) {
                parser_error(parser, "cannot compare non-integer values");
            }
        }
        _ => return left_type,
    }

    Type::Bool
}

fn parse_arithmetic(parser: &mut Parser) -> Type {
    let left_type: Type = parse_term(parser);

    while or(
        parser_current_token_eq(parser, &Token::Plus),
        parser_current_token_eq(parser, &Token::Minus),
    ) {
        match parser_current_token(parser) {
            Token::Plus => (),
            Token::Minus => (),
            _ => (),
        }
        parser_next_token(parser);

        let right_type: Type = parse_term(parser);

        parser_expect_same_type(parser, &left_type, &right_type);
        parser_expect_numeric_type(parser, &left_type);

        llvm_emit_line(parser_llvm_mut(parser), "  ; add/sub");
    }

    left_type
}

fn parse_term(parser: &mut Parser) -> Type {
    let left_type: Type = parse_cast(parser);

    while or(
        parser_current_token_eq(parser, &Token::Star),
        or(
            parser_current_token_eq(parser, &Token::Slash),
            parser_current_token_eq(parser, &Token::Remainder),
        ),
    ) {
        match parser_current_token(parser) {
            Token::Star => (),
            Token::Slash => (),
            Token::Remainder => (),
            _ => (),
        }
        parser_next_token(parser);

        let right_type: Type = parse_cast(parser);

        parser_expect_same_type(parser, &left_type, &right_type);
        parser_expect_numeric_type(parser, &left_type);

        llvm_emit_line(parser_llvm_mut(parser), "  ; mul/div/rem");
    }

    left_type
}

fn parse_cast(parser: &mut Parser) -> Type {
    let mut ty: Type = parse_factor(parser);

    while parser_try_consume(parser, &Token::As) {
        let cast_type: Type = parse_type(parser);
        ty = cast_type;
    }

    ty
}

fn parse_factor(parser: &mut Parser) -> Type {
    match parser_current_token(parser) {
        Token::Minus => {
            parser_next_token(parser);
            let inner: Type = parse_factor(parser);
            parser_expect_numeric_type(parser, &inner);
            inner
        }
        Token::Star => {
            parser_next_token(parser);
            let inner: Type = parse_factor(parser);
            match inner {
                Type::RawPointerMut(pointed) => type_clone(typeBox_deref(&pointed)),
                Type::Reference(pointed) => type_clone(typeBox_deref(&pointed)),
                Type::ReferenceMut(pointed) => type_clone(typeBox_deref(&pointed)),
                _ => parser_error(parser, "cannot dereference this expression"),
            }
        }
        Token::Ampersand => {
            parser_next_token(parser);
            let mutable: bool = parser_try_consume(parser, &Token::Mut);
            let inner: Type = parse_factor(parser);
            if mutable {
                Type::ReferenceMut(typeBox_new(inner))
            } else {
                Type::Reference(typeBox_new(inner))
            }
        }
        Token::Literal(_) => parse_literal(parser),
        Token::Identifier(_) => {
            let name: String = parser_expect_identifier(parser);
            if parser_current_token_eq(parser, &Token::LParen) {
                parse_call(parser, name)
            } else {
                match symTable_lookup_variable_type(parser_symtable(parser), &name) {
                    TypeOption::Some(ty) => ty,
                    TypeOption::None => parser_error(parser, "undefined variable"),
                }
            }
        }
        Token::LParen => {
            parser_next_token(parser);
            let ty: Type = parse_expression(parser);
            parser_expect_token(parser, &Token::RParen);
            ty
        }
        Token::Unsafe => {
            parser_next_token(parser);
            parse_block(parser)
        }
        Token::LBrace => parse_block(parser),
        Token::If => parse_if(parser),
        Token::While => parse_while(parser),
        Token::Match => parse_match(parser),
        _ => parser_error(parser, "unexpected token"),
    }
}

fn parse_if(parser: &mut Parser) -> Type {
    parser_expect_token(parser, &Token::If);

    let condition_type: Type = parse_expression(parser);
    parser_expect_bool_type(parser, &condition_type);

    let then_type: Type = parse_block(parser);
    if parser_try_consume(parser, &Token::Else) {
        let else_type: Type = if parser_current_token_eq(parser, &Token::If) {
            parse_if(parser)
        } else {
            parse_block(parser)
        };
        parser_expect_same_type(parser, &then_type, &else_type);
        then_type
    } else {
        Type::Unit
    }
}

fn parse_while(parser: &mut Parser) -> Type {
    parser_expect_token(parser, &Token::While);

    let condition_type: Type = parse_expression(parser);
    parser_expect_bool_type(parser, &condition_type);

    parse_block(parser);

    Type::Unit
}

fn parse_match(parser: &mut Parser) -> Type {
    parser_expect_token(parser, &Token::Match);

    let expression_type: Type = parse_expression(parser);

    parser_expect_token(parser, &Token::LBrace);

    let return_type: Type = parse_arms(parser, &expression_type);
    parser_expect_token(parser, &Token::RBrace);

    return_type
}

fn parse_arms(parser: &mut Parser, matched_type: &Type) -> Type {
    let first_pattern: Pattern = parse_pattern(parser);
    let first_pattern_type: Type = pattern_type_for_expression(&first_pattern, matched_type);
    parser_expect_same_type(parser, &first_pattern_type, matched_type);

    parser_expect_token(parser, &Token::ArmArrow);

    let return_type: Type = parse_expression(parser);
    parser_expect_token(parser, &Token::Comma);

    while not(parser_current_token_eq(parser, &Token::RBrace)) {
        let pattern: Pattern = parse_pattern(parser);
        let pattern_type: Type = pattern_type_for_expression(&pattern, matched_type);
        parser_expect_same_type(parser, &pattern_type, matched_type);

        parser_expect_token(parser, &Token::ArmArrow);

        let arm_type: Type = parse_expression(parser);
        parser_expect_same_type(parser, &return_type, &arm_type);
        parser_expect_token(parser, &Token::Comma);
    }

    return_type
}

/// Pattern forms supported by the parser.
/// TODO: currently very simple skeleton
enum Pattern {
    Literal(Type),
    Identifier(String),
    /// type name, variant name
    EnumVariant(String, String),
    Wildcard,
}

/// Derive the expected type contributed by a match pattern.
fn pattern_type_for_expression(pattern: &Pattern, expression_type: &Type) -> Type {
    match pattern {
        Pattern::Literal(ty) => type_clone(ty),
        Pattern::Identifier(_) => type_clone(expression_type),
        Pattern::EnumVariant(enum_name, _) => Type::Custom(string_clone(enum_name)),
        Pattern::Wildcard => type_clone(expression_type),
    }
}

fn parse_pattern(parser: &mut Parser) -> Pattern {
    match parser_current_token(parser) {
        Token::Literal(literal) => {
            let current_literal: Literal = literalToken_clone(literal);
            parser_next_token(parser);
            match current_literal {
                Literal::Int(_) => Pattern::Literal(Type::Usize),
                Literal::Char(_) => Pattern::Literal(Type::Char),
                Literal::String(_) => Pattern::Literal(Type::Reference(typeBox_new(Type::Custom(
                    string_from_str("str"),
                )))),
                Literal::Bool(_) => Pattern::Literal(Type::Bool),
            }
        }
        Token::Identifier(_) => {
            let identifier: String = parser_expect_identifier(parser);

            if string_eq(&identifier, &string_from_str("_")) {
                Pattern::Wildcard
            } else if parser_try_consume(parser, &Token::DoubleColon) {
                let variant_name: String = parser_expect_identifier(parser);

                if parser_try_consume(parser, &Token::LParen) {
                    let pattern: Pattern = parse_pattern(parser);

                    while parser_try_consume(parser, &Token::Comma) {
                        let pattern: Pattern = parse_pattern(parser);
                    }

                    parser_expect_token(parser, &Token::RParen);
                }

                Pattern::EnumVariant(identifier, variant_name)
            } else {
                Pattern::Identifier(identifier)
            }
        }
        _ => parser_error(parser, "expected pattern"),
    }
}

fn parse_call(parser: &mut Parser, function_name: String) -> Type {
    parser_expect_token(parser, &Token::LParen);

    let mut argument_types: TypeList = typeList_new();
    if not(parser_current_token_eq(parser, &Token::RParen)) {
        let first_argument_type: Type = parse_expression(parser);
        typeList_append(&mut argument_types, first_argument_type);

        while and(
            parser_try_consume(parser, &Token::Comma),
            not(parser_current_token_eq(parser, &Token::RParen)),
        ) {
            let argument_type: Type = parse_expression(parser);
            typeList_append(&mut argument_types, argument_type);
        }
    }
    parser_expect_token(parser, &Token::RParen);

    match symTable_lookup_function_signature(parser_symtable(parser), &function_name) {
        FnSignatureOption::Some(FnSignature::Fn(parameter_types, return_type)) => {
            if not(typeList_eq(&parameter_types, &argument_types)) {
                parser_error(parser, "function call does not match function signature");
            }

            llvm_emit_call_comment(parser_llvm_mut(parser), &function_name);

            return_type
        }
        FnSignatureOption::None => parser_error(parser, "call to undefined function"),
    }
}

fn parse_literal(parser: &mut Parser) -> Type {
    match parser_current_token(parser) {
        Token::Literal(literal) => {
            let current_literal: Literal = literalToken_clone(literal);
            parser_next_token(parser);

            match current_literal {
                Literal::Int(_) => Type::Usize,
                Literal::String(_) => {
                    Type::Reference(typeBox_new(Type::Custom(string_from_str("str"))))
                }
                Literal::Char(_) => Type::Char,
                Literal::Bool(_) => Type::Bool,
            }
        }
        _ => parser_error(parser, "expected literal"),
    }
}

/// Data structure that manages a global symbol table and (multiple) local symbol tables.
enum SymTable {
    Table(GlobalSymTable, LocalSymTableStack),
}

/// Create an empty symbol table.
fn symTable_new() -> SymTable {
    let global: GlobalSymTable = GlobalSymTable::Nil;
    let local: LocalSymTableStack = LocalSymTableStack::Nil;
    SymTable::Table(global, local)
}

/// Check whether a symbol exists in local scopes or globals.
fn symTable_contains(symtable: &SymTable, name: &String) -> bool {
    let SymTable::Table(global, local): &SymTable = symtable;
    or(
        localSymTableStack_contains(local, name),
        globalSymTable_contains(global, name),
    )
}

/// Lookup a variable type in local scopes.
fn symTable_lookup_variable_type(symtable: &SymTable, name: &String) -> TypeOption {
    let SymTable::Table(_, local): &SymTable = symtable;
    localSymTableStack_lookup_variable_type(local, name)
}

/// Lookup a function signature in the global symbol table.
fn symTable_lookup_function_signature(symtable: &SymTable, name: &String) -> FnSignatureOption {
    let SymTable::Table(global, _): &SymTable = symtable;
    globalSymTable_lookup_function_signature(global, name)
}

/// Enter a new local scope.
fn symTable_enter_scope(symtable: &mut SymTable) {
    let SymTable::Table(_, local): &mut SymTable = symtable;
    localSymTableStack_push(local);
}

/// Leave the current local scope.
fn symTable_leave_scope(symtable: &mut SymTable) -> bool {
    let SymTable::Table(_, local_stack): &mut SymTable = symtable;
    localSymTableStack_pop(local_stack)
}

/// Insert a function into the global symbol table, returning false on duplicate name.
fn symTable_insert_function(
    symtable: &mut SymTable,
    name: String,
    parameter_types: TypeList,
    return_type: Type,
) -> bool {
    let SymTable::Table(global, _) = symtable;
    globalSymTable_insert_function(global, name, parameter_types, return_type)
}

/// Insert an enum into the global table.
/// Return true, if the name is not taken else false.
fn symTable_insert_enum(symtable: &mut SymTable, name: String, variants: TypeList) -> bool {
    let SymTable::Table(global, _) = symtable;
    globalSymTable_insert_enum(global, name, variants)
}

/// Insert a variable into the current local scope.
/// Returns true if the variable name is not already taken, else false.
fn symTable_insert_variable(
    symtable: &mut SymTable,
    name: String,
    variable_type: Type,
    mutable: bool,
) -> bool {
    let SymTable::Table(_, local_stack): &mut SymTable = symtable;
    match local_stack {
        LocalSymTableStack::Cons(local, _) => {
            localSymTable_insert_variable(local, name, variable_type, mutable)
        }
        LocalSymTableStack::Nil => true,
    }
}

/// Global symbol table represented as a cons list.
enum GlobalSymTable {
    /// head, tail
    Cons(SymTableEntry, GlobalSymTableBox),
    Nil,
}

/// Prepend an entry to the global table.
fn globalSymTable_prepend(symtable: &mut GlobalSymTable, entry: SymTableEntry) {
    let old_copy: GlobalSymTable = globalSymTable_clone(symtable);
    let tail: GlobalSymTableBox = globalSymTableBox_new(old_copy);
    *symtable = GlobalSymTable::Cons(entry, tail);
}

/// Check whether a name exists in the global table.
fn globalSymTable_contains(symtable: &GlobalSymTable, name: &String) -> bool {
    match symtable {
        GlobalSymTable::Cons(head, tail) => {
            let entry_name: &String = symTableEntry_name(&head);
            let matches: bool = string_eq(entry_name, name);
            or(
                matches,
                globalSymTable_contains(globalSymTableBox_deref(tail), name),
            )
        }
        GlobalSymTable::Nil => false,
    }
}

/// Lookup a function signature in globals.
fn globalSymTable_lookup_function_signature(
    symtable: &GlobalSymTable,
    name: &String,
) -> FnSignatureOption {
    match symtable {
        GlobalSymTable::Cons(entry, tail) => match entry {
            SymTableEntry::Function(entry_name, signature) => {
                if string_eq(entry_name, name) {
                    FnSignatureOption::Some(fnSignature_clone(signature))
                } else {
                    globalSymTable_lookup_function_signature(globalSymTableBox_deref(tail), name)
                }
            }
            _ => globalSymTable_lookup_function_signature(globalSymTableBox_deref(tail), name),
        },
        GlobalSymTable::Nil => FnSignatureOption::None,
    }
}

/// Insert a function entry into globals, returning false on duplicate name.
fn globalSymTable_insert_function(
    symtable: &mut GlobalSymTable,
    name: String,
    parameter_types: TypeList,
    return_type: Type,
) -> bool {
    if globalSymTable_contains(symtable, &name) {
        return false;
    }

    let signature: FnSignature = FnSignature::Fn(parameter_types, return_type);
    let entry: SymTableEntry = SymTableEntry::Function(name, signature);
    globalSymTable_prepend(symtable, entry);
    true
}

/// Insert an enum entry into globals, returning false on duplicate name.
fn globalSymTable_insert_enum(
    symtable: &mut GlobalSymTable,
    name: String,
    variants: TypeList,
) -> bool {
    if globalSymTable_contains(symtable, &name) {
        return false;
    }

    let entry: SymTableEntry = SymTableEntry::Enum(name, variants);
    globalSymTable_prepend(symtable, entry);
    true
}

/// Stack of local scopes represented as a cons list.
enum LocalSymTableStack {
    /// head, tail
    Cons(LocalSymTable, LocalSymTableStackBox),
    Nil,
}

/// Push a new empty local scope onto the stack.
fn localSymTableStack_push(stack: &mut LocalSymTableStack) {
    let old_copy: LocalSymTableStack = localSymTableStack_clone(stack);
    let tail: LocalSymTableStackBox = localSymTableStackBox_new(old_copy);
    *stack = LocalSymTableStack::Cons(LocalSymTable::Nil, tail);
}

/// Pop the top local scope from the stack.
fn localSymTableStack_pop(stack: &mut LocalSymTableStack) -> bool {
    match stack {
        LocalSymTableStack::Cons(_, tail) => {
            *stack = localSymTableStack_clone(localSymTableStackBox_deref(tail));
            true
        }
        LocalSymTableStack::Nil => false,
    }
}

/// Check whether a name exists in any local scope.
fn localSymTableStack_contains(stack: &LocalSymTableStack, name: &String) -> bool {
    match stack {
        LocalSymTableStack::Cons(local, tail) => or(
            localSymTable_contains(local, name),
            localSymTableStack_contains(localSymTableStackBox_deref(tail), name),
        ),
        LocalSymTableStack::Nil => false,
    }
}

/// Lookup a variable type in any local scope.
fn localSymTableStack_lookup_variable_type(
    stack: &LocalSymTableStack,
    name: &String,
) -> TypeOption {
    match stack {
        LocalSymTableStack::Cons(local, tail) => {
            match localSymTable_lookup_variable_type(local, name) {
                TypeOption::Some(variable_type) => TypeOption::Some(variable_type),
                TypeOption::None => {
                    localSymTableStack_lookup_variable_type(localSymTableStackBox_deref(tail), name)
                }
            }
        }
        LocalSymTableStack::Nil => TypeOption::None,
    }
}

/// Single local scope represented as a linked cons list.
enum LocalSymTable {
    /// head, tail
    Cons(SymTableEntry, LocalSymTableBox),
    Nil,
}

/// Prepend an entry to a local scope.
fn localSymTable_prepend(symtable: &mut LocalSymTable, entry: SymTableEntry) {
    let old_copy: LocalSymTable = localSymTable_clone(symtable);
    let tail: LocalSymTableBox = localSymTableBox_new(old_copy);
    *symtable = LocalSymTable::Cons(entry, tail);
}

/// Check whether a name exists in a local scope.
fn localSymTable_contains(symtable: &LocalSymTable, name: &String) -> bool {
    match symtable {
        LocalSymTable::Cons(head, tail) => {
            let entry_name: &String = symTableEntry_name(head);
            let matches: bool = string_eq(entry_name, name);
            or(
                matches,
                localSymTable_contains(localSymTableBox_deref(tail), name),
            )
        }
        LocalSymTable::Nil => false,
    }
}

/// Lookup a variable type in a single local scope.
fn localSymTable_lookup_variable_type(symtable: &LocalSymTable, name: &String) -> TypeOption {
    match symtable {
        LocalSymTable::Cons(entry, tail) => match entry {
            SymTableEntry::Variable(entry_name, variable_type, _) => {
                if string_eq(entry_name, name) {
                    TypeOption::Some(type_clone(variable_type))
                } else {
                    localSymTable_lookup_variable_type(localSymTableBox_deref(tail), name)
                }
            }
            _ => localSymTable_lookup_variable_type(localSymTableBox_deref(tail), name),
        },
        LocalSymTable::Nil => TypeOption::None,
    }
}

/// Insert a variable entry into a single local scope.
/// Returns true if the variable name is not already taken, else false (in which case it is still
/// inserted (= shadowing))
fn localSymTable_insert_variable(
    symtable: &mut LocalSymTable,
    name: String,
    variable_type: Type,
    mutable: bool,
) -> bool {
    let already_used: bool = localSymTable_contains(symtable, &name);
    let entry: SymTableEntry = SymTableEntry::Variable(name, variable_type, mutable);
    localSymTable_prepend(symtable, entry);
    already_used
}

/// Symbol table entry for functions, enums, and variables.
enum SymTableEntry {
    /// name, signature
    Function(String, FnSignature),
    /// name, variant types
    Enum(String, TypeList),
    /// name, type, is mutable
    Variable(String, Type, bool),
}

/// Get the name associated with a symbol table entry.
fn symTableEntry_name(entry: &SymTableEntry) -> &String {
    match entry {
        SymTableEntry::Function(name, _) => name,
        SymTableEntry::Enum(name, _) => name,
        SymTableEntry::Variable(name, _, _) => name,
    }
}

/// A type that represents the (type) signature of a function
enum FnSignature {
    /// parameter types, return type
    Fn(TypeList, Type),
}

/// Type forms supported by the front-end.
enum Type {
    U8,
    Usize,
    Bool,
    Char,
    Unit,                   // ()
    Never,                  // !
    Custom(String),         // enums
    Reference(TypeBox),     // &Type
    ReferenceMut(TypeBox),  // &mut Type
    RawPointerMut(TypeBox), // *mut Type
}

fn type_is_numeric(ty: &Type) -> bool {
    match ty {
        Type::U8 => true,
        Type::Usize => true,
        _ => false,
    }
}

/// Convert type into a simple LLVM-IR type name.
fn type_to_llvm_name(ty: &Type) -> String {
    match ty {
        Type::U8 => string_from_str("i8"),
        Type::Usize => string_from_str("i64"), // assume 64-bit for now
        Type::Bool => string_from_str("i1"),
        Type::Char => string_from_str("i32"),
        Type::Unit => string_from_str("void"),
        Type::Never => string_from_str("void"),
        Type::Custom(_) => string_from_str("i64"),
        Type::Reference(_) => string_from_str("i64"),
        Type::ReferenceMut(_) => string_from_str("i64"),
        Type::RawPointerMut(_) => string_from_str("i64"),
    }
}

// -----------------------------------------------------------------
// -----------------------------------------------------------------
// ------------------------ LLVM Emulator -------------------------
// -----------------------------------------------------------------
// -----------------------------------------------------------------

// -----------------------------------------------------------------
// ---------------------- Lexical Analysis -------------------------
// -----------------------------------------------------------------

/// Tokens produced by the LLVM lexer.
enum LlvmToken {
    Define,          // "define"
    Ret,             // "ret"
    Br,              // "br"
    Label,           // "label"
    Add,             // "add"
    Sub,             // "sub"
    Mul,             // "mul"
    Udiv,            // "udiv"
    Urem,            // "urem"
    Icmp,            // "icmp"
    Call,            // "call"
    Gep,             // "getelementptr"
    Constant,        // "constant"
    Ult,             // "ult"
    Ptr,             // "ptr"
    I64,             // "i64"
    I32,             // "i32"
    I8,              // "i8"
    I1,              // "i1"
    Void,            // "void"
    At,              // "@"
    Percent,         // "%"
    LParen,          // "("
    RParen,          // ")"
    LBrace,          // "{"
    RBrace,          // "}"
    LBracket,        // "["
    RBracket,        // "]"
    Comma,           // ","
    Minus,           // "-"
    Assign,          // "="
    Colon,           // ":"
    CString(String), // c"..."
    Identifier(String),
    Integer(usize),
    Eof,
}

/// A type that encapsulates the state of the lexer for the LLVM-IR parser.
enum LlvmLexer {
    /// LLVM-IR human-readable source file, current token
    Lexer(SourceFile, LlvmToken),
}

/// Create a new LLVM lexer and scan the first token.
fn llvmLexer_new(source: String) -> LlvmLexer {
    let source_file: SourceFile = SourceFile::SourceFile(source, 0, SourceLocation::Coords(1, 1));
    let mut lexer: LlvmLexer = LlvmLexer::Lexer(source_file, LlvmToken::Eof);
    llvmLexer_next_token(&mut lexer);
    lexer
}

/// Get the lexer source file.
fn llvmLexer_sourcefile(lexer: &LlvmLexer) -> &SourceFile {
    let LlvmLexer::Lexer(source, _): &LlvmLexer = lexer;
    source
}

/// Get the lexer source file.
fn llvmLexer_sourcefile_mut(lexer: &mut LlvmLexer) -> &mut SourceFile {
    let LlvmLexer::Lexer(source, _): &mut LlvmLexer = lexer;
    source
}

/// Get the current lexer token.
fn llvmLexer_current_token(lexer: &LlvmLexer) -> &LlvmToken {
    let LlvmLexer::Lexer(_, token): &LlvmLexer = lexer;
    token
}

/// Set the current lexer token.
fn llvmLexer_set_current_token(lexer: &mut LlvmLexer, token: LlvmToken) {
    let LlvmLexer::Lexer(_, old_token): &mut LlvmLexer = lexer;
    *old_token = token;
}

/// Get the location the lexer is currently at.
fn llvmLexer_location(lexer: &LlvmLexer) -> &SourceLocation {
    let SourceFile::SourceFile(_, _, location) = llvmLexer_sourcefile(lexer);
    location
}

/// Peek the current source character.
fn llvmLexer_peek_char(lexer: &LlvmLexer) -> CharOption {
    let SourceFile::SourceFile(content, index, _): &SourceFile = llvmLexer_sourcefile(lexer);
    string_get(content, *index)
}

/// Peek the next source character after the current one and return true if it is the expected
/// character
fn llvmLexer_next_char_eq(lexer: &LlvmLexer, expected: char) -> bool {
    let SourceFile::SourceFile(content, index, _): &SourceFile = llvmLexer_sourcefile(lexer);
    match string_get(content, *index + 1) {
        CharOption::Some(character) => character == expected,
        _ => false,
    }
}

fn llvmLexer_expect_char(lexer: &mut LlvmLexer, expected: char) {
    match llvmLexer_consume_char(lexer) {
        CharOption::Some(c) => {
            if c != expected {
                panic!("unexpected character");
            }
        }
        _ => panic!("unexpected EOF"),
    }
}

/// Consume and return the current source character.
fn llvmLexer_consume_char(lexer: &mut LlvmLexer) -> CharOption {
    let SourceFile::SourceFile(source, index, location): &mut SourceFile =
        llvmLexer_sourcefile_mut(lexer);

    let current: CharOption = string_get(source, *index);
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

/// Consume and return the next token.
fn llvmLexer_next_token(lexer: &mut LlvmLexer) -> LlvmToken {
    llvmLexer_skip_whitespace_and_comments(lexer);

    let token: LlvmToken = match llvmLexer_peek_char(lexer) {
        CharOption::Some(ch) => {
            if and(ch == 'c', llvmLexer_next_char_eq(lexer, '"')) {
                let value: String = llvmLexer_scan_cstring(lexer);
                LlvmToken::CString(value)
            } else if or(is_alpha(ch), ch == '.') {
                let ident: String = llvmLexer_scan_identifier_or_keyword(lexer);
                llvm_identifier_to_token(ident)
            } else if is_digit(ch) {
                let value: usize = llvmLexer_scan_integer(lexer);
                LlvmToken::Integer(value)
            } else {
                llvmLexer_scan_symbol(lexer)
            }
        }
        CharOption::None => LlvmToken::Eof,
    };

    llvmLexer_set_current_token(lexer, llvmToken_clone(&token));
    token
}

/// Scan and return a c"..." string literal.
fn llvmLexer_scan_cstring(lexer: &mut LlvmLexer) -> String {
    let mut literal: String = string_new();
    llvmLexer_expect_char(lexer, 'c');
    llvmLexer_expect_char(lexer, '"');

    while true {
        match llvmLexer_consume_char(lexer) {
            CharOption::Some('"') => return literal,
            CharOption::Some('\\') => {
                let character: char = llvmLexer_scan_escape(lexer);
                string_push(&mut literal, character);
            }
            CharOption::Some(ch) => string_push(&mut literal, ch),
            CharOption::None => panic!("unterminated LLVM c-string"),
        }
    }
    literal // satisfy compiler
}

fn llvmLexer_scan_escape(lexer: &mut LlvmLexer) -> char {
    match llvmLexer_consume_char(lexer) {
        CharOption::Some(hex_digit) => {
            if is_hexadecimal_digit(hex_digit) {
                match llvmLexer_consume_char(lexer) {
                    CharOption::Some(second_hex_digit) => {
                        let mut char_byte: String = string_new();
                        string_push(&mut char_byte, hex_digit);
                        string_push(&mut char_byte, second_hex_digit);

                        unwrap_usize(string_to_integer(&mut char_byte, 16)) as u8 as char
                    }
                    _ => panic!("expected second digit for escaped character byte"),
                }
            } else {
                hex_digit
            }
        }
        CharOption::None => panic!("unterminated LLVM c-string"),
    }
}

fn llvmLexer_scan_identifier_or_keyword(lexer: &mut LlvmLexer) -> String {
    let mut identifier: String = string_new();
    while true {
        match llvmLexer_peek_char(lexer) {
            CharOption::Some(ch) => {
                if is_alphanumeric_or_dot(ch) {
                    llvmLexer_consume_char(lexer);
                    string_push(&mut identifier, ch);
                } else {
                    return identifier;
                }
            }
            CharOption::None => return identifier,
        }
    }
    identifier // satisfy compiler
}

fn llvm_identifier_to_token(identifier: String) -> LlvmToken {
    if string_eq(&identifier, &string_from_str("define")) {
        LlvmToken::Define
    } else if string_eq(&identifier, &string_from_str("ret")) {
        LlvmToken::Ret
    } else if string_eq(&identifier, &string_from_str("br")) {
        LlvmToken::Br
    } else if string_eq(&identifier, &string_from_str("label")) {
        LlvmToken::Label
    } else if string_eq(&identifier, &string_from_str("add")) {
        LlvmToken::Add
    } else if string_eq(&identifier, &string_from_str("sub")) {
        LlvmToken::Sub
    } else if string_eq(&identifier, &string_from_str("mul")) {
        LlvmToken::Mul
    } else if string_eq(&identifier, &string_from_str("udiv")) {
        LlvmToken::Udiv
    } else if string_eq(&identifier, &string_from_str("urem")) {
        LlvmToken::Urem
    } else if string_eq(&identifier, &string_from_str("icmp")) {
        LlvmToken::Icmp
    } else if string_eq(&identifier, &string_from_str("call")) {
        LlvmToken::Call
    } else if string_eq(&identifier, &string_from_str("getelementptr")) {
        LlvmToken::Gep
    } else if string_eq(&identifier, &string_from_str("constant")) {
        LlvmToken::Constant
    } else if string_eq(&identifier, &string_from_str("ult")) {
        LlvmToken::Ult
    } else if string_eq(&identifier, &string_from_str("ptr")) {
        LlvmToken::Ptr
    } else if string_eq(&identifier, &string_from_str("i64")) {
        LlvmToken::I64
    } else if string_eq(&identifier, &string_from_str("i32")) {
        LlvmToken::I32
    } else if string_eq(&identifier, &string_from_str("i8")) {
        LlvmToken::I8
    } else if string_eq(&identifier, &string_from_str("i1")) {
        LlvmToken::I1
    } else if string_eq(&identifier, &string_from_str("void")) {
        LlvmToken::Void
    } else {
        LlvmToken::Identifier(identifier)
    }
}

fn llvmLexer_scan_integer(lexer: &mut LlvmLexer) -> usize {
    let mut value: usize = 0;
    while true {
        match llvmLexer_peek_char(lexer) {
            CharOption::Some(ch) => {
                if is_digit(ch) {
                    let digit: usize = (ch as usize) - ('0' as usize);
                    value = value * 10 + digit;
                    llvmLexer_consume_char(lexer);
                } else {
                    return value;
                }
            }
            CharOption::None => return value,
        }
    }
    value
}

fn llvmLexer_scan_symbol(lexer: &mut LlvmLexer) -> LlvmToken {
    match unwrap_char(llvmLexer_consume_char(lexer)) {
        '@' => LlvmToken::At,
        '%' => LlvmToken::Percent,
        '(' => LlvmToken::LParen,
        ')' => LlvmToken::RParen,
        '{' => LlvmToken::LBrace,
        '}' => LlvmToken::RBrace,
        '[' => LlvmToken::LBracket,
        ']' => LlvmToken::RBracket,
        ',' => LlvmToken::Comma,
        '-' => LlvmToken::Minus,
        '=' => LlvmToken::Assign,
        ':' => LlvmToken::Colon,
        _ => panic!("unsupported token in LLVM input"),
    }
}

fn llvmLexer_skip_whitespace_and_comments(lexer: &mut LlvmLexer) {
    while true {
        match llvmLexer_peek_char(lexer) {
            CharOption::Some(ch) => {
                if is_whitespace(ch) {
                    llvmLexer_consume_char(lexer);
                } else if ch == ';' {
                    llvmLexer_consume_char(lexer);
                    llvmLexer_skip_line(lexer);
                } else {
                    return;
                }
            }
            CharOption::None => return,
        }
    }
}

fn llvmLexer_skip_line(lexer: &mut LlvmLexer) {
    while true {
        match llvmLexer_consume_char(lexer) {
            CharOption::Some('\n') => return,
            CharOption::Some(_) => (),
            CharOption::None => return,
        }
    }
}

// -----------------------------------------------------------------
// -----------------------------------------------------------------
// ------------------------- Library -------------------------------
// -----------------------------------------------------------------
// -----------------------------------------------------------------

// ------------------------- LLVM-IR -------------------------------

/// Append raw text to the LLVM-IR output buffer.
fn llvm_emit_str(llvm: &mut String, str: &str) {
    string_push_str(llvm, str);
}

/// Append a String value to the LLVM-IR output buffer.
fn llvm_emit_string(llvm: &mut String, string: &String) {
    string_push_string(llvm, string);
}

/// Append a single newline to the LLVM-IR output buffer.
fn llvm_emit_newline(llvm: &mut String) {
    string_push(llvm, '\n');
}

/// Append one full LLVM-IR line to the output buffer.
fn llvm_emit_line(llvm: &mut String, text: &str) {
    llvm_emit_str(llvm, text);
    llvm_emit_newline(llvm);
}

/// Emit a function header.
fn llvm_emit_function_header(llvm: &mut String, fn_name: &String, return_type_name: &String) {
    llvm_emit_str(llvm, "define ");
    llvm_emit_string(llvm, return_type_name);
    llvm_emit_str(llvm, " @");
    llvm_emit_string(llvm, fn_name);
    llvm_emit_line(llvm, "() {");
}

/// Emit an enum comment line.
fn llvm_emit_enum_comment(llvm: &mut String, enum_name: &String) {
    llvm_emit_str(llvm, "; enum ");
    llvm_emit_string(llvm, enum_name);
    llvm_emit_newline(llvm);
}

/// Emit a let-binding comment line.
fn llvm_emit_let_comment(
    llvm: &mut String,
    variable_name: &String,
    type_name: &String,
    is_mutable: bool,
) {
    llvm_emit_str(llvm, "  ; let ");
    if is_mutable {
        llvm_emit_str(llvm, "mut ");
    }
    llvm_emit_string(llvm, variable_name);
    llvm_emit_str(llvm, ": ");
    llvm_emit_string(llvm, type_name);
    llvm_emit_newline(llvm);
}

/// Emit a function-call comment line.
fn llvm_emit_call_comment(llvm: &mut String, function_name: &String) {
    llvm_emit_str(llvm, "  ; call ");
    llvm_emit_string(llvm, function_name);
    llvm_emit_newline(llvm);
}

// -------------------------- error --------------------------------

/// Report an error message with source location and exit.
/// TODO: not subset-conform
fn lexer_error(lexer: &Lexer, message: &str) -> ! {
    let SourceLocation::Coords(line, col): &SourceLocation = lexer_location(lexer);
    eprintln!("error at {}:{}: {}", line, col, message);
    std::process::exit(1);
}

/// Emit an error at the parser current location and abort.
fn parser_error(parser: &Parser, message: &str) -> ! {
    lexer_error(parser_lexer(parser), message)
}

// -----------------------------------------------------------------
// -------------------------- bool ---------------------------------
// -----------------------------------------------------------------

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

// -----------------------------------------------------------------
// -------------------------- char ---------------------------------
// -----------------------------------------------------------------

/// Check whether a character is whitespace.
fn is_whitespace(c: char) -> bool {
    or(or(c == ' ', c == '\t'), or(c == '\n', c == '\r'))
}

/// Check whether a character is a decimal digit.
fn is_digit(c: char) -> bool {
    and(c >= '0', c <= '9')
}

/// Check whether a character is a hexadecimal digit.
/// Both upper and lowercase hexadecimal digits are considered valid.
fn is_hexadecimal_digit(c: char) -> bool {
    let upper: char = to_uppercase(c);
    or(is_digit(c), and(upper >= 'A', upper <= 'F'))
}

/// Check whether a character is a lowercase letter
fn is_lowercase(c: char) -> bool {
    and(c >= 'a', c <= 'z')
}

/// Check whether a character is an uppercase letter
fn is_uppercase(c: char) -> bool {
    and(c >= 'A', c <= 'Z')
}

/// Check whether a character is an alphabetical letter
fn is_letter(c: char) -> bool {
    or(is_lowercase(c), is_uppercase(c))
}

/// Check whether a character is alphabetic or underscore.
fn is_alpha(c: char) -> bool {
    or(is_letter(c), c == '_')
}

/// Check whether a character is alphanumeric.
fn is_alphanumeric(c: char) -> bool {
    or(is_alpha(c), is_digit(c))
}

/// Check whether a character is alphanumeric or '.'.
fn is_alphanumeric_or_dot(ch: char) -> bool {
    or(is_alphanumeric(ch), ch == '.')
}

/// Convert an ASCII character to uppercase.
/// If the character is not a letter, it is returned as is.
fn to_uppercase(c: char) -> char {
    if or(not(is_letter(c)), is_uppercase(c)) {
        c
    } else {
        (c as u8 - ('a' as u8 - 'A' as u8)) as char
    }
}

// -----------------------------------------------------------------
// ------------------------ Option<T> ------------------------------
// -----------------------------------------------------------------

/// Option type for FnSignature.
enum FnSignatureOption {
    Some(FnSignature),
    None,
}

/// Option type for Type.
enum TypeOption {
    Some(Type),
    None,
}

/// Option type for char.
enum CharOption {
    Some(char),
    None,
}

/// Option type for usize.
enum UsizeOption {
    Some(usize),
    None,
}

/// Returns the value wrapped in Some.
/// If the option is None, end the program.
fn unwrap_char(char_opt: CharOption) -> char {
    match char_opt {
        CharOption::Some(c) => c,
        CharOption::None => panic!("tried to unwrap None variant of CharOption"),
    }
}

/// Returns the value wrapped in Some.
/// If the option is None, end the program.
fn unwrap_usize(n_opt: UsizeOption) -> usize {
    match n_opt {
        UsizeOption::Some(n) => n,
        UsizeOption::None => panic!("tried to unwrap None variant of UsizeOption"),
    }
}

// -----------------------------------------------------------------
// -------------------------- Lists --------------------------------
// -----------------------------------------------------------------

/// List of Type values.
enum TypeList {
    /// head, tail
    Cons(Type, TypeListBox),
    Nil,
}

/// Create an empty TypeList.
fn typeList_new() -> TypeList {
    TypeList::Nil
}

/// Append one type to a TypeList.
fn typeList_append(list: &mut TypeList, ty: Type) {
    let mut current: &mut TypeList = list;

    while true {
        match current {
            TypeList::Nil => {
                *current = TypeList::Cons(ty, typeListBox_new(TypeList::Nil));
                return;
            }
            TypeList::Cons(_, tail) => current = typeListBox_deref_mut(tail),
        }
    }
}

// ----------------------------------------------------------------
// -------------------- Pointers (Box, Rc) ------------------------
// ----------------------------------------------------------------

/// Box-like type that is a pointer to an owned heap-allocated GlobalSymTable.
enum GlobalSymTableBox {
    Ptr(*mut GlobalSymTable),
}

/// Allocate and box a GlobalSymTable value on the heap.
fn globalSymTableBox_new(symtable: GlobalSymTable) -> GlobalSymTableBox {
    let ptr_u8: *mut u8 = alloc(
        std::mem::size_of::<GlobalSymTable>(),
        std::mem::size_of::<usize>(),
    );
    let ptr: *mut GlobalSymTable = ptr_u8 as *mut GlobalSymTable;
    unsafe { *ptr = symtable };
    GlobalSymTableBox::Ptr(ptr)
}

/// Dereference a GlobalSymTable box.
fn globalSymTableBox_deref(ptr_wrap: &GlobalSymTableBox) -> &GlobalSymTable {
    let GlobalSymTableBox::Ptr(ptr): &GlobalSymTableBox = ptr_wrap;
    unsafe { &**ptr }
}

/// Box-like type that is a pointer to an owned heap-allocated LocalSymTableStack.
enum LocalSymTableStackBox {
    Ptr(*mut LocalSymTableStack),
}

/// Allocate and box a LocalSymTableStack value on the heap.
fn localSymTableStackBox_new(stack: LocalSymTableStack) -> LocalSymTableStackBox {
    let ptr_u8: *mut u8 = alloc(
        std::mem::size_of::<LocalSymTableStack>(),
        std::mem::size_of::<usize>(),
    );
    let ptr: *mut LocalSymTableStack = ptr_u8 as *mut LocalSymTableStack;
    unsafe { *ptr = stack };
    LocalSymTableStackBox::Ptr(ptr)
}

/// Dereference a LocalSymTableStack box.
fn localSymTableStackBox_deref(ptr_wrap: &LocalSymTableStackBox) -> &LocalSymTableStack {
    let LocalSymTableStackBox::Ptr(ptr): &LocalSymTableStackBox = ptr_wrap;
    unsafe { &**ptr }
}

/// Box-like type that is a pointer to an owned heap-allocated LocalSymTable.
enum LocalSymTableBox {
    Ptr(*mut LocalSymTable),
}

/// Allocate and box a LocalSymTable value on the heap.
fn localSymTableBox_new(symtable: LocalSymTable) -> LocalSymTableBox {
    let ptr_u8: *mut u8 = alloc(
        std::mem::size_of::<LocalSymTable>(),
        std::mem::size_of::<usize>(),
    );
    let ptr: *mut LocalSymTable = ptr_u8 as *mut LocalSymTable;
    unsafe { *ptr = symtable };
    LocalSymTableBox::Ptr(ptr)
}

/// Dereference a LocalSymTable box.
fn localSymTableBox_deref(ptr_wrap: &LocalSymTableBox) -> &LocalSymTable {
    let LocalSymTableBox::Ptr(ptr): &LocalSymTableBox = ptr_wrap;
    unsafe { &**ptr }
}

/// Box-like type that is a pointer to an owned heap-allocated Type.
enum TypeBox {
    Ptr(*mut Type),
}

/// Allocate and box a Type value on the heap.
fn typeBox_new(ty: Type) -> TypeBox {
    let ptr_u8: *mut u8 = alloc(std::mem::size_of::<Type>(), std::mem::size_of::<usize>());
    let ptr: *mut Type = ptr_u8 as *mut Type;
    unsafe { *ptr = ty };
    TypeBox::Ptr(ptr)
}

/// Dereference a Type box.
fn typeBox_deref(ptr_wrap: &TypeBox) -> &Type {
    let TypeBox::Ptr(ptr): &TypeBox = ptr_wrap;
    unsafe { &**ptr }
}

/// Box-like type that is a pointer to an owned heap-allocated TypeList.
enum TypeListBox {
    Ptr(*mut TypeList),
}

/// Allocate and box a TypeList value on the heap.
fn typeListBox_new(types: TypeList) -> TypeListBox {
    let ptr_u8: *mut u8 = alloc(
        std::mem::size_of::<TypeList>(),
        std::mem::size_of::<usize>(),
    );
    let ptr: *mut TypeList = ptr_u8 as *mut TypeList;
    unsafe { *ptr = types };
    TypeListBox::Ptr(ptr)
}

/// Dereference a TypeList box.
fn typeListBox_deref(ptr_wrap: &TypeListBox) -> &TypeList {
    let TypeListBox::Ptr(ptr): &TypeListBox = ptr_wrap;
    unsafe { &**ptr }
}

/// Mutably dereference a TypeList box.
fn typeListBox_deref_mut(ptr_wrap: &mut TypeListBox) -> &mut TypeList {
    let TypeListBox::Ptr(ptr): &mut TypeListBox = ptr_wrap;
    unsafe { &mut **ptr }
}

// ----------------------------------------------------------------
// --------------------------- Eq ---------------------------------
// ----------------------------------------------------------------

/// Check if two tokens are equal.
fn token_eq(a: &Token, b: &Token) -> bool {
    match a {
        Token::Unsafe => match b {
            Token::Unsafe => true,
            _ => false,
        },
        Token::Fn => match b {
            Token::Fn => true,
            _ => false,
        },
        Token::Enum => match b {
            Token::Enum => true,
            _ => false,
        },
        Token::Let => match b {
            Token::Let => true,
            _ => false,
        },
        Token::If => match b {
            Token::If => true,
            _ => false,
        },
        Token::Else => match b {
            Token::Else => true,
            _ => false,
        },
        Token::While => match b {
            Token::While => true,
            _ => false,
        },
        Token::Return => match b {
            Token::Return => true,
            _ => false,
        },
        Token::Match => match b {
            Token::Match => true,
            _ => false,
        },
        Token::As => match b {
            Token::As => true,
            _ => false,
        },
        Token::Mut => match b {
            Token::Mut => true,
            _ => false,
        },
        Token::Ampersand => match b {
            Token::Ampersand => true,
            _ => false,
        },
        Token::LBrace => match b {
            Token::LBrace => true,
            _ => false,
        },
        Token::RBrace => match b {
            Token::RBrace => true,
            _ => false,
        },
        Token::LParen => match b {
            Token::LParen => true,
            _ => false,
        },
        Token::RParen => match b {
            Token::RParen => true,
            _ => false,
        },
        Token::Colon => match b {
            Token::Colon => true,
            _ => false,
        },
        Token::DoubleColon => match b {
            Token::DoubleColon => true,
            _ => false,
        },
        Token::SemiColon => match b {
            Token::SemiColon => true,
            _ => false,
        },
        Token::Comma => match b {
            Token::Comma => true,
            _ => false,
        },
        Token::Assign => match b {
            Token::Assign => true,
            _ => false,
        },
        Token::Bang => match b {
            Token::Bang => true,
            _ => false,
        },
        Token::Cmp(left_comparison) => match b {
            Token::Cmp(right_comparison) => comparison_eq(left_comparison, right_comparison),
            _ => false,
        },
        Token::ArmArrow => match b {
            Token::ArmArrow => true,
            _ => false,
        },
        Token::Plus => match b {
            Token::Plus => true,
            _ => false,
        },
        Token::Minus => match b {
            Token::Minus => true,
            _ => false,
        },
        Token::Star => match b {
            Token::Star => true,
            _ => false,
        },
        Token::Slash => match b {
            Token::Slash => true,
            _ => false,
        },
        Token::Remainder => match b {
            Token::Remainder => true,
            _ => false,
        },
        Token::Usize => match b {
            Token::Usize => true,
            _ => false,
        },
        Token::U8 => match b {
            Token::U8 => true,
            _ => false,
        },
        Token::Bool => match b {
            Token::Bool => true,
            _ => false,
        },
        Token::Char => match b {
            Token::Char => true,
            _ => false,
        },
        Token::Str => match b {
            Token::Str => true,
            _ => false,
        },
        Token::TypeArrow => match b {
            Token::TypeArrow => true,
            _ => false,
        },
        Token::Literal(left_literal) => match b {
            Token::Literal(right_literal) => literalToken_eq(left_literal, right_literal),
            _ => false,
        },
        Token::SizeOf(left) => match b {
            Token::SizeOf(right) => left == right,
            _ => false,
        },
        Token::Identifier(left) => match b {
            Token::Identifier(right) => string_eq(left, right),
            _ => false,
        },
        Token::Eof => match b {
            Token::Eof => true,
            _ => false,
        },
    }
}

/// Check if two comparison tokens are equal.
fn comparison_eq(left: &Comparison, right: &Comparison) -> bool {
    match left {
        Comparison::Eq => match right {
            Comparison::Eq => true,
            _ => false,
        },
        Comparison::Neq => match right {
            Comparison::Neq => true,
            _ => false,
        },
        Comparison::Gt => match right {
            Comparison::Gt => true,
            _ => false,
        },
        Comparison::Lt => match right {
            Comparison::Lt => true,
            _ => false,
        },
        Comparison::Geq => match right {
            Comparison::Geq => true,
            _ => false,
        },
        Comparison::Leq => match right {
            Comparison::Leq => true,
            _ => false,
        },
    }
}

/// Check if two literal tokens are equal.
fn literalToken_eq(left: &Literal, right: &Literal) -> bool {
    match left {
        Literal::Int(left_value) => match right {
            Literal::Int(right_value) => left_value == right_value,
            _ => false,
        },
        Literal::String(left_value) => match right {
            Literal::String(right_value) => string_eq(left_value, right_value),
            _ => false,
        },
        Literal::Char(left_value) => match right {
            Literal::Char(right_value) => left_value == right_value,
            _ => false,
        },
        Literal::Bool(left_value) => match right {
            Literal::Bool(right_value) => left_value == right_value,
            _ => false,
        },
    }
}

/// Check two types for equality.
fn type_eq(a: &Type, b: &Type) -> bool {
    match a {
        Type::U8 => match b {
            Type::U8 => true,
            _ => false,
        },
        Type::Usize => match b {
            Type::Usize => true,
            _ => false,
        },
        Type::Bool => match b {
            Type::Bool => true,
            _ => false,
        },
        Type::Char => match b {
            Type::Char => true,
            _ => false,
        },
        Type::Unit => match b {
            Type::Unit => true,
            _ => false,
        },
        Type::Never => match b {
            Type::Never => true,
            _ => false,
        },
        Type::Custom(left) => match b {
            Type::Custom(right) => string_eq(left, right),
            _ => false,
        },
        Type::Reference(left) => match b {
            Type::Reference(right) => type_eq(typeBox_deref(left), typeBox_deref(right)),
            _ => false,
        },
        Type::ReferenceMut(left) => match b {
            Type::ReferenceMut(right) => type_eq(typeBox_deref(left), typeBox_deref(right)),
            _ => false,
        },
        Type::RawPointerMut(left) => match b {
            Type::RawPointerMut(right) => type_eq(typeBox_deref(left), typeBox_deref(right)),
            _ => false,
        },
    }
}

/// Compare two TypeLists in order.
fn typeList_eq(left: &TypeList, right: &TypeList) -> bool {
    match left {
        TypeList::Nil => match right {
            TypeList::Nil => true,
            _ => false,
        },
        TypeList::Cons(lhead, ltail) => match right {
            TypeList::Cons(rhead, rtail) => and(
                type_eq(lhead, rhead),
                typeList_eq(typeListBox_deref(ltail), typeListBox_deref(rtail)),
            ),
            _ => false,
        },
    }
}

/// Check two LLVM tokens for equality.
fn llvmToken_eq(left: &LlvmToken, right: &LlvmToken) -> bool {
    match left {
        LlvmToken::Define => match right {
            LlvmToken::Define => true,
            _ => false,
        },
        LlvmToken::Ret => match right {
            LlvmToken::Ret => true,
            _ => false,
        },
        LlvmToken::Br => match right {
            LlvmToken::Br => true,
            _ => false,
        },
        LlvmToken::Label => match right {
            LlvmToken::Label => true,
            _ => false,
        },
        LlvmToken::Add => match right {
            LlvmToken::Add => true,
            _ => false,
        },
        LlvmToken::Sub => match right {
            LlvmToken::Sub => true,
            _ => false,
        },
        LlvmToken::Mul => match right {
            LlvmToken::Mul => true,
            _ => false,
        },
        LlvmToken::Udiv => match right {
            LlvmToken::Udiv => true,
            _ => false,
        },
        LlvmToken::Urem => match right {
            LlvmToken::Urem => true,
            _ => false,
        },
        LlvmToken::Icmp => match right {
            LlvmToken::Icmp => true,
            _ => false,
        },
        LlvmToken::Call => match right {
            LlvmToken::Call => true,
            _ => false,
        },
        LlvmToken::Gep => match right {
            LlvmToken::Gep => true,
            _ => false,
        },
        LlvmToken::Constant => match right {
            LlvmToken::Constant => true,
            _ => false,
        },
        LlvmToken::Ult => match right {
            LlvmToken::Ult => true,
            _ => false,
        },
        LlvmToken::Ptr => match right {
            LlvmToken::Ptr => true,
            _ => false,
        },
        LlvmToken::I64 => match right {
            LlvmToken::I64 => true,
            _ => false,
        },
        LlvmToken::I32 => match right {
            LlvmToken::I32 => true,
            _ => false,
        },
        LlvmToken::I8 => match right {
            LlvmToken::I8 => true,
            _ => false,
        },
        LlvmToken::I1 => match right {
            LlvmToken::I1 => true,
            _ => false,
        },
        LlvmToken::Void => match right {
            LlvmToken::Void => true,
            _ => false,
        },
        LlvmToken::At => match right {
            LlvmToken::At => true,
            _ => false,
        },
        LlvmToken::Percent => match right {
            LlvmToken::Percent => true,
            _ => false,
        },
        LlvmToken::LParen => match right {
            LlvmToken::LParen => true,
            _ => false,
        },
        LlvmToken::RParen => match right {
            LlvmToken::RParen => true,
            _ => false,
        },
        LlvmToken::LBrace => match right {
            LlvmToken::LBrace => true,
            _ => false,
        },
        LlvmToken::RBrace => match right {
            LlvmToken::RBrace => true,
            _ => false,
        },
        LlvmToken::LBracket => match right {
            LlvmToken::LBracket => true,
            _ => false,
        },
        LlvmToken::RBracket => match right {
            LlvmToken::RBracket => true,
            _ => false,
        },
        LlvmToken::Comma => match right {
            LlvmToken::Comma => true,
            _ => false,
        },
        LlvmToken::Minus => match right {
            LlvmToken::Minus => true,
            _ => false,
        },
        LlvmToken::Assign => match right {
            LlvmToken::Assign => true,
            _ => false,
        },
        LlvmToken::Colon => match right {
            LlvmToken::Colon => true,
            _ => false,
        },
        LlvmToken::CString(left_value) => match right {
            LlvmToken::CString(right_value) => string_eq(left_value, right_value),
            _ => false,
        },
        LlvmToken::Identifier(left_name) => match right {
            LlvmToken::Identifier(right_name) => string_eq(left_name, right_name),
            _ => false,
        },
        LlvmToken::Integer(left_value) => match right {
            LlvmToken::Integer(right_value) => *left_value == *right_value,
            _ => false,
        },
        LlvmToken::Eof => match right {
            LlvmToken::Eof => true,
            _ => false,
        },
    }
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

// ----------------------------------------------------------------
// ------------------------- Clone --------------------------------
// ----------------------------------------------------------------

/// Clone a token value.
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
        Token::Bang => Token::Bang,
        Token::Cmp(comparison) => Token::Cmp(comparison_clone(comparison)),
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
        Token::Literal(literal) => Token::Literal(literalToken_clone(literal)),
        Token::Identifier(value) => Token::Identifier(string_clone(value)),
        Token::SizeOf(value) => Token::SizeOf(*value),
        Token::Eof => Token::Eof,
    }
}

/// Clone a comparison operator.
fn comparison_clone(comparison: &Comparison) -> Comparison {
    match comparison {
        Comparison::Eq => Comparison::Eq,
        Comparison::Neq => Comparison::Neq,
        Comparison::Gt => Comparison::Gt,
        Comparison::Lt => Comparison::Lt,
        Comparison::Geq => Comparison::Geq,
        Comparison::Leq => Comparison::Leq,
    }
}

/// Clone a literal token payload.
fn literalToken_clone(literal: &Literal) -> Literal {
    match literal {
        Literal::Int(value) => Literal::Int(*value),
        Literal::String(value) => Literal::String(string_clone(value)),
        Literal::Char(value) => Literal::Char(*value),
        Literal::Bool(value) => Literal::Bool(*value),
    }
}

/// Clone a symbol table entry.
fn symTableEntry_clone(entry: &SymTableEntry) -> SymTableEntry {
    match entry {
        SymTableEntry::Function(name, signature) => {
            SymTableEntry::Function(string_clone(name), fnSignature_clone(signature))
        }
        SymTableEntry::Enum(name, variants) => {
            SymTableEntry::Enum(string_clone(name), typeList_clone(variants))
        }
        SymTableEntry::Variable(name, variable_type, mutable) => {
            SymTableEntry::Variable(string_clone(name), type_clone(variable_type), *mutable)
        }
    }
}

/// Clone the global symbol table.
fn globalSymTable_clone(symtable: &GlobalSymTable) -> GlobalSymTable {
    match symtable {
        GlobalSymTable::Nil => GlobalSymTable::Nil,
        GlobalSymTable::Cons(head, tail) => {
            GlobalSymTable::Cons(symTableEntry_clone(head), globalSymTableBox_clone(tail))
        }
    }
}

/// Clone a local scope symbol table.
fn localSymTable_clone(symtable: &LocalSymTable) -> LocalSymTable {
    match symtable {
        LocalSymTable::Nil => LocalSymTable::Nil,
        LocalSymTable::Cons(head, tail) => {
            LocalSymTable::Cons(symTableEntry_clone(head), localSymTableBox_clone(tail))
        }
    }
}

/// Clone the stack of local scopes.
fn localSymTableStack_clone(stack: &LocalSymTableStack) -> LocalSymTableStack {
    match stack {
        LocalSymTableStack::Nil => LocalSymTableStack::Nil,
        LocalSymTableStack::Cons(local, tail) => LocalSymTableStack::Cons(
            localSymTable_clone(local),
            localSymTableStackBox_clone(tail),
        ),
    }
}

/// Clone a function signature.
fn fnSignature_clone(signature: &FnSignature) -> FnSignature {
    match signature {
        FnSignature::Fn(parameter_types, return_type) => {
            FnSignature::Fn(typeList_clone(parameter_types), type_clone(return_type))
        }
    }
}

/// Clone a type value.
fn type_clone(t: &Type) -> Type {
    match t {
        Type::U8 => Type::U8,
        Type::Usize => Type::Usize,
        Type::Bool => Type::Bool,
        Type::Char => Type::Char,
        Type::Unit => Type::Unit,
        Type::Never => Type::Never,
        Type::Custom(name) => Type::Custom(string_clone(name)),
        Type::Reference(inner) => Type::Reference(typeBox_clone(inner)),
        Type::ReferenceMut(inner) => Type::ReferenceMut(typeBox_clone(inner)),
        Type::RawPointerMut(inner) => Type::RawPointerMut(typeBox_clone(inner)),
    }
}

/// Clone an LLVM token.
fn llvmToken_clone(token: &LlvmToken) -> LlvmToken {
    match token {
        LlvmToken::Define => LlvmToken::Define,
        LlvmToken::Ret => LlvmToken::Ret,
        LlvmToken::Br => LlvmToken::Br,
        LlvmToken::Label => LlvmToken::Label,
        LlvmToken::Add => LlvmToken::Add,
        LlvmToken::Sub => LlvmToken::Sub,
        LlvmToken::Mul => LlvmToken::Mul,
        LlvmToken::Udiv => LlvmToken::Udiv,
        LlvmToken::Urem => LlvmToken::Urem,
        LlvmToken::Icmp => LlvmToken::Icmp,
        LlvmToken::Call => LlvmToken::Call,
        LlvmToken::Gep => LlvmToken::Gep,
        LlvmToken::Constant => LlvmToken::Constant,
        LlvmToken::Ult => LlvmToken::Ult,
        LlvmToken::Ptr => LlvmToken::Ptr,
        LlvmToken::I64 => LlvmToken::I64,
        LlvmToken::I32 => LlvmToken::I32,
        LlvmToken::I8 => LlvmToken::I8,
        LlvmToken::I1 => LlvmToken::I1,
        LlvmToken::Void => LlvmToken::Void,
        LlvmToken::At => LlvmToken::At,
        LlvmToken::Percent => LlvmToken::Percent,
        LlvmToken::LParen => LlvmToken::LParen,
        LlvmToken::RParen => LlvmToken::RParen,
        LlvmToken::LBrace => LlvmToken::LBrace,
        LlvmToken::RBrace => LlvmToken::RBrace,
        LlvmToken::LBracket => LlvmToken::LBracket,
        LlvmToken::RBracket => LlvmToken::RBracket,
        LlvmToken::Comma => LlvmToken::Comma,
        LlvmToken::Minus => LlvmToken::Minus,
        LlvmToken::Assign => LlvmToken::Assign,
        LlvmToken::Colon => LlvmToken::Colon,
        LlvmToken::CString(value) => LlvmToken::CString(string_clone(value)),
        LlvmToken::Identifier(name) => LlvmToken::Identifier(string_clone(name)),
        LlvmToken::Integer(value) => LlvmToken::Integer(*value),
        LlvmToken::Eof => LlvmToken::Eof,
    }
}

/// Clone a TypeList linked list.
fn typeList_clone(types: &TypeList) -> TypeList {
    match types {
        TypeList::Nil => TypeList::Nil,
        TypeList::Cons(head, tail) => TypeList::Cons(type_clone(head), typeListBox_clone(tail)),
    }
}

/// Clone a GlobalSymTable box and its heap-owned value.
fn globalSymTableBox_clone(ptr: &GlobalSymTableBox) -> GlobalSymTableBox {
    let cloned: GlobalSymTable = globalSymTable_clone(globalSymTableBox_deref(ptr));
    globalSymTableBox_new(cloned)
}

/// Clone a LocalSymTableStack box and its heap-owned value.
fn localSymTableStackBox_clone(ptr: &LocalSymTableStackBox) -> LocalSymTableStackBox {
    let cloned: LocalSymTableStack = localSymTableStack_clone(localSymTableStackBox_deref(ptr));
    localSymTableStackBox_new(cloned)
}

/// Clone a LocalSymTable box and its heap-owned value.
fn localSymTableBox_clone(ptr: &LocalSymTableBox) -> LocalSymTableBox {
    let cloned: LocalSymTable = localSymTable_clone(localSymTableBox_deref(ptr));
    localSymTableBox_new(cloned)
}

/// Clone a Type box and its heap-owned value.
fn typeBox_clone(ptr: &TypeBox) -> TypeBox {
    let cloned: Type = type_clone(typeBox_deref(ptr));
    typeBox_new(cloned)
}

/// Clone a TypeList box and its heap-owned value.
fn typeListBox_clone(ptr: &TypeListBox) -> TypeListBox {
    let cloned: TypeList = typeList_clone(typeListBox_deref(ptr));
    typeListBox_new(cloned)
}

/// Clone a string.
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

// ------------------------- String -------------------------------

/// A growable ASCII string.
#[derive(Debug)]
enum String {
    /// start, length, capacity
    String(*mut u8, usize, usize),
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

/// Create a string from a string slice.
fn string_from_str(str: &str) -> String {
    let mut s: String = string_new();
    string_push_str(&mut s, str);
    s
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

/// Get the character at the given index.
fn string_get(string: &String, index: usize) -> CharOption {
    if index >= string_len(string) {
        CharOption::None
    } else {
        let ptr: *mut u8 = ptr_add(string_ptr(string), index);
        unsafe { CharOption::Some(*ptr as char) }
    }
}

/// Append a character to the string.
fn string_push(string: &mut String, character: char) {
    string_accomodate_extra_space(string, 1);
    let String::String(ptr, len, _): &mut String = string;
    unsafe { *ptr_add(*ptr, *len) = character as u8 }
    *len = *len + 1;
}

/// Append a string slice to the string.
fn string_push_str(string: &mut String, str: &str) {
    let str_len: usize = str.len();
    string_accomodate_extra_space(string, str_len);

    let str_ptr: *mut u8 = str.as_ptr() as *mut u8;

    let String::String(string_ptr, len, _): &mut String = string;
    let string_end: *mut u8 = ptr_add(*string_ptr, *len);

    unsafe { memcopy(string_end, str_ptr, str_len) }
    *len = *len + str_len;
}

/// Push a string onto another string.
fn string_push_string(string: &mut String, other: &String) {
    let other_len: usize = string_len(other);
    string_accomodate_extra_space(string, other_len);

    let other_ptr: *mut u8 = string_ptr(other);

    let String::String(ptr, len, _): &mut String = string;
    let ptr: *mut u8 = ptr_add(*ptr, *len);

    unsafe { memcopy(ptr, other_ptr, other_len) }
    *len = *len + other_len;
}

/// Converts a string into an integer given the base.
/// Returns None if the integer contained in the string is invalid for 64-bit integers.
fn string_to_integer(string: &mut String, base: usize) -> UsizeOption {
    let mut value: usize = 0;

    let mut i: usize = 0;
    while i < string_len(string) {
        let digit: char = unwrap_char(string_get(string, i));

        let digit_value: usize = if is_digit(digit) {
            digit as usize - '0' as usize
        } else {
            digit as usize - 'A' as usize + 10
        };

        let max: usize = 18446744073709551615; // 2^64 - 1

        if or(digit_value > base - 1, value > max / base) {
            return UsizeOption::None;
        }

        value = value * base + digit_value;

        i = i + 1;
    }
    UsizeOption::Some(value)
}

/// Hash a String.
fn string_hash(string: &String, bucket_count: usize) -> usize {
    if bucket_count == 0 {
        return 0;
    }

    let mut hash: usize = 0;
    let mut i: usize = 0;
    while i < string_len(string) {
        let character: usize = unwrap_char(string_get(string, i)) as usize;
        hash = hash * 67 + character;
        i = i + 1;
    }
    hash % bucket_count
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
