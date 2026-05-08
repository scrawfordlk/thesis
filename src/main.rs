#![allow(clippy::assign_op_pattern, while_true, non_snake_case)]

fn main() {
    let args: std::vec::Vec<std::string::String> = std::env::args().collect();

    if args.len() > 1 {
        let code: String =
            parse_to_llvm(&std::fs::read_to_string(&args[1]).expect("no program found"));

        let String::Inner(vec): String = code;
        let mut file = std::fs::File::create("code.ll").expect("can create file");
        use std::io::Write;
        let slice = unsafe { core::slice::from_raw_parts(vec_ptr(&vec), vec_len(&vec)) };
        file.write_all(slice).expect("can write all code");
        std::io::stdout()
            .write_all(slice)
            .expect("print code to terminal");
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
    /// source, next character index, current line, character index of last newline
    SourceFile(String, usize, usize, usize),
}

/// Get the character at the given index.
fn sourceFile_get_char(file: &SourceFile, index: usize) -> Option<char> {
    let SourceFile::SourceFile(string, _, _, _): &SourceFile = file;
    string_get(string, index)
}

/// Returns the current line.
fn sourceFile_current_line(file: &SourceFile) -> usize {
    let SourceFile::SourceFile(_, _, line, _): &SourceFile = file;
    *line
}

/// Returns the current column in the current line.
fn sourceFile_current_column(file: &SourceFile) -> usize {
    let SourceFile::SourceFile(_, next_char_idx, _, last_newline_idx): &SourceFile = file;
    *next_char_idx - *last_newline_idx
}

/// Returns the index of the beginning of the current line.
fn sourceFile_current_line_start(file: &SourceFile) -> usize {
    let SourceFile::SourceFile(_, _, _, last_newline_idx): &SourceFile = file;
    *last_newline_idx + 1
}

/// Create a lexer and prime it with the first token.
fn lexer_new(source: String) -> Lexer {
    let source_file: SourceFile = SourceFile::SourceFile(source, 0, 0, 0);
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

/// Peek at the next character without consuming it.
fn lexer_peek_char(lexer: &Lexer) -> Option<char> {
    let SourceFile::SourceFile(string, index, _, _): &SourceFile = lexer_sourcefile(lexer);
    string_get(string, *index)
}

/// Consume and return the next character.
fn lexer_consume_char(lexer: &mut Lexer) -> Option<char> {
    let SourceFile::SourceFile(source, index, line, last_newline_idx): &mut SourceFile =
        lexer_sourcefile_mut(lexer);

    let current: Option<char> = string_get(source, *index);
    *index = *index + 1;

    match current {
        Option::Some(character) => {
            if character == '\n' {
                *line = *line + 1;
                *last_newline_idx = *index;
            }
        }
        Option::None => {}
    }
    current
}

/// Consume the next character, erroring if it doesn't match expected.
fn lexer_expect_char(lexer: &mut Lexer, expected: char) {
    match lexer_consume_char(lexer) {
        Option::Some(c) => {
            if c != expected {
                lexer_error(lexer, "unexpected character");
            }
        }
        Option::None => lexer_error(lexer, "unexpected end of input"),
    }
}

// ---------------------- Lexer ----------------------

/// Consume and return the next token.
fn lexer_next_token(lexer: &mut Lexer) -> Token {
    skip_whitespace(lexer);

    let token: Token = match lexer_peek_char(lexer) {
        Option::Some(c) => {
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
        Option::None => Token::Eof,
    };

    lexer_set_current_token(lexer, token_clone(&token));
    token
}

/// Scan an identifier or keyword.
fn lexer_scan_identifier(lexer: &mut Lexer) -> String {
    let mut ident: String = string_new();
    while true {
        match lexer_peek_char(lexer) {
            Option::Some(c) => {
                if is_alphanumeric(c) {
                    lexer_consume_char(lexer);
                    string_push(&mut ident, c);
                } else {
                    return ident;
                }
            }
            Option::None => return ident,
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
            Option::Some(c) => {
                if is_digit(c) {
                    value = value * 10 + (c as usize) - ('0' as usize);
                    lexer_consume_char(lexer);
                } else {
                    return value;
                }
            }
            Option::None => return value,
        }
    }
    value // satisfy compiler
}

fn lexer_scan_char_literal(lexer: &mut Lexer) -> char {
    lexer_expect_char(lexer, '\'');
    let c: char = match lexer_consume_char(lexer) {
        Option::Some('\\') => lexer_scan_escape_char(lexer),
        Option::Some(ch) => ch,
        Option::None => lexer_error(lexer, "unexpected end of char literal"),
    };
    lexer_expect_char(lexer, '\'');
    c
}

fn lexer_scan_string_literal(lexer: &mut Lexer) -> String {
    lexer_expect_char(lexer, '"');
    let mut s: String = string_new();
    while true {
        match lexer_consume_char(lexer) {
            Option::Some('"') => return s,
            Option::Some('\\') => string_push(&mut s, lexer_scan_escape_char(lexer)),
            Option::Some(c) => string_push(&mut s, c),
            Option::None => lexer_error(lexer, "unexpected end of string literal"),
        }
    }
    s // satisfy compiler
}

/// Scan an escape sequence after backslash.
fn lexer_scan_escape_char(lexer: &mut Lexer) -> char {
    match lexer_consume_char(lexer) {
        Option::Some('n') => '\n',
        Option::Some('t') => '\t',
        Option::Some('r') => '\r',
        Option::Some('0') => '\0',
        Option::Some(c) => c,
        Option::None => lexer_error(lexer, "unexpected end of escape sequence"),
    }
}

fn lexer_scan_symbol(lexer: &mut Lexer) -> Token {
    match unwrap::<char>(lexer_consume_char(lexer)) {
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
        Option::Some('/') => {
            lexer_consume_char(lexer);
            skip_line_comment(lexer);
            lexer_next_token(lexer)
        }
        _ => Token::Slash,
    }
}

fn lexer_scan_colon(lexer: &mut Lexer) -> Token {
    match lexer_peek_char(lexer) {
        Option::Some(':') => {
            lexer_consume_char(lexer);
            Token::DoubleColon
        }
        _ => Token::Colon,
    }
}

fn lexer_scan_equals(lexer: &mut Lexer) -> Token {
    match lexer_peek_char(lexer) {
        Option::Some('=') => {
            lexer_consume_char(lexer);
            Token::Cmp(Comparison::Eq)
        }
        Option::Some('>') => {
            lexer_consume_char(lexer);
            Token::ArmArrow
        }
        _ => Token::Assign,
    }
}

fn lexer_scan_minus(lexer: &mut Lexer) -> Token {
    match lexer_peek_char(lexer) {
        Option::Some('>') => {
            lexer_consume_char(lexer);
            Token::TypeArrow
        }
        _ => Token::Minus,
    }
}

fn lexer_scan_bang(lexer: &mut Lexer) -> Token {
    match lexer_peek_char(lexer) {
        Option::Some('=') => {
            lexer_consume_char(lexer);
            Token::Cmp(Comparison::Neq)
        }
        _ => Token::Bang,
    }
}

fn lexer_scan_less(lexer: &mut Lexer) -> Token {
    match lexer_peek_char(lexer) {
        Option::Some('=') => {
            lexer_consume_char(lexer);
            Token::Cmp(Comparison::Leq)
        }
        _ => Token::Cmp(Comparison::Lt),
    }
}

fn lexer_scan_greater(lexer: &mut Lexer) -> Token {
    match lexer_peek_char(lexer) {
        Option::Some('=') => {
            lexer_consume_char(lexer);
            Token::Cmp(Comparison::Geq)
        }
        _ => Token::Cmp(Comparison::Gt),
    }
}

fn skip_whitespace(lexer: &mut Lexer) {
    while true {
        match lexer_peek_char(lexer) {
            Option::Some(c) => {
                if is_whitespace(c) {
                    lexer_consume_char(lexer);
                } else {
                    return;
                }
            }
            Option::None => return,
        }
    }
}

fn skip_line_comment(lexer: &mut Lexer) {
    while true {
        match lexer_consume_char(lexer) {
            Option::Some('\n') => return,
            Option::Some(_) => (),
            Option::None => return,
        }
    }
}

// -------------------------- Parser -------------------------------

/// Type that encapsulates the parser's state..
enum Parser {
    /// lexer, llvm code, symbol table, current function return type, LLVM context
    Parser(Lexer, String, SymTable, Type, Context),
}

/// Create a parser from a String.
fn parser_new(source: String) -> Parser {
    Parser::Parser(
        lexer_new(source),
        string_new(),
        symTable_new(),
        Type::Unit,
        context_new(),
    )
}

/// Create a parser from a string slice.
fn parser_from_str(source: &str) -> Parser {
    parser_new(string_from_str(source))
}

/// Get immutable access to the parser lexer.
fn parser_lexer(parser: &Parser) -> &Lexer {
    let Parser::Parser(lexer, _, _, _, _): &Parser = parser;
    lexer
}

/// Get mutable access to the parser lexer.
fn parser_lexer_mut(parser: &mut Parser) -> &mut Lexer {
    let Parser::Parser(lexer, _, _, _, _): &mut Parser = parser;
    lexer
}

/// Get immutable access to the parser LLVM output buffer.
fn parser_llvm(parser: &Parser) -> &String {
    let Parser::Parser(_, llvm, _, _, _): &Parser = parser;
    llvm
}

/// Get mutable access to the parser LLVM output buffer.
fn parser_llvm_mut(parser: &mut Parser) -> &mut String {
    let Parser::Parser(_, llvm, _, _, _): &mut Parser = parser;
    llvm
}

/// Get immutable access to the parser symbol table.
fn parser_symtable(parser: &Parser) -> &SymTable {
    let Parser::Parser(_, _, symTable, _, _): &Parser = parser;
    symTable
}

/// Get mutable access to the parser symbol table.
fn parser_symtable_mut(parser: &mut Parser) -> &mut SymTable {
    let Parser::Parser(_, _, symTable, _, _): &mut Parser = parser;
    symTable
}

/// Get the expected return type of the current function.
fn parser_current_fn_return_type(parser: &Parser) -> &Type {
    let Parser::Parser(_, _, _, return_type, _): &Parser = parser;
    return_type
}

/// Update the expected return type of the current function.
fn parser_set_current_fn_return_type(parser: &mut Parser, ty: Type) {
    let Parser::Parser(_, _, _, return_type, _): &mut Parser = parser;
    *return_type = ty;
}

/// Get a mutable reference to the LLVM context.
fn parser_context_mut(parser: &mut Parser) -> &mut Context {
    let Parser::Parser(_, _, _, _, context): &mut Parser = parser;
    context
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

/// Pair that contains a String and a Rust Type
enum STPair {
    ST(String, Type),
}

fn stPair_get_type(pair: STPair) -> Type {
    let STPair::ST(_, ty): STPair = pair;
    ty
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
    let mut parameter_types: List<Type> = list_new::<Type>();

    parser_expect_token(parser, &Token::LParen);
    if not(parser_current_token_eq(parser, &Token::RParen)) {
        let first_parameter: Variable = parse_variable(parser);
        let Variable::Var(pattern, param_type, is_mutable): Variable = first_parameter;
        list_append::<Type>(&mut parameter_types, type_clone(&param_type));

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
            list_append::<Type>(&mut parameter_types, type_clone(&param_type));

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

    let STPair::ST(name, block_type): STPair = parse_block(parser);
    parser_expect_same_type(parser, &block_type, &function_return_type);

    match function_return_type {
        Type::Unit => llvm_emit_ret_void(parser),
        _ => llvm_emit_ret_value(parser, &block_type, &name),
    }

    llvm_emit_line(parser_llvm_mut(parser), "}");
    symTable_leave_scope(parser_symtable_mut(parser));
    parser_set_current_fn_return_type(parser, Type::Unit);
}

fn parse_enum(parser: &mut Parser) {
    parser_expect_token(parser, &Token::Enum);
    let enum_name: String = parser_expect_identifier(parser);
    parser_expect_token(parser, &Token::LBrace);

    let mut variants: List<Type> = list_new::<Type>();
    let first_variant_type: Type = parse_variant(parser);
    list_append::<Type>(&mut variants, first_variant_type);
    parser_expect_token(parser, &Token::Comma);

    while not(parser_current_token_eq(parser, &Token::RBrace)) {
        let variant_type: Type = parse_variant(parser);
        // TODO: check for duplicate variants
        list_append::<Type>(&mut variants, variant_type);
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

fn parse_block(parser: &mut Parser) -> STPair {
    parser_expect_token(parser, &Token::LBrace);

    symTable_enter_scope(parser_symtable_mut(parser));

    while not(parser_current_token_eq(parser, &Token::RBrace)) {
        match parser_current_token(parser) {
            Token::Let => {
                parse_binding(parser);
                parser_expect_token(parser, &Token::SemiColon);
            }
            _ => {
                let STPair::ST(name, ty) = parse_expression(parser);
                if parser_try_consume(parser, &Token::SemiColon) {
                    llvm_emit_line(parser_llvm_mut(parser), "");
                } else {
                    parser_expect_token(parser, &Token::RBrace);
                    symTable_leave_scope(parser_symtable_mut(parser));
                    return STPair::ST(name, ty);
                }
            }
        }
    }

    parser_expect_token(parser, &Token::RBrace);
    symTable_leave_scope(parser_symtable_mut(parser));
    STPair::ST(string_new(), Type::Unit)
}

// TODO: code generation
fn parse_binding(parser: &mut Parser) {
    parser_expect_token(parser, &Token::Let);
    let Variable::Var(pattern, binding_type, mutable): Variable = parse_variable(parser);
    parser_expect_token(parser, &Token::Assign);
    let STPair::ST(name, left_type): STPair = parse_expression(parser);

    parser_expect_same_type(parser, &binding_type, &left_type);

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
                Type::Reference(box_new::<Type>(inner), true)
            } else if parser_try_consume(parser, &Token::Str) {
                Type::Reference(box_new::<Type>(Type::Custom(string_from_str("str"))), false)
            } else {
                let inner: Type = parse_type(parser);
                Type::Reference(box_new::<Type>(inner), false)
            }
        }
        Token::Star => {
            parser_next_token(parser);
            parser_expect_token(parser, &Token::Mut);
            let inner: Type = parse_type(parser);
            Type::RawPointerMut(box_new::<Type>(inner))
        }
        Token::Identifier(_) => Type::Custom(parser_expect_identifier(parser)),
        _ => parser_error(parser, "expected type"),
    }
}

fn parse_expression(parser: &mut Parser) -> STPair {
    match parser_current_token(parser) {
        Token::Return => {
            parser_next_token(parser);

            match parser_current_token(parser) {
                Token::SemiColon | Token::RBrace => STPair::ST(string_new(), Type::Unit),

                _ => {
                    let STPair::ST(name, ty): STPair = parse_expression(parser);
                    parser_expect_same_type(parser, &ty, parser_current_fn_return_type(parser));
                    STPair::ST(name, ty)
                }
            }
        }

        _ => parse_assignment(parser),
    }
}

fn parse_assignment(parser: &mut Parser) -> STPair {
    let STPair::ST(left_name, left_type): STPair = parse_comparison(parser);

    if parser_try_consume(parser, &Token::Assign) {
        let STPair::ST(right_name, right_type): STPair = parse_assignment(parser);
        parser_expect_same_type(parser, &left_type, &right_type);

        llvm_emit_line(parser_llvm_mut(parser), "  ; assignment");

        STPair::ST(right_name, Type::Unit)
    } else {
        STPair::ST(left_name, left_type)
    }
}

fn parse_comparison(parser: &mut Parser) -> STPair {
    let STPair::ST(left_name, left_type): STPair = parse_arithmetic(parser);

    match parser_current_token(parser) {
        Token::Cmp(operator) => {
            let operator: Comparison = comparison_clone(operator);
            parser_next_token(parser);

            let STPair::ST(right_name, rtype): STPair = parse_arithmetic(parser);

            parser_expect_same_type(parser, &left_type, &rtype);
            if not(or(
                type_is_numeric(&left_type),
                type_eq(&left_type, &Type::Char),
            )) {
                parser_error(parser, "cannot compare non-integer values");
            }

            let name: String = match operator {
                Comparison::Eq => llvm_emit_icmp(parser, "eq", &rtype, &left_name, &right_name),
                Comparison::Neq => llvm_emit_icmp(parser, "ne", &rtype, &left_name, &right_name),
                Comparison::Gt => llvm_emit_icmp(parser, "ugt", &rtype, &left_name, &right_name),
                Comparison::Lt => llvm_emit_icmp(parser, "ult", &rtype, &left_name, &right_name),
                Comparison::Geq => llvm_emit_icmp(parser, "uge", &rtype, &left_name, &right_name),
                Comparison::Leq => llvm_emit_icmp(parser, "ule", &rtype, &left_name, &right_name),
            };
            STPair::ST(name, Type::Bool)
        }

        _ => STPair::ST(left_name, left_type),
    }
}

fn parse_arithmetic(parser: &mut Parser) -> STPair {
    let STPair::ST(left_name, left_type): STPair = parse_term(parser);

    match parser_current_token(parser) {
        Token::Plus | Token::Minus => {
            let operator: Token = token_clone(parser_current_token(parser));
            parser_next_token(parser);

            let STPair::ST(right_name, right_type): STPair = parse_arithmetic(parser);

            parser_expect_same_type(parser, &left_type, &right_type);
            parser_expect_numeric_type(parser, &left_type);

            let name: String = match operator {
                Token::Plus => llvm_emit_add(parser, &right_type, &left_name, &right_name),
                Token::Minus => llvm_emit_sub(parser, &right_type, &left_name, &right_name),
                _ => panic!("unreachable"),
            };
            STPair::ST(name, left_type)
        }

        _ => STPair::ST(left_name, left_type),
    }
}

fn parse_term(parser: &mut Parser) -> STPair {
    let STPair::ST(left_name, left_type): STPair = parse_cast(parser);

    match parser_current_token(parser) {
        Token::Star | Token::Slash | Token::Remainder => {
            let operator: Token = token_clone(parser_current_token(parser));
            parser_next_token(parser);

            let STPair::ST(right_name, right_type): STPair = parse_term(parser);

            parser_expect_same_type(parser, &left_type, &right_type);
            parser_expect_numeric_type(parser, &left_type);

            let name: String = match operator {
                Token::Star => llvm_emit_mul(parser, &right_type, &left_name, &right_name),
                Token::Slash => llvm_emit_udiv(parser, &right_type, &left_name, &right_name),
                Token::Remainder => llvm_emit_urem(parser, &right_type, &left_name, &right_name),
                _ => panic!("unreachable"),
            };
            STPair::ST(name, left_type)
        }

        _ => STPair::ST(left_name, left_type),
    }
}

fn parse_cast(parser: &mut Parser) -> STPair {
    let STPair::ST(mut name, mut ty): STPair = parse_unary(parser);

    while parser_try_consume(parser, &Token::As) {
        let cast_type: Type = parse_type(parser);

        match type_get_cast_operation(&ty, &cast_type) {
            CastOperation::ZeroExtend => {
                let casted_name: String = llvm_emit_zext(parser, &ty, &cast_type, &name);
                name = casted_name;
                ty = cast_type;
            }
            CastOperation::Truncate => {
                let casted_name: String = llvm_emit_trunc(parser, &ty, &cast_type, &name);
                name = casted_name;
                ty = cast_type;
            }
            CastOperation::None => {} // no-op
            CastOperation::Invalid => parser_error(parser, "invalid cast"),
        }
    }

    STPair::ST(name, ty)
}

fn parse_unary(parser: &mut Parser) -> STPair {
    match parser_current_token(parser) {
        Token::Ampersand => {
            parser_next_token(parser);
            let mutable: bool = parser_try_consume(parser, &Token::Mut);

            let STPair::ST(name, ty) = parse_unary(parser);

            let reference: String = llvm_emit_alloca(parser, &ty, 1);
            llvm_emit_store(parser, &ty, &name, &reference);

            STPair::ST(reference, Type::Reference(box_new::<Type>(ty), mutable))
        }

        Token::Star => {
            parser_next_token(parser);
            let STPair::ST(name, ty) = parse_unary(parser);

            let inner_type: Type = match ty {
                Type::Reference(pointed, _) => type_clone(box_deref::<Type>(&pointed)),
                Type::RawPointerMut(pointed) => type_clone(box_deref::<Type>(&pointed)),
                _ => parser_error(parser, "cannot dereference this expression"),
            };

            let dereferenced: String = llvm_emit_load(parser, &inner_type, &name);

            STPair::ST(dereferenced, inner_type)
        }
        _ => parse_factor(parser),
    }
}

fn parse_factor(parser: &mut Parser) -> STPair {
    match parser_current_token(parser) {
        Token::Literal(_) => parse_literal(parser),
        Token::Identifier(_) => {
            let name: String = parser_expect_identifier(parser);
            if parser_current_token_eq(parser, &Token::LParen) {
                STPair::ST(string_new(), parse_call(parser, name))
            } else {
                match symTable_lookup_variable_type(parser_symtable(parser), &name) {
                    Option::Some(ty) => STPair::ST(string_new(), ty),
                    Option::None => parser_error(parser, "undefined variable"),
                }
            }
        }
        Token::LParen => {
            parser_next_token(parser);
            let STPair::ST(name, ty): STPair = parse_expression(parser);
            parser_expect_token(parser, &Token::RParen);
            STPair::ST(name, ty)
        }
        Token::Unsafe => {
            parser_next_token(parser);
            parse_block(parser)
        }
        Token::LBrace => parse_block(parser),
        Token::If => STPair::ST(string_new(), parse_if(parser)),
        Token::While => STPair::ST(string_new(), parse_while(parser)),
        Token::Match => STPair::ST(string_new(), parse_match(parser)),
        _ => parser_error(parser, "unexpected token in parse_factor()"),
    }
}

fn parse_if(parser: &mut Parser) -> Type {
    parser_expect_token(parser, &Token::If);

    let STPair::ST(cond_name, condition_type): STPair = parse_expression(parser);
    parser_expect_bool_type(parser, &condition_type);

    let STPair::ST(then_name, then_type): STPair = parse_block(parser);
    if parser_try_consume(parser, &Token::Else) {
        let else_type: Type = if parser_current_token_eq(parser, &Token::If) {
            parse_if(parser)
        } else {
            stPair_get_type(parse_block(parser))
        };
        parser_expect_same_type(parser, &then_type, &else_type);
        then_type
    } else {
        Type::Unit
    }
}

fn parse_while(parser: &mut Parser) -> Type {
    parser_expect_token(parser, &Token::While);

    let STPair::ST(cond_name, condition_type): STPair = parse_expression(parser);
    parser_expect_bool_type(parser, &condition_type);

    parse_block(parser);

    Type::Unit
}

fn parse_match(parser: &mut Parser) -> Type {
    parser_expect_token(parser, &Token::Match);

    let STPair::ST(name, expression_type): STPair = parse_expression(parser);

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

    let return_type: Type = stPair_get_type(parse_expression(parser));
    parser_expect_token(parser, &Token::Comma);

    while not(parser_current_token_eq(parser, &Token::RBrace)) {
        let pattern: Pattern = parse_pattern(parser);
        let pattern_type: Type = pattern_type_for_expression(&pattern, matched_type);
        parser_expect_same_type(parser, &pattern_type, matched_type);

        parser_expect_token(parser, &Token::ArmArrow);

        let arm_type: Type = stPair_get_type(parse_expression(parser));
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
                Literal::String(_) => Pattern::Literal(Type::Reference(
                    box_new::<Type>(Type::Custom(string_from_str("str"))),
                    false,
                )),
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

    let mut argument_types: List<Type> = list_new::<Type>();
    if not(parser_current_token_eq(parser, &Token::RParen)) {
        let first_argument_type: Type = stPair_get_type(parse_expression(parser));
        list_append::<Type>(&mut argument_types, first_argument_type);

        while and(
            parser_try_consume(parser, &Token::Comma),
            not(parser_current_token_eq(parser, &Token::RParen)),
        ) {
            let argument_type: Type = stPair_get_type(parse_expression(parser));
            list_append::<Type>(&mut argument_types, argument_type);
        }
    }
    parser_expect_token(parser, &Token::RParen);

    match symTable_lookup_function_signature(parser_symtable(parser), &function_name) {
        Option::Some(FnSignature::Fn(parameter_types, return_type)) => {
            if not(list_eq::<Type>(&parameter_types, &argument_types, type_eq)) {
                parser_error(parser, "function call does not match function signature");
            }

            llvm_emit_call_comment(parser_llvm_mut(parser), &function_name);

            return_type
        }
        Option::None => parser_error(parser, "call to undefined function"),
    }
}

fn parse_literal(parser: &mut Parser) -> STPair {
    match parser_current_token(parser) {
        Token::Literal(literal) => {
            let current_literal: Literal = literalToken_clone(literal);
            parser_next_token(parser);

            match current_literal {
                Literal::Int(value) => STPair::ST(integer_to_string(value), Type::Usize),
                Literal::Char(value) => STPair::ST(integer_to_string(value as usize), Type::Char),
                Literal::Bool(value) => STPair::ST(integer_to_string(value as usize), Type::Bool),

                // TODO: Implement string literal
                Literal::String(_) => STPair::ST(
                    string_new(),
                    Type::Reference(box_new::<Type>(Type::Custom(string_from_str("str"))), false),
                ),
            }
        }
        _ => parser_error(parser, "expected literal"),
    }
}

/// Manages the context of the currently generated LLVM-IR.
/// It handles e.g. the already assigned virtual registers.
enum Context {
    /// temporary counter
    Context(usize),
}

fn context_new() -> Context {
    Context::Context(0)
}

fn context_get_counter(context: &Context) -> usize {
    let Context::Context(counter): &Context = context;
    *counter
}

fn context_increment_counter(context: &mut Context) {
    let Context::Context(counter): &mut Context = context;
    *counter = *counter + 1;
}

/// Get the next available virtual register name.
fn context_next_temporary(context: &mut Context) -> String {
    let id: usize = context_get_counter(context);
    context_increment_counter(context);
    let mut name: String = string_from_str("%.t"); // '.' avoids name clashes with variables
    string_push_string(&mut name, &integer_to_string(id));
    name
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
fn symTable_lookup_variable_type(symtable: &SymTable, name: &String) -> Option<Type> {
    let SymTable::Table(_, local): &SymTable = symtable;
    localSymTableStack_lookup_variable_type(local, name)
}

/// Lookup a function signature in the global symbol table.
fn symTable_lookup_function_signature(symtable: &SymTable, name: &String) -> Option<FnSignature> {
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
    parameter_types: List<Type>,
    return_type: Type,
) -> bool {
    let SymTable::Table(global, _) = symtable;
    globalSymTable_insert_function(global, name, parameter_types, return_type)
}

/// Insert an enum into the global table.
/// Return true, if the name is not taken else false.
fn symTable_insert_enum(symtable: &mut SymTable, name: String, variants: List<Type>) -> bool {
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
    Cons(SymTableEntry, Box<GlobalSymTable>),
    Nil,
}

/// Prepend an entry to the global table.
fn globalSymTable_prepend(symtable: &mut GlobalSymTable, entry: SymTableEntry) {
    let old_copy: GlobalSymTable = globalSymTable_clone(symtable);
    let tail: Box<GlobalSymTable> = box_new::<GlobalSymTable>(old_copy);
    *symtable = GlobalSymTable::Cons(entry, tail);
}

/// Check whether a name exists in the global table.
fn globalSymTable_contains(symtable: &GlobalSymTable, name: &String) -> bool {
    match symtable {
        GlobalSymTable::Cons(head, tail) => {
            let entry_name: &String = symTableEntry_name(&head);
            or(
                string_eq(entry_name, name),
                globalSymTable_contains(box_deref::<GlobalSymTable>(tail), name),
            )
        }
        GlobalSymTable::Nil => false,
    }
}

/// Lookup a function signature in globals.
fn globalSymTable_lookup_function_signature(
    symtable: &GlobalSymTable,
    name: &String,
) -> Option<FnSignature> {
    match symtable {
        GlobalSymTable::Cons(entry, tail) => match entry {
            SymTableEntry::Function(entry_name, signature) => {
                if string_eq(entry_name, name) {
                    Option::Some(fnSignature_clone(signature))
                } else {
                    globalSymTable_lookup_function_signature(
                        box_deref::<GlobalSymTable>(tail),
                        name,
                    )
                }
            }
            _ => globalSymTable_lookup_function_signature(box_deref::<GlobalSymTable>(tail), name),
        },
        GlobalSymTable::Nil => Option::None,
    }
}

/// Insert a function entry into globals, returning false on duplicate name.
fn globalSymTable_insert_function(
    symtable: &mut GlobalSymTable,
    name: String,
    parameter_types: List<Type>,
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
    variants: List<Type>,
) -> bool {
    if globalSymTable_contains(symtable, &name) {
        return false;
    }

    let entry: SymTableEntry = SymTableEntry::Enum(name, variants);
    globalSymTable_prepend(symtable, entry);
    true
}

/// Stack of local scopes.
enum LocalSymTableStack {
    /// head, tail
    Cons(LocalSymTable, Box<LocalSymTableStack>),
    Nil,
}

/// Push a new empty local scope onto the stack.
fn localSymTableStack_push(stack: &mut LocalSymTableStack) {
    let old_copy: LocalSymTableStack = localSymTableStack_clone(stack);
    let tail: Box<LocalSymTableStack> = box_new::<LocalSymTableStack>(old_copy);
    *stack = LocalSymTableStack::Cons(LocalSymTable::Nil, tail);
}

/// Pop the top local scope from the stack.
fn localSymTableStack_pop(stack: &mut LocalSymTableStack) -> bool {
    match stack {
        LocalSymTableStack::Cons(_, tail) => {
            *stack = localSymTableStack_clone(box_deref::<LocalSymTableStack>(tail));
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
            localSymTableStack_contains(box_deref::<LocalSymTableStack>(tail), name),
        ),
        LocalSymTableStack::Nil => false,
    }
}

/// Lookup a variable type in any local scope.
fn localSymTableStack_lookup_variable_type(
    stack: &LocalSymTableStack,
    name: &String,
) -> Option<Type> {
    match stack {
        LocalSymTableStack::Cons(local, tail) => {
            match localSymTable_lookup_variable_type(local, name) {
                Option::Some(variable_type) => Option::Some(variable_type),
                Option::None => localSymTableStack_lookup_variable_type(
                    box_deref::<LocalSymTableStack>(tail),
                    name,
                ),
            }
        }
        LocalSymTableStack::Nil => Option::None,
    }
}

/// Single local scope represented as a linked cons list.
enum LocalSymTable {
    /// head, tail
    Cons(SymTableEntry, Box<LocalSymTable>),
    Nil,
}

/// Prepend an entry to a local scope.
fn localSymTable_prepend(symtable: &mut LocalSymTable, entry: SymTableEntry) {
    let old_copy: LocalSymTable = localSymTable_clone(symtable);
    let tail: Box<LocalSymTable> = box_new::<LocalSymTable>(old_copy);
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
                localSymTable_contains(box_deref::<LocalSymTable>(tail), name),
            )
        }
        LocalSymTable::Nil => false,
    }
}

/// Lookup a variable type in a single local scope.
fn localSymTable_lookup_variable_type(symtable: &LocalSymTable, name: &String) -> Option<Type> {
    match symtable {
        LocalSymTable::Cons(entry, tail) => match entry {
            SymTableEntry::Variable(entry_name, variable_type, _) => {
                if string_eq(entry_name, name) {
                    Option::Some(type_clone(variable_type))
                } else {
                    localSymTable_lookup_variable_type(box_deref::<LocalSymTable>(tail), name)
                }
            }
            _ => localSymTable_lookup_variable_type(box_deref::<LocalSymTable>(tail), name),
        },
        LocalSymTable::Nil => Option::None,
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
    Enum(String, List<Type>),
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
    Fn(List<Type>, Type),
}

/// Type forms supported by the front-end.
#[derive(Debug)]
enum Type {
    U8,
    Usize,
    Bool,
    Char,
    Unit,                       // ()
    Never,                      // !
    Custom(String),             // enums
    Reference(Box<Type>, bool), // &Type and &mut Type
    RawPointerMut(Box<Type>),   // *mut Type
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
        Type::Reference(_, _) => string_from_str("ptr"),
        Type::RawPointerMut(_) => string_from_str("ptr"),
    }
}

/// Returns true if the `from` type can be cast into the `to` type
fn type_can_cast_to(from: &Type, to: &Type) -> bool {
    match from {
        Type::U8 | Type::Usize => match to {
            Type::U8 | Type::Usize | Type::Char => true,
            _ => false,
        },
        Type::Bool => match to {
            Type::Bool | Type::U8 | Type::Usize => true,
            _ => false,
        },
        Type::Char => match to {
            Type::Char | Type::Usize | Type::U8 => true,
            _ => false,
        },
        Type::Reference(_, _) | Type::RawPointerMut(_) => match to {
            Type::RawPointerMut(_) => true,
            _ => false,
        },
        _ => false,
    }
}

/// Different operations that can be done when casting a value.
///
/// ZeroExtend: A type with smaller bitwidth is zero-extended to a larger bitwidth.
/// Truncate: A type with larger bitwidth is truncated to a smaller bitwidth.
/// None: Do not perform a cast (because the cast would be a no-op which is illegal in LLVM-IR).
/// Invalid: The cast is illegal.
enum CastOperation {
    /// A type with smaller bitwidth is zero-extended to a larger bitwidth.
    ZeroExtend,
    /// A type with larger bitwidth is truncated to a smaller bitwidth.
    Truncate,
    /// Do not perform a cast.
    None,
    /// The cast is illegal.
    Invalid,
}

/// Return the CastOperation that is applicable from `left_type` to `right_type`.
/// See documentation of CastOperation for more details.
fn type_get_cast_operation(left_type: &Type, right_type: &Type) -> CastOperation {
    if type_eq(left_type, right_type) {
        return CastOperation::None;
    }

    match left_type {
        Type::U8 => match right_type {
            Type::Usize | Type::Char => CastOperation::ZeroExtend,
            _ => CastOperation::Invalid,
        },
        Type::Usize => match right_type {
            Type::U8 => CastOperation::Truncate,
            _ => CastOperation::Invalid,
        },
        Type::Bool => match right_type {
            Type::U8 | Type::Usize => CastOperation::ZeroExtend,
            _ => CastOperation::Invalid,
        },
        Type::Char => match right_type {
            Type::Usize => CastOperation::ZeroExtend,
            Type::U8 => CastOperation::Truncate,
            _ => CastOperation::Invalid,
        },
        _ => CastOperation::Invalid,
    }
}

// -----------------------------------------------------------------
// ---------------------- Code Generation --------------------------
// -----------------------------------------------------------------

/// Emit a binary instruction of the following form:
/// `name` = `op` `ty` `lhs`,`rhs`
/// and return `name`.
///
/// The destination register's name `name` is the next available virtual register name that is retrieved
/// from the LLVM context.
fn llvm_emit_binary(
    parser: &mut Parser,
    op: &str,
    ty: &Type,
    lhs: &String,
    rhs: &String,
) -> String {
    let name: String = context_next_temporary(parser_context_mut(parser));
    let code: &mut String = parser_llvm_mut(parser);
    string_push_str(code, "  ");
    string_push_string(code, &name);
    string_push_str(code, " = ");
    string_push_str(code, op);
    string_push(code, ' ');
    string_push_string(code, &type_to_llvm_name(ty));
    string_push(code, ' ');
    string_push_string(code, lhs);
    string_push(code, ',');
    string_push_string(code, rhs);
    string_push(code, '\n');
    name
}

/// Emit an add instruction:
/// `name` = add `ty` `lhs`,`rhs`
/// and return `name`.
fn llvm_emit_add(parser: &mut Parser, ty: &Type, lhs: &String, rhs: &String) -> String {
    llvm_emit_binary(parser, "add", ty, lhs, rhs)
}

/// Emit an add instruction:
/// `name` = add `ty` `lhs`,`rhs`
/// and return `name`.
fn llvm_emit_sub(parser: &mut Parser, ty: &Type, lhs: &String, rhs: &String) -> String {
    llvm_emit_binary(parser, "sub", ty, lhs, rhs)
}

/// Emit a mul instruction:
/// `name` = mul `ty` `lhs`,`rhs`
/// and return `name`.
fn llvm_emit_mul(parser: &mut Parser, ty: &Type, lhs: &String, rhs: &String) -> String {
    llvm_emit_binary(parser, "mul", ty, lhs, rhs)
}

/// Emit a divu instruction:
/// `name` = divu `ty` `lhs`,`rhs`
/// and return `name`.
fn llvm_emit_udiv(parser: &mut Parser, ty: &Type, lhs: &String, rhs: &String) -> String {
    llvm_emit_binary(parser, "udiv", ty, lhs, rhs)
}

/// Emit a remu instruction:
/// `name` = remu `ty` `lhs`, `rhs`
/// and return `name`.
fn llvm_emit_urem(parser: &mut Parser, ty: &Type, lhs: &String, rhs: &String) -> String {
    llvm_emit_binary(parser, "urem", ty, lhs, rhs)
}

/// Emit an icmp instruction:
/// `name` = icmp `op` `ty` `lhs`,`rhs`
/// and return `name`.
fn llvm_emit_icmp(parser: &mut Parser, op: &str, ty: &Type, lhs: &String, rhs: &String) -> String {
    let name: String = context_next_temporary(parser_context_mut(parser));
    let code: &mut String = parser_llvm_mut(parser);
    string_push_str(code, "  ");
    string_push_string(code, &name);
    string_push_str(code, " = icmp ");
    string_push_str(code, op);
    string_push(code, ' ');
    string_push_string(code, &type_to_llvm_name(ty));
    string_push(code, ' ');
    string_push_string(code, lhs);
    string_push(code, ',');
    string_push_string(code, rhs);
    string_push(code, '\n');
    name
}

/// Emit a ret instruction:
/// ret `ty` `value`
fn llvm_emit_ret_value(parser: &mut Parser, ty: &Type, value: &String) {
    let code: &mut String = parser_llvm_mut(parser);
    string_push_str(code, "  ");
    string_push_str(code, "ret ");
    string_push_string(code, &type_to_llvm_name(ty));
    string_push(code, ' ');
    string_push_string(code, value);
    string_push(code, '\n');
}

/// Emit a ret void instruction:
/// ret void
fn llvm_emit_ret_void(parser: &mut Parser) {
    let code: &mut String = parser_llvm_mut(parser);
    string_push_str(code, "  ret void\n");
}

/// Emit a cast instruction of the following form:
/// `name` = `cast_op` `from_type` `value` to `to_type`
/// and return `name`.
///
/// The destination register's name `name` is the next available virtual register name that is retrieved
/// from the LLVM context.
fn llvm_emit_cast(
    parser: &mut Parser,
    cast_op: &str,
    from_type: &Type,
    to_type: &Type,
    value: &String,
) -> String {
    let name: String = context_next_temporary(parser_context_mut(parser));
    let code: &mut String = parser_llvm_mut(parser);
    string_push_str(code, "  ");
    string_push_string(code, &name);
    string_push_str(code, " = ");
    string_push_str(code, cast_op);
    string_push(code, ' ');
    string_push_string(code, &type_to_llvm_name(from_type));
    string_push(code, ' ');
    string_push_string(code, value);
    string_push_str(code, " to ");
    string_push_string(code, &type_to_llvm_name(to_type));
    string_push(code, '\n');
    name
}

/// Emit a zext instruction:
/// `name` = zext `from_type` `value` to `to_type`
/// and return `name`.
fn llvm_emit_zext(parser: &mut Parser, from_type: &Type, to_type: &Type, value: &String) -> String {
    llvm_emit_cast(parser, "zext", from_type, to_type, value)
}

/// Emit a trunc instruction:
/// `name` = trunc `from_type` `value` to `to_type`
/// and return `name`.
fn llvm_emit_trunc(
    parser: &mut Parser,
    from_type: &Type,
    to_type: &Type,
    value: &String,
) -> String {
    llvm_emit_cast(parser, "trunc", from_type, to_type, value)
}

/// Emit an alloca instruction:
/// `name` = alloca `ty`, `ty` `num_elements`
/// and return `name`.
fn llvm_emit_alloca(parser: &mut Parser, ty: &Type, num_elements: usize) -> String {
    let name: String = context_next_temporary(parser_context_mut(parser));
    let llvm_type: String = type_to_llvm_name(ty);
    let code: &mut String = parser_llvm_mut(parser);
    string_push_str(code, "  ");
    string_push_string(code, &name);
    string_push_str(code, " = alloca ");
    string_push_string(code, &llvm_type);
    string_push_str(code, ", i64 ");
    string_push_string(code, &integer_to_string(num_elements));
    string_push(code, '\n');
    name
}

/// Emit a store instruction:
/// store `ty` `value`, ptr `pointer`.
fn llvm_emit_store(parser: &mut Parser, ty: &Type, value: &String, pointer: &String) {
    let code: &mut String = parser_llvm_mut(parser);
    string_push_str(code, "  store ");
    string_push_string(code, &type_to_llvm_name(ty));
    string_push(code, ' ');
    string_push_string(code, value);
    string_push(code, ',');
    string_push_str(code, " ptr ");
    string_push_string(code, pointer);
    string_push(code, '\n');
}

/// Emit a load instruction:
/// `name` = load `ty`, `ptr` pointer`.
fn llvm_emit_load(parser: &mut Parser, ty: &Type, pointer: &String) -> String {
    let name: String = context_next_temporary(parser_context_mut(parser));
    let code: &mut String = parser_llvm_mut(parser);
    string_push_str(code, "  ");
    string_push_string(code, &name);
    string_push_str(code, " = load ");
    string_push_string(code, &type_to_llvm_name(ty));
    string_push(code, ',');
    string_push_str(code, " ptr ");
    string_push_string(code, pointer);
    string_push(code, '\n');
    name
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
    let source_file: SourceFile = SourceFile::SourceFile(source, 0, 1, 0);
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

/// Peek the current source character.
fn llvmLexer_peek_char(lexer: &LlvmLexer) -> Option<char> {
    let SourceFile::SourceFile(string, index, _, _): &SourceFile = llvmLexer_sourcefile(lexer);
    string_get(string, *index)
}

/// Peek the next source character after the current one and return true if it is the expected
/// character
fn llvmLexer_next_char_eq(lexer: &LlvmLexer, expected: char) -> bool {
    let SourceFile::SourceFile(content, index, _, _): &SourceFile = llvmLexer_sourcefile(lexer);
    match string_get(content, *index + 1) {
        Option::Some(character) => character == expected,
        _ => false,
    }
}

fn llvmLexer_expect_char(lexer: &mut LlvmLexer, expected: char) {
    match llvmLexer_consume_char(lexer) {
        Option::Some(c) => {
            if c != expected {
                panic!("unexpected character");
            }
        }
        _ => panic!("unexpected EOF"),
    }
}

/// Consume and return the current source character.
fn llvmLexer_consume_char(lexer: &mut LlvmLexer) -> Option<char> {
    let SourceFile::SourceFile(source, index, line, last_newline_idx): &mut SourceFile =
        llvmLexer_sourcefile_mut(lexer);

    let current: Option<char> = string_get(source, *index);
    *index = *index + 1;

    match current {
        Option::Some(character) => {
            if character == '\n' {
                *line = *line + 1;
                *last_newline_idx = *index;
            }
        }
        Option::None => {}
    }
    current
}

/// Consume and return the next token.
fn llvmLexer_next_token(lexer: &mut LlvmLexer) -> LlvmToken {
    llvmLexer_skip_whitespace_and_comments(lexer);

    let token: LlvmToken = match llvmLexer_peek_char(lexer) {
        Option::Some(ch) => {
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
        Option::None => LlvmToken::Eof,
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
            Option::Some('"') => return literal,
            Option::Some('\\') => {
                let character: char = llvmLexer_scan_escape(lexer);
                string_push(&mut literal, character);
            }
            Option::Some(ch) => string_push(&mut literal, ch),
            Option::None => panic!("unterminated LLVM c-string"),
        }
    }
    literal // satisfy compiler
}

fn llvmLexer_scan_escape(lexer: &mut LlvmLexer) -> char {
    match llvmLexer_consume_char(lexer) {
        Option::Some(hex_digit) => {
            if is_hexadecimal_digit(hex_digit) {
                match llvmLexer_consume_char(lexer) {
                    Option::Some(second_hex_digit) => {
                        let mut char_byte: String = string_new();
                        string_push(&mut char_byte, hex_digit);
                        string_push(&mut char_byte, second_hex_digit);

                        unwrap::<usize>(string_to_integer(&mut char_byte, 16)) as u8 as char
                    }
                    _ => panic!("expected second digit for escaped character byte"),
                }
            } else {
                hex_digit
            }
        }
        Option::None => panic!("unterminated LLVM c-string"),
    }
}

fn llvmLexer_scan_identifier_or_keyword(lexer: &mut LlvmLexer) -> String {
    let mut identifier: String = string_new();
    while true {
        match llvmLexer_peek_char(lexer) {
            Option::Some(ch) => {
                if is_alphanumeric_or_dot(ch) {
                    llvmLexer_consume_char(lexer);
                    string_push(&mut identifier, ch);
                } else {
                    return identifier;
                }
            }
            Option::None => return identifier,
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
            Option::Some(ch) => {
                if is_digit(ch) {
                    let digit: usize = (ch as usize) - ('0' as usize);
                    value = value * 10 + digit;
                    llvmLexer_consume_char(lexer);
                } else {
                    return value;
                }
            }
            Option::None => return value,
        }
    }
    value
}

fn llvmLexer_scan_symbol(lexer: &mut LlvmLexer) -> LlvmToken {
    match unwrap::<char>(llvmLexer_consume_char(lexer)) {
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
            Option::Some(ch) => {
                if is_whitespace(ch) {
                    llvmLexer_consume_char(lexer);
                } else if ch == ';' {
                    llvmLexer_consume_char(lexer);
                    llvmLexer_skip_line(lexer);
                } else {
                    return;
                }
            }
            Option::None => return,
        }
    }
}

fn llvmLexer_skip_line(lexer: &mut LlvmLexer) {
    while true {
        match llvmLexer_consume_char(lexer) {
            Option::Some('\n') => return,
            Option::Some(_) => (),
            Option::None => return,
        }
    }
}

// -----------------------------------------------------------------
// ------------------------- Parser --------------------------------
// -----------------------------------------------------------------

/// Type that encapsulates the LLVM Parser's state
enum LlvmParser {
    Parser(LlvmLexer, LlvmAST, LlvmLocalSymTable),
}

/// Create an LLVM parser and prime the first token.
fn llvmParser_new(source: String) -> LlvmParser {
    LlvmParser::Parser(
        llvmLexer_new(source),
        llvmAST_new(),
        llvmLocalSymTable_new(),
    )
}

/// Create an LLVM parser from a string slice.
fn llvmParser_from_str(source: &str) -> LlvmParser {
    llvmParser_new(string_from_str(source))
}

/// Get immutable parser lexer access.
fn llvmParser_lexer(parser: &LlvmParser) -> &LlvmLexer {
    let LlvmParser::Parser(lexer, _, _): &LlvmParser = parser;
    lexer
}

/// Get mutable parser lexer access.
fn llvmParser_lexer_mut(parser: &mut LlvmParser) -> &mut LlvmLexer {
    let LlvmParser::Parser(lexer, _, _): &mut LlvmParser = parser;
    lexer
}

/// Get immutable parser AST access.
fn llvmParser_ast(parser: &LlvmParser) -> &LlvmAST {
    let LlvmParser::Parser(_, ast, _): &LlvmParser = parser;
    ast
}

/// Get mutable parser AST access.
fn llvmParser_ast_mut(parser: &mut LlvmParser) -> &mut LlvmAST {
    let LlvmParser::Parser(_, ast, _): &mut LlvmParser = parser;
    ast
}

fn llvmParser_local(parser: &LlvmParser) -> &LlvmLocalSymTable {
    let LlvmParser::Parser(_, _, local): &LlvmParser = parser;
    local
}

fn llvmParser_local_mut(parser: &mut LlvmParser) -> &mut LlvmLocalSymTable {
    let LlvmParser::Parser(_, _, local): &mut LlvmParser = parser;
    local
}

/// Parse LLVM source into LLVM AST.
fn llvmParser_parse_to_ast(source: String) -> LlvmAST {
    let mut parser: LlvmParser = llvmParser_new(source);
    llvmParser_parse_language(&mut parser);
    let LlvmParser::Parser(_, ast, _): LlvmParser = parser;
    ast
}

/// Get current LLVM parser token.
fn llvmParser_current_token(parser: &LlvmParser) -> &LlvmToken {
    llvmLexer_current_token(llvmParser_lexer(parser))
}

/// Advance and return next LLVM parser token.
fn llvmParser_next_token(parser: &mut LlvmParser) -> LlvmToken {
    llvmLexer_next_token(llvmParser_lexer_mut(parser))
}

/// Check whether parser current token equals expected token.
fn llvmParser_current_token_eq(parser: &LlvmParser, token: &LlvmToken) -> bool {
    llvmToken_eq(llvmParser_current_token(parser), token)
}

/// Try consuming one token and report success.
fn llvmParser_try_consume(parser: &mut LlvmParser, token: &LlvmToken) -> bool {
    if llvmParser_current_token_eq(parser, token) {
        llvmParser_next_token(parser);
        true
    } else {
        false
    }
}

/// Require and consume one token.
fn llvmParser_expect_token(parser: &mut LlvmParser, token: &LlvmToken) {
    if not(llvmParser_try_consume(parser, token)) {
        llvmParser_error(parser, "unexpected LLVM token");
    }
}

/// Read and consume one identifier token.
fn llvmParser_expect_identifier(parser: &mut LlvmParser) -> String {
    match llvmParser_current_token(parser) {
        LlvmToken::Identifier(identifier) => {
            let value: String = string_clone(identifier);
            llvmParser_next_token(parser);
            value
        }
        _ => llvmParser_error(parser, "expected LLVM identifier"),
    }
}

enum LlvmAST {
    AST(StringMap<LlvmFunction>),
}

/// Create an empty LLVM AST.
fn llvmAST_new() -> LlvmAST {
    LlvmAST::AST(stringMap_new::<LlvmFunction>())
}

/// Get immutable access to the top-level function map.
fn llvmAST_functions(ast: &LlvmAST) -> &StringMap<LlvmFunction> {
    let LlvmAST::AST(functions): &LlvmAST = ast;
    functions
}

/// Get mutable access to the top-level function map.
fn llvmAST_functions_mut(ast: &mut LlvmAST) -> &mut StringMap<LlvmFunction> {
    let LlvmAST::AST(functions): &mut LlvmAST = ast;
    functions
}

/// Insert a function entry into the AST. Returns false on duplicate name.
fn llvmAST_insert_function(ast: &mut LlvmAST, name: String, function: LlvmFunction) -> bool {
    match stringMap_get::<LlvmFunction>(llvmAST_functions(ast), &name) {
        Option::Some(_) => false,
        Option::None => {
            stringMap_insert::<LlvmFunction>(llvmAST_functions_mut(ast), name, function);
            true
        }
    }
}

/// Lookup a function in the AST by name.
fn llvmAST_lookup_function(ast: &LlvmAST, name: String) -> &LlvmFunction {
    match stringMap_get::<LlvmFunction>(llvmAST_functions(ast), &name) {
        Option::Some(function) => function,
        Option::None => panic!("unknown LLVM function"),
    }
}

/// Local symbol table for LLVM
enum LlvmLocalSymTable {
    Registers(List<LlvmLocalSymTableEntry>),
}

/// Create an empty LLVM local symbol table.
fn llvmLocalSymTable_new() -> LlvmLocalSymTable {
    LlvmLocalSymTable::Registers(list_new::<LlvmLocalSymTableEntry>())
}

/// Clear local register table buckets.
fn llvmLocalSymTable_clear(symtable: &mut LlvmLocalSymTable) {
    match symtable {
        LlvmLocalSymTable::Registers(registers) => {
            *registers = list_new::<LlvmLocalSymTableEntry>()
        }
    }
}

/// Insert register value. Returns false on duplicate.
/// TODO: should pass type instead of value
fn llvmLocalSymTable_insert_register(
    symtable: &mut LlvmLocalSymTable,
    name: String,
    value: usize,
) -> bool {
    match symtable {
        LlvmLocalSymTable::Registers(registers) => {
            let mut cursor: &List<LlvmLocalSymTableEntry> = registers;
            while true {
                match cursor {
                    List::Nil => {
                        list_append::<LlvmLocalSymTableEntry>(
                            registers,
                            LlvmLocalSymTableEntry::Register(name, value),
                        );
                        return true;
                    }
                    List::Cons(entry, tail) => {
                        let LlvmLocalSymTableEntry::Register(register_name, _) = entry;
                        if string_eq(register_name, &name) {
                            return false;
                        }
                        cursor = box_deref::<List<LlvmLocalSymTableEntry>>(tail);
                    }
                }
            }
            false
        }
    }
}

/// Virtual register entry of a LlvmLocalSymTable
enum LlvmLocalSymTableEntry {
    /// identifier, value
    Register(String, usize),
}

enum LlvmFunction {
    /// return type, parameters, instructions, local symbols
    Function(LlvmType, Vec<LlvmParameter>, Vec<InstructionBlock>),
}

/// Represents a parameter of an LLVM function.
enum LlvmParameter {
    /// identifier, type
    Parameter(String, LlvmType),
}

/// Supported types of LLVM.
enum LlvmType {
    I1,
    I8,
    I32,
    I64,
    Ptr,
    Array(usize, Box<LlvmType>),
    Void,
}

/// Represents an instruction block.
enum InstructionBlock {
    /// label, instructions
    Block(String, Vec<Instruction>),
}

/// Get a shared reference to the label of an instruction block.
fn instructionBlock_label(instruction_block: &InstructionBlock) -> &String {
    let InstructionBlock::Block(label, _): &InstructionBlock = instruction_block;
    label
}

/// Get a shared reference to the instructions of an instruction block.
fn instructionBlock_instructions(instruction_block: &InstructionBlock) -> &Vec<Instruction> {
    let InstructionBlock::Block(_, instructions): &InstructionBlock = instruction_block;
    instructions
}

/// Get the instructions of the block labelled by the given label.
fn instructionBlock_fetch_instructions(
    blocks: &Vec<InstructionBlock>,
    label: String,
) -> &Vec<Instruction> {
    let mut i: usize = 0;
    while i < vec_len::<InstructionBlock>(blocks) {
        let block: &InstructionBlock =
            unwrap::<&InstructionBlock>(vec_get::<InstructionBlock>(blocks, i));

        let other_label: &String = instructionBlock_label(block);
        if string_eq(other_label, &label) {
            return instructionBlock_instructions(block);
        }

        i = i + 1;
    }
    panic!("unknown LLVM block label");
}

/// Represents an instruction inside an instruction block
enum Instruction {
    Assignment(AssignInstruction),
    Terminator(TerminatorInstruction),
}

/// Represents an instruction which terminates an instruction block.
enum TerminatorInstruction {
    RetVoid,
    /// type, value
    Ret(LlvmType, LlvmValue),
    Br(Branch),
}

/// Represents "br", either a conditional or unconditional jump.
enum Branch {
    /// label
    Unconditional(String),
    /// condition, then label, else label
    Conditional(LlvmValue, String, String),
}

/// Represents an assignment instruction.
enum AssignInstruction {
    Assign(String, AssignOp),
}

/// Represents the right-hand-side of an assignment
enum AssignOp {
    /// operation, type, left operand, right operand
    Binary(BinaryOp, LlvmType, LlvmValue, LlvmValue),
    /// return type, callee, arguments
    Call(LlvmType, String, Vec<LlvmTypedValue>),
    /// type, pointer, indexes
    Gep(LlvmType, LlvmValue, Vec<LlvmTypedValue>),
}

/// Binary operations that can only appear in assignments.
enum BinaryOp {
    Add,
    Sub,
    Mul,
    Udiv,
    Urem,
    IcmpUlt,
}

/// Represents a value in a register, global or as a literal.
enum LlvmValue {
    /// identifier
    Register(String),
    /// integer value
    Literal(usize),
    /// identifier
    Global(String),
}

/// Represents a value with a specified type.
// TODO: drop this: the AST does not need to know about types. Parser ensures type safety.
enum LlvmTypedValue {
    Pair(LlvmType, LlvmValue),
}

fn llvmParser_parse_language(parser: &mut LlvmParser) {
    while not(llvmParser_current_token_eq(parser, &LlvmToken::Eof)) {
        match llvmParser_current_token(parser) {
            LlvmToken::At => llvmParser_parse_string(parser),
            LlvmToken::Define => llvmParser_parse_function(parser),
            _ => llvmParser_error(parser, "expected LLVM top-level item"),
        }
    }
}

fn llvmParser_parse_string(parser: &mut LlvmParser) {
    let name: String = llvmParser_parse_global_name(parser);
    llvmParser_expect_token(parser, &LlvmToken::Assign);
    llvmParser_expect_token(parser, &LlvmToken::Constant);
    llvmParser_parse_type(parser);

    match llvmParser_current_token(parser) {
        LlvmToken::CString(value) => {
            let string_value: String = string_clone(value);
            llvmParser_next_token(parser);
        }
        _ => llvmParser_error(parser, "expected LLVM c-string literal"),
    }
}

fn llvmParser_parse_function(parser: &mut LlvmParser) {
    llvmParser_expect_token(parser, &LlvmToken::Define);
    let return_type: LlvmType = llvmParser_parse_type(parser);
    let function_name: String = llvmParser_parse_global_name(parser);

    llvmLocalSymTable_clear(llvmParser_local_mut(parser));

    let parameters: Vec<LlvmParameter> = llvmParser_parse_parameters(parser);

    llvmParser_expect_token(parser, &LlvmToken::LBrace);
    let blocks: Vec<InstructionBlock> = llvmParser_parse_blocks(parser);
    llvmParser_expect_token(parser, &LlvmToken::RBrace);

    let function: LlvmFunction = LlvmFunction::Function(return_type, parameters, blocks);
    if not(llvmAST_insert_function(
        llvmParser_ast_mut(parser),
        function_name,
        function,
    )) {
        llvmParser_error(parser, "duplicate LLVM function definition");
    }
}

fn llvmParser_parse_parameters(parser: &mut LlvmParser) -> Vec<LlvmParameter> {
    let mut parameters: Vec<LlvmParameter> = vec_new::<LlvmParameter>();

    llvmParser_expect_token(parser, &LlvmToken::LParen);

    if not(llvmParser_current_token_eq(parser, &LlvmToken::RParen)) {
        let parameter_type: LlvmType = llvmParser_parse_type(parser);
        let param_name: String = llvmParser_parse_register(parser);
        llvmLocalSymTable_insert_register(
            llvmParser_local_mut(parser),
            string_clone(&param_name),
            0,
        );

        let parameter: LlvmParameter = LlvmParameter::Parameter(param_name, parameter_type);
        vec_push::<LlvmParameter>(&mut parameters, parameter);

        while llvmParser_current_token_eq(parser, &LlvmToken::Comma) {
            llvmParser_next_token(parser);

            let parameter_type: LlvmType = llvmParser_parse_type(parser);
            let param_name: String = llvmParser_parse_register(parser);

            if not(llvmLocalSymTable_insert_register(
                llvmParser_local_mut(parser),
                string_clone(&param_name),
                0,
            )) {
                llvmParser_error(parser, "duplicate parameters in LLVM function");
            }

            let parameter: LlvmParameter = LlvmParameter::Parameter(param_name, parameter_type);
            vec_push::<LlvmParameter>(&mut parameters, parameter);
        }
    }
    llvmParser_expect_token(parser, &LlvmToken::RParen);
    parameters
}

fn llvmParser_parse_global_name(parser: &mut LlvmParser) -> String {
    llvmParser_expect_token(parser, &LlvmToken::At);
    llvmParser_expect_identifier(parser)
}

fn llvmParser_parse_blocks(parser: &mut LlvmParser) -> Vec<InstructionBlock> {
    let mut blocks: Vec<InstructionBlock> = vec_new::<InstructionBlock>();
    while not(llvmParser_current_token_eq(parser, &LlvmToken::RBrace)) {
        let block: InstructionBlock = llvmParser_parse_block(parser);
        vec_push::<InstructionBlock>(&mut blocks, block);
    }
    blocks
}

fn llvmParser_parse_block(parser: &mut LlvmParser) -> InstructionBlock {
    let label: String = llvmParser_expect_identifier(parser);
    llvmParser_expect_token(parser, &LlvmToken::Colon);
    // TODO: insert into symbol table

    let mut instructions: Vec<Instruction> = vec_new::<Instruction>();
    let mut is_terminator: bool = false;

    while not(is_terminator) {
        let instruction: Instruction = llvmParser_parse_instruction(parser);
        match &instruction {
            Instruction::Terminator(_) => is_terminator = true,
            Instruction::Assignment(_) => is_terminator = false,
        }
        vec_push::<Instruction>(&mut instructions, instruction);
    }

    InstructionBlock::Block(label, instructions)
}

fn llvmParser_parse_register(parser: &mut LlvmParser) -> String {
    llvmParser_expect_token(parser, &LlvmToken::Percent);
    llvmParser_expect_identifier(parser)
}

fn llvmParser_parse_instruction(parser: &mut LlvmParser) -> Instruction {
    match llvmParser_current_token(parser) {
        LlvmToken::Ret => Instruction::Terminator(llvmParser_parse_return(parser)),
        LlvmToken::Br => Instruction::Terminator(llvmParser_parse_branch(parser)),
        LlvmToken::Percent => Instruction::Assignment(llvmParser_parse_assignment(parser)),
        _ => llvmParser_error(parser, "expected LLVM instruction"),
    }
}

fn llvmParser_parse_return(parser: &mut LlvmParser) -> TerminatorInstruction {
    llvmParser_expect_token(parser, &LlvmToken::Ret);
    if llvmParser_try_consume(parser, &LlvmToken::Void) {
        TerminatorInstruction::RetVoid
    } else {
        let returned_type: LlvmType = llvmParser_parse_type(parser);
        let returned_value: LlvmValue = llvmParser_parse_value(parser);
        TerminatorInstruction::Ret(returned_type, returned_value)
    }
}

fn llvmParser_parse_branch(parser: &mut LlvmParser) -> TerminatorInstruction {
    llvmParser_expect_token(parser, &LlvmToken::Br);
    let branch: Branch = if llvmParser_try_consume(parser, &LlvmToken::Label) {
        let target_label: String = llvmParser_parse_register(parser);
        Branch::Unconditional(target_label)
    } else {
        llvmParser_expect_token(parser, &LlvmToken::I1);
        let condition: LlvmValue = llvmParser_parse_value(parser);
        llvmParser_expect_token(parser, &LlvmToken::Comma);

        llvmParser_expect_token(parser, &LlvmToken::Label);
        let then_label: String = llvmParser_parse_register(parser);
        llvmParser_expect_token(parser, &LlvmToken::Comma);

        llvmParser_expect_token(parser, &LlvmToken::Label);
        let else_label: String = llvmParser_parse_register(parser);

        Branch::Conditional(condition, then_label, else_label)
    };
    TerminatorInstruction::Br(branch)
}

fn llvmParser_parse_assignment(parser: &mut LlvmParser) -> AssignInstruction {
    let target_register: String = llvmParser_parse_register(parser);
    if not(llvmLocalSymTable_insert_register(
        llvmParser_local_mut(parser),
        string_clone(&target_register),
        0,
    )) {
        llvmParser_error(
            parser,
            "SSA violation: duplicate virtual register assignment",
        );
    }

    llvmParser_expect_token(parser, &LlvmToken::Assign);
    let operation: AssignOp = match llvmParser_current_token(parser) {
        LlvmToken::Add => llvmParser_parse_binary_assign(parser, LlvmToken::Add, BinaryOp::Add),
        LlvmToken::Sub => llvmParser_parse_binary_assign(parser, LlvmToken::Sub, BinaryOp::Sub),
        LlvmToken::Mul => llvmParser_parse_binary_assign(parser, LlvmToken::Mul, BinaryOp::Mul),
        LlvmToken::Udiv => llvmParser_parse_binary_assign(parser, LlvmToken::Udiv, BinaryOp::Udiv),
        LlvmToken::Urem => llvmParser_parse_binary_assign(parser, LlvmToken::Urem, BinaryOp::Urem),
        LlvmToken::Icmp => llvmParser_parse_icmp_assign(parser),
        LlvmToken::Call => llvmParser_parse_call_assign(parser),
        LlvmToken::Gep => llvmParser_parse_gep_assign(parser),
        _ => llvmParser_error(parser, "expected LLVM assignment operation"),
    };
    AssignInstruction::Assign(target_register, operation)
}

fn llvmParser_parse_binary_assign(
    parser: &mut LlvmParser,
    operator_token: LlvmToken,
    operator: BinaryOp,
) -> AssignOp {
    llvmParser_expect_token(parser, &operator_token);
    let ty: LlvmType = llvmParser_parse_type(parser);
    let left: LlvmValue = llvmParser_parse_value(parser);
    llvmParser_expect_token(parser, &LlvmToken::Comma);
    let right: LlvmValue = llvmParser_parse_value(parser);
    AssignOp::Binary(operator, ty, left, right)
}

fn llvmParser_parse_icmp_assign(parser: &mut LlvmParser) -> AssignOp {
    llvmParser_expect_token(parser, &LlvmToken::Icmp);
    llvmParser_expect_token(parser, &LlvmToken::Ult);
    let ty: LlvmType = llvmParser_parse_type(parser);
    let left: LlvmValue = llvmParser_parse_value(parser);
    llvmParser_expect_token(parser, &LlvmToken::Comma);
    let right: LlvmValue = llvmParser_parse_value(parser);
    AssignOp::Binary(BinaryOp::IcmpUlt, ty, left, right)
}

fn llvmParser_parse_call_assign(parser: &mut LlvmParser) -> AssignOp {
    llvmParser_expect_token(parser, &LlvmToken::Call);
    let return_type: LlvmType = llvmParser_parse_type(parser);
    let callee: String = llvmParser_parse_global_name(parser);
    let mut arguments: Vec<LlvmTypedValue> = vec_new::<LlvmTypedValue>();

    llvmParser_expect_token(parser, &LlvmToken::LParen);
    if not(llvmParser_current_token_eq(parser, &LlvmToken::RParen)) {
        let argument: LlvmTypedValue = llvmParser_parse_typed_value(parser);
        vec_push::<LlvmTypedValue>(&mut arguments, argument);

        while llvmParser_current_token_eq(parser, &LlvmToken::Comma) {
            llvmParser_next_token(parser);
            let argument: LlvmTypedValue = llvmParser_parse_typed_value(parser);
            vec_push::<LlvmTypedValue>(&mut arguments, argument);
        }
    }
    llvmParser_expect_token(parser, &LlvmToken::RParen);

    AssignOp::Call(return_type, callee, arguments)
}

fn llvmParser_parse_gep_assign(parser: &mut LlvmParser) -> AssignOp {
    llvmParser_expect_token(parser, &LlvmToken::Gep);
    let base_type: LlvmType = llvmParser_parse_type(parser);
    llvmParser_expect_token(parser, &LlvmToken::Comma);
    llvmParser_expect_token(parser, &LlvmToken::Ptr);
    let pointer_value: LlvmValue = llvmParser_parse_value(parser);
    llvmParser_expect_token(parser, &LlvmToken::Comma);

    let mut indexes: Vec<LlvmTypedValue> = vec_new::<LlvmTypedValue>();
    let first_index: LlvmTypedValue = llvmParser_parse_typed_value(parser);
    vec_push::<LlvmTypedValue>(&mut indexes, first_index);
    while llvmParser_try_consume(parser, &LlvmToken::Comma) {
        let index: LlvmTypedValue = llvmParser_parse_typed_value(parser);
        vec_push::<LlvmTypedValue>(&mut indexes, index);
    }

    AssignOp::Gep(base_type, pointer_value, indexes)
}

fn llvmParser_parse_type(parser: &mut LlvmParser) -> LlvmType {
    match llvmParser_current_token(parser) {
        LlvmToken::I1 => {
            llvmParser_next_token(parser);
            LlvmType::I1
        }
        LlvmToken::I8 => {
            llvmParser_next_token(parser);
            LlvmType::I8
        }
        LlvmToken::I32 => {
            llvmParser_next_token(parser);
            LlvmType::I32
        }
        LlvmToken::I64 => {
            llvmParser_next_token(parser);
            LlvmType::I64
        }
        LlvmToken::Void => {
            llvmParser_next_token(parser);
            LlvmType::Void
        }
        LlvmToken::Ptr => {
            llvmParser_next_token(parser);
            LlvmType::Ptr
        }
        LlvmToken::LBracket => {
            llvmParser_next_token(parser);
            let len: usize = llvmParser_parse_non_negative_number(parser);
            match llvmParser_current_token(parser) {
                LlvmToken::Identifier(separator) => {
                    if not(string_eq(separator, &string_from_str("x"))) {
                        llvmParser_error(parser, "expected x in LLVM array type");
                    }
                    llvmParser_next_token(parser);
                }
                _ => llvmParser_error(parser, "expected x in LLVM array type"),
            }
            let inner: LlvmType = llvmParser_parse_type(parser);
            llvmParser_expect_token(parser, &LlvmToken::RBracket);
            LlvmType::Array(len, box_new::<LlvmType>(inner))
        }
        _ => llvmParser_error(parser, "expected LLVM type"),
    }
}

fn llvmParser_parse_value(parser: &mut LlvmParser) -> LlvmValue {
    match llvmParser_current_token(parser) {
        LlvmToken::Percent => LlvmValue::Register(llvmParser_parse_register(parser)),
        LlvmToken::At => LlvmValue::Global(llvmParser_parse_global_name(parser)),
        LlvmToken::Minus => LlvmValue::Literal(llvmParser_parse_number_literal(parser)),
        LlvmToken::Integer(_) => LlvmValue::Literal(llvmParser_parse_number_literal(parser)),
        _ => llvmParser_error(parser, "expected LLVM value"),
    }
}

fn llvmParser_parse_typed_value(parser: &mut LlvmParser) -> LlvmTypedValue {
    let ty: LlvmType = llvmParser_parse_type(parser);
    let value: LlvmValue = llvmParser_parse_value(parser);
    LlvmTypedValue::Pair(ty, value)
}

fn llvmParser_parse_non_negative_number(parser: &mut LlvmParser) -> usize {
    match llvmParser_current_token(parser) {
        LlvmToken::Integer(value) => {
            let result: usize = *value;
            llvmParser_next_token(parser);
            result
        }
        _ => llvmParser_error(parser, "expected LLVM number"),
    }
}

// TODO: do not support negative literals
fn llvmParser_parse_number_literal(parser: &mut LlvmParser) -> usize {
    let negative: bool = llvmParser_try_consume(parser, &LlvmToken::Minus);
    match llvmParser_current_token(parser) {
        LlvmToken::Integer(value) => {
            let magnitude: usize = *value;
            llvmParser_next_token(parser);
            if negative {
                0usize.wrapping_sub(magnitude)
            } else {
                magnitude
            }
        }
        _ => llvmParser_error(parser, "expected LLVM integer literal"),
    }
}

// ------------------------- Interpreter -----------------------------

/// Execution control flow after one instruction.
enum LlvmExecFlow {
    Continue,
    /// label
    Jump(String),
    /// return value
    Return(usize),
}

/// Type that encapsulates the state of the LLVM emulator.
enum Llvmulator {
    /// map of global values
    Llvmulator(StringMap<usize>),
}

/// Create a new emulator state around one AST.
fn llvmulator_new() -> Llvmulator {
    Llvmulator::Llvmulator(stringMap_new::<usize>())
}

/// Get a shared reference to the global values.
fn llvmulator_globals(emulator: &Llvmulator) -> &StringMap<usize> {
    let Llvmulator::Llvmulator(globals): &Llvmulator = emulator;
    globals
}

/// Parse and emulate LLVM source and return the return value of @main.
fn llvmulator_execute_llvm(source: String) -> usize {
    let mut emulator: Llvmulator = llvmulator_new();
    let ast: LlvmAST = llvmParser_parse_to_ast(source);
    let main_name: String = string_from_str("main");
    let empty_args: Vec<LlvmTypedValue> = vec_new::<LlvmTypedValue>();
    llvmulator_execute_function_named(&mut emulator, &ast, &main_name, &empty_args)
}

/// Lookup a function by name and execute it.
fn llvmulator_execute_function_named(
    emulator: &mut Llvmulator,
    ast: &LlvmAST,
    function_name: &String,
    arguments: &Vec<LlvmTypedValue>,
) -> usize {
    let function: &LlvmFunction = llvmAST_lookup_function(ast, string_clone(function_name));
    llvmulator_execute_function(emulator, ast, function, arguments)
}

/// Execute the given function's body.
fn llvmulator_execute_function(
    emulator: &mut Llvmulator,
    ast: &LlvmAST,
    function: &LlvmFunction,
    arguments: &Vec<LlvmTypedValue>,
) -> usize {
    let LlvmFunction::Function(_, parameters, blocks): &LlvmFunction = function;
    if vec_len::<LlvmParameter>(parameters) != vec_len::<LlvmTypedValue>(arguments) {
        panic!("LLVM call argument count mismatch");
    }

    let mut local_register_values: StringMap<usize> = stringMap_new::<usize>();

    let mut i: usize = 0;
    while i < vec_len::<LlvmParameter>(parameters) {
        let parameter: &LlvmParameter =
            unwrap::<&LlvmParameter>(vec_get::<LlvmParameter>(parameters, i));
        let argument: &LlvmTypedValue =
            unwrap::<&LlvmTypedValue>(vec_get::<LlvmTypedValue>(arguments, i));

        let LlvmParameter::Parameter(name, _): &LlvmParameter = parameter;
        let LlvmTypedValue::Pair(_, argument_value): &LlvmTypedValue = argument;

        let value: usize = llvm_eval_value(
            llvmulator_globals(emulator),
            &local_register_values,
            argument_value,
        );
        stringMap_insert::<usize>(&mut local_register_values, string_clone(name), value);

        i = i + 1;
    }

    let mut current_label: String = match vec_get::<InstructionBlock>(blocks, 0) {
        Option::Some(label) => string_clone(instructionBlock_label(label)),
        _ => panic!("empty function body!"),
    };
    while true {
        let instructions: &Vec<Instruction> =
            instructionBlock_fetch_instructions(blocks, string_clone(&current_label));

        let flow: LlvmExecFlow = llvmulator_execute_instructions(
            emulator,
            ast,
            &mut local_register_values,
            instructions,
        );

        match flow {
            LlvmExecFlow::Continue => panic!("LLVM block did not terminate"),
            LlvmExecFlow::Jump(next_label) => current_label = next_label,
            LlvmExecFlow::Return(value) => return value,
        }
    }
    0
}

/// Execute a given list of instructions.
fn llvmulator_execute_instructions(
    emulator: &mut Llvmulator,
    ast: &LlvmAST,
    register_values: &mut StringMap<usize>,
    instructions: &Vec<Instruction>,
) -> LlvmExecFlow {
    let mut i: usize = 0;
    while i < vec_len::<Instruction>(instructions) {
        let instruction: &Instruction =
            unwrap::<&Instruction>(vec_get::<Instruction>(instructions, i));

        match instruction {
            Instruction::Assignment(assign_instruction) => {
                llvmulator_execute_assignment(emulator, ast, register_values, assign_instruction);
            }
            Instruction::Terminator(terminator) => {
                return llvmulator_execute_terminator(
                    llvmulator_globals(emulator),
                    register_values,
                    terminator,
                );
            }
        }

        i = i + 1;
    }
    LlvmExecFlow::Continue
}

/// Execute the given assignment instruction.
fn llvmulator_execute_assignment(
    emulator: &mut Llvmulator,
    ast: &LlvmAST,
    register_values: &mut StringMap<usize>,
    instruction: &AssignInstruction,
) {
    let AssignInstruction::Assign(target, operation): &AssignInstruction = instruction;
    let value: usize = llvmulator_evaluate_assign_op(emulator, ast, register_values, operation);
    stringMap_insert(register_values, string_clone(target), value);
}

/// Evaluate the value of the assignment operation.
fn llvmulator_evaluate_assign_op(
    emulator: &mut Llvmulator,
    ast: &LlvmAST,
    register_values: &mut StringMap<usize>,
    operation: &AssignOp,
) -> usize {
    let global_values: &StringMap<usize> = llvmulator_globals(emulator);
    match operation {
        AssignOp::Binary(operator, _, left, right) => {
            let lhs: usize = llvm_eval_value(global_values, register_values, left);
            let rhs: usize = llvm_eval_value(llvmulator_globals(emulator), register_values, right);
            match operator {
                BinaryOp::Add => lhs + rhs,
                BinaryOp::Sub => lhs - rhs,
                BinaryOp::Mul => lhs * rhs,
                BinaryOp::Udiv => lhs / rhs,
                BinaryOp::Urem => lhs % rhs,
                BinaryOp::IcmpUlt => (lhs < rhs) as usize,
            }
        }
        AssignOp::Call(_, callee, arguments) => {
            llvmulator_execute_function_named(emulator, ast, callee, arguments)
        }
        AssignOp::Gep(_, pointer, indexes) => {
            let mut address: usize = llvm_eval_value(global_values, register_values, pointer);
            let mut i: usize = 0;
            while i < vec_len::<LlvmTypedValue>(indexes) {
                let typed_value: &LlvmTypedValue =
                    unwrap::<&LlvmTypedValue>(vec_get::<LlvmTypedValue>(indexes, i));
                let LlvmTypedValue::Pair(_, index_value): &LlvmTypedValue = typed_value;
                address = address + llvm_eval_value(global_values, register_values, index_value);
                i = i + 1;
            }
            address
        }
    }
}

/// Execute the given terminator instruction.
fn llvmulator_execute_terminator(
    global_values: &StringMap<usize>,
    register_values: &StringMap<usize>,
    terminator: &TerminatorInstruction,
) -> LlvmExecFlow {
    match terminator {
        TerminatorInstruction::RetVoid => LlvmExecFlow::Return(0),
        TerminatorInstruction::Ret(_, value) => {
            LlvmExecFlow::Return(llvm_eval_value(global_values, register_values, value))
        }
        TerminatorInstruction::Br(branch) => match branch {
            Branch::Unconditional(target_label) => LlvmExecFlow::Jump(string_clone(target_label)),
            Branch::Conditional(condition, then_label, else_label) => {
                let condition_value: usize =
                    llvm_eval_value(global_values, register_values, condition);
                if condition_value == 1 {
                    LlvmExecFlow::Jump(string_clone(then_label))
                } else {
                    LlvmExecFlow::Jump(string_clone(else_label))
                }
            }
        },
    }
}

/// Evaluate the value of a virtual register, global name or literal.
fn llvm_eval_value(
    global_values: &StringMap<usize>,
    register_values: &StringMap<usize>,
    value: &LlvmValue,
) -> usize {
    match value {
        LlvmValue::Literal(number) => *number,
        LlvmValue::Register(name) => match stringMap_get(register_values, name) {
            Option::Some(register_value) => *register_value,
            Option::None => panic!("unknown LLVM register"),
        },
        LlvmValue::Global(name) => match stringMap_get::<usize>(global_values, name) {
            Option::Some(value) => *value,
            Option::None => panic!("unknown LLVM global value"),
        },
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
    llvm_emit_line(llvm, "entry:");
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

// -------------------------- Error --------------------------------

/// Report an error message with source location and exit.
/// TODO: not subset-conform
fn lexer_error(lexer: &Lexer, message: &str) -> ! {
    let file: &SourceFile = lexer_sourcefile(lexer);
    let line: usize = sourceFile_current_line(file);
    let col: usize = sourceFile_current_column(file);

    eprintln!("ERROR at {}:{}:", line, col);

    let mut start: usize = sourceFile_current_line_start(file);
    let mut reached_end: bool = false;
    while not(reached_end) {
        match sourceFile_get_char(file, start) {
            Option::Some('\n') => reached_end = true,
            Option::Some(c) => eprint!("{}", c),
            Option::None => reached_end = true,
        }
        start = start + 1;
    }
    eprintln!();

    let mut i: usize = 1;
    while i < col {
        eprint!(" ");
        i = i + 1;
    }
    eprintln!("^ {}", message);

    std::process::exit(1);
}

/// Emit an error at the parser current location and abort.
fn parser_error(parser: &Parser, message: &str) -> ! {
    lexer_error(parser_lexer(parser), message)
}

/// Emit an LLVM parser error and panic.
fn llvmParser_error(parser: &LlvmParser, message: &str) -> ! {
    let file: &SourceFile = llvmLexer_sourcefile(llvmParser_lexer(parser));
    let line: usize = sourceFile_current_line(file);
    let col: usize = sourceFile_current_column(file);

    eprintln!("ERROR at {}:{}:", line, col);

    let mut start: usize = sourceFile_current_line_start(file);
    let mut reached_end: bool = false;
    while not(reached_end) {
        match sourceFile_get_char(file, start) {
            Option::Some('\n') => reached_end = true,
            Option::Some(c) => eprint!("{}", c),
            Option::None => reached_end = true,
        }
        start = start + 1;
    }
    eprintln!();

    let mut i: usize = 1;
    while i < col {
        eprint!(" ");
        i = i + 1;
    }
    eprintln!("^ {}", message);

    std::process::exit(1);
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

/// Optional type that can contain some value with type T or no value.
enum Option<T> {
    Some(T),
    None,
}

/// Check whether an Option value is the None variant.
fn option_is_none<T>(opt: &Option<T>) -> bool {
    match opt {
        Option::Some(_) => false,
        Option::None => true,
    }
}

/// Returns the value wrapped in Some.
/// If the Option is None, end the program.
fn unwrap<T>(opt: Option<T>) -> T {
    match opt {
        Option::Some(value) => value,
        Option::None => panic!("tried to unwrap None variant of Option<T>"),
    }
}

// -----------------------------------------------------------------
// -------------------------- Lists --------------------------------
// -----------------------------------------------------------------

/// Generic cons list.
enum List<T> {
    /// head, tail
    Cons(T, Box<List<T>>),
    Nil,
}

/// Create an empty list.
fn list_new<T>() -> List<T> {
    List::Nil
}

/// Append one value to a list.
fn list_append<T>(list: &mut List<T>, value: T) {
    let mut current: &mut List<T> = list;

    while true {
        match current {
            List::Nil => {
                *current = List::Cons(value, box_new::<List<T>>(List::Nil));
                return;
            }
            List::Cons(_, tail) => current = box_deref_mut::<List<T>>(tail),
        }
    }
}

// ----------------------------------------------------------------
// --------------------------- Box --------------------------------
// ----------------------------------------------------------------

/// Pointer to heap that owns its value.
#[derive(Debug)]
enum Box<T> {
    Ptr(*mut T),
}

/// Allocate and box a value on the heap.
fn box_new<T>(value: T) -> Box<T> {
    let ptr_u8: *mut u8 = alloc(std::mem::size_of::<T>(), std::mem::align_of::<T>());
    let ptr: *mut T = ptr_u8 as *mut T;
    unsafe { *ptr = value };
    Box::Ptr(ptr)
}

/// Dereference a box.
fn box_deref<T>(ptr_wrap: &Box<T>) -> &T {
    let Box::Ptr(ptr): &Box<T> = ptr_wrap;
    unsafe { &**ptr }
}

/// Mutably dereference a box.
fn box_deref_mut<T>(ptr_wrap: &mut Box<T>) -> &mut T {
    let Box::Ptr(ptr): &mut Box<T> = ptr_wrap;
    unsafe { &mut **ptr }
}

/// Clone a boxed value.
fn box_clone<T>(ptr: &Box<T>, clone_fn: fn(&T) -> T) -> Box<T> {
    box_new::<T>(clone_fn(box_deref::<T>(ptr)))
}

// ----------------------------------------------------------------
// --------------------------- Vec --------------------------------
// ----------------------------------------------------------------

/// Generic contiguous growable buffer.
#[derive(Debug)]
enum Vec<T> {
    /// start, length, capacity
    Vec(*mut T, usize, usize),
}

/// Create an empty vector.
fn vec_new<T>() -> Vec<T> {
    vec_with_capacity::<T>(10)
}

/// Create a vector with fixed starting capacity.
fn vec_with_capacity<T>(initial_capacity: usize) -> Vec<T> {
    let capacity: usize = if initial_capacity == 0 {
        1
    } else {
        initial_capacity
    };
    let elem_size: usize = std::mem::size_of::<T>();
    let byte_len: usize = if elem_size == 0 {
        capacity
    } else {
        capacity * elem_size
    };
    let ptr: *mut T = alloc(byte_len, std::mem::align_of::<T>()) as *mut T;
    Vec::Vec(ptr, 0, capacity)
}

/// Get the backing pointer.
fn vec_ptr<T>(vec: &Vec<T>) -> *mut T {
    let Vec::Vec(ptr, _, _): &Vec<T> = vec;
    *ptr
}

/// Get the logical length.
fn vec_len<T>(vec: &Vec<T>) -> usize {
    let Vec::Vec(_, len, _): &Vec<T> = vec;
    *len
}

/// Get the capacity.
fn vec_capacity<T>(vec: &Vec<T>) -> usize {
    let Vec::Vec(_, _, capacity): &Vec<T> = vec;
    *capacity
}

/// Ensure capacity for extra elements.
fn vec_accomodate_extra_space<T>(vec: &mut Vec<T>, space: usize) {
    let len: usize = vec_len::<T>(vec);
    let capacity: usize = vec_capacity::<T>(vec);
    if capacity < len + space {
        let Vec::Vec(ptr, len_ref, capacity_ref): &mut Vec<T> = vec;
        *capacity_ref = *capacity_ref * 2;

        let elem_size: usize = std::mem::size_of::<T>();
        let new_byte_len: usize = if elem_size == 0 {
            *capacity_ref
        } else {
            *capacity_ref * elem_size
        };

        let new_ptr: *mut T = alloc(new_byte_len, std::mem::align_of::<T>()) as *mut T;
        unsafe { memcopy::<T>(new_ptr, *ptr, *len_ref) };
        *ptr = new_ptr;
        vec_accomodate_extra_space::<T>(vec, space); // if doubling was not enough, double again
    }
}

/// Append one element.
fn vec_push<T>(vec: &mut Vec<T>, value: T) {
    vec_accomodate_extra_space::<T>(vec, 1);
    let Vec::Vec(ptr, len, _): &mut Vec<T> = vec;
    unsafe { *ptr_add::<T>(*ptr, *len) = value }
    *len = *len + 1;
}

/// Set vector length after writing raw bytes/elements.
fn vec_set_len<T>(vec: &mut Vec<T>, len: usize) {
    let Vec::Vec(_, old_len, _): &mut Vec<T> = vec;
    *old_len = len;
}

/// Get an immutable reference to an element by index.
fn vec_get<'a, T>(vec: &'a Vec<T>, index: usize) -> Option<&'a T> {
    if index >= vec_len::<T>(vec) {
        Option::None
    } else {
        let ptr: *mut T = ptr_add::<T>(vec_ptr::<T>(vec), index);
        unsafe { Option::Some(&*ptr) }
    }
}

/// Get a mutable reference to an element by index.
fn vec_get_mut<'a, T>(vec: &'a mut Vec<T>, index: usize) -> Option<&'a mut T> {
    if index >= vec_len::<T>(vec) {
        Option::None
    } else {
        let ptr: *mut T = ptr_add::<T>(vec_ptr::<T>(vec), index);
        unsafe { Option::Some(&mut *ptr) }
    }
}

/// Set a value at the given index. Return false if the index is out of bounds.
fn vec_set<T>(vec: &mut Vec<T>, index: usize, value: T) -> bool {
    if index >= vec_len::<T>(vec) {
        false
    } else {
        let ptr: *mut T = vec_ptr::<T>(vec);
        let ptr: *mut T = ptr_add::<T>(ptr, index);
        unsafe {
            *ptr = value;
        }
        true
    }
}

/// Append all elements from one vector to another.
fn vec_extend<T>(vec: &mut Vec<T>, other: &Vec<T>) {
    let other_len: usize = vec_len::<T>(other);
    vec_accomodate_extra_space::<T>(vec, other_len);

    let len: usize = vec_len::<T>(vec);
    let dest: *mut T = ptr_add::<T>(vec_ptr::<T>(vec), len);
    let src: *mut T = vec_ptr::<T>(other);
    unsafe { memcopy::<T>(dest, src, other_len) };
    vec_set_len::<T>(vec, len + other_len);
}

// ----------------------------------------------------------------
// ------------------------ StringMap -----------------------------
// ----------------------------------------------------------------

/// Bucket entry for StringMap.
enum StringMapEntry<T> {
    Entry(String, T),
}

/// Get the key stored in one StringMapEntry.
fn stringMapEntry_get_key<T>(entry: &StringMapEntry<T>) -> &String {
    let StringMapEntry::Entry(key, _) = entry;
    key
}

/// Get the value stored in one StringMapEntry.
fn stringMapEntry_get_value<T>(entry: &StringMapEntry<T>) -> &T {
    let StringMapEntry::Entry(_, value) = entry;
    value
}

/// Hash map from String keys to generic values.
enum StringMap<T> {
    Map(Vec<List<StringMapEntry<T>>>),
}

/// Create a map with default len.
fn stringMap_new<T>() -> StringMap<T> {
    stringMap_with_len::<T>(1024)
}

/// Create a map with explicit len.
fn stringMap_with_len<T>(len: usize) -> StringMap<T> {
    let bucket_len: usize = if len == 0 { 1 } else { len };
    let mut buckets: Vec<List<StringMapEntry<T>>> =
        vec_with_capacity::<List<StringMapEntry<T>>>(bucket_len);
    let mut i: usize = 0;
    while i < bucket_len {
        vec_push::<List<StringMapEntry<T>>>(&mut buckets, List::Nil);
        i = i + 1;
    }
    StringMap::Map(buckets)
}

/// Insert a key/value pair by prepending it to the bucket list.
fn stringMap_insert<T>(map: &mut StringMap<T>, key: String, value: T) {
    let bucket_index: usize = {
        let StringMap::Map(buckets): &StringMap<T> = map;
        string_hash(&key, vec_len::<List<StringMapEntry<T>>>(buckets))
    };

    let StringMap::Map(buckets): &mut StringMap<T> = map;
    let bucket: &mut List<StringMapEntry<T>> = unwrap::<&mut List<StringMapEntry<T>>>(
        vec_get_mut::<List<StringMapEntry<T>>>(buckets, bucket_index),
    );

    let mut old_bucket: List<StringMapEntry<T>> = List::Nil;
    unsafe {
        memcopy::<List<StringMapEntry<T>>>(
            &mut old_bucket as *mut List<StringMapEntry<T>>,
            bucket as *mut List<StringMapEntry<T>>,
            1,
        );
    }

    *bucket = List::Cons(
        StringMapEntry::Entry(key, value),
        box_new::<List<StringMapEntry<T>>>(old_bucket),
    );
}

/// Get a shared reference to the value for a key.
fn stringMap_get<'a, T>(map: &'a StringMap<T>, key: &String) -> Option<&'a T> {
    let StringMap::Map(buckets): &'a StringMap<T> = map;
    let bucket_index: usize = string_hash(key, vec_len::<List<StringMapEntry<T>>>(buckets));

    let maybe_bucket: Option<&List<StringMapEntry<T>>> =
        vec_get::<List<StringMapEntry<T>>>(buckets, bucket_index);
    if option_is_none::<&List<StringMapEntry<T>>>(&maybe_bucket) {
        return Option::None;
    }
    let mut bucket: &List<StringMapEntry<T>> = unwrap::<&List<StringMapEntry<T>>>(maybe_bucket);

    while true {
        match bucket {
            List::Cons(entry, tail) => {
                let other_key: &String = stringMapEntry_get_key::<T>(entry);
                if string_eq(other_key, key) {
                    return Option::Some(stringMapEntry_get_value::<T>(entry));
                }

                // repeat with next bucket
                bucket = box_deref::<List<StringMapEntry<T>>>(tail);
            }

            List::Nil => return Option::None,
        }
    }
    Option::None
}

/// Check whether a key exists.
fn stringMap_contains<T>(map: &StringMap<T>, key: &String) -> bool {
    match stringMap_get::<T>(map, key) {
        Option::Some(_) => true,
        Option::None => false,
    }
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
        Type::Reference(left, left_mut) => match b {
            Type::Reference(right, right_mut) => and(
                *left_mut == *right_mut,
                type_eq(box_deref::<Type>(left), box_deref::<Type>(right)),
            ),
            _ => false,
        },
        Type::RawPointerMut(left) => match b {
            Type::RawPointerMut(right) => {
                type_eq(box_deref::<Type>(left), box_deref::<Type>(right))
            }
            _ => false,
        },
    }
}

/// Compare two lists in order using an element equality function.
fn list_eq<T>(left: &List<T>, right: &List<T>, item_eq: fn(&T, &T) -> bool) -> bool {
    match left {
        List::Nil => match right {
            List::Nil => true,
            _ => false,
        },
        List::Cons(lhead, ltail) => match right {
            List::Cons(rhead, rtail) => and(
                item_eq(lhead, rhead),
                list_eq::<T>(
                    box_deref::<List<T>>(ltail),
                    box_deref::<List<T>>(rtail),
                    item_eq,
                ),
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
        let c1: char = unwrap::<char>(string_get(s1, i));
        let c2: char = unwrap::<char>(string_get(s2, i));
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
            SymTableEntry::Enum(string_clone(name), list_clone::<Type>(variants, type_clone))
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
        GlobalSymTable::Cons(head, tail) => GlobalSymTable::Cons(
            symTableEntry_clone(head),
            box_clone::<GlobalSymTable>(tail, globalSymTable_clone),
        ),
    }
}

/// Clone a local scope symbol table.
fn localSymTable_clone(symtable: &LocalSymTable) -> LocalSymTable {
    match symtable {
        LocalSymTable::Nil => LocalSymTable::Nil,
        LocalSymTable::Cons(head, tail) => LocalSymTable::Cons(
            symTableEntry_clone(head),
            box_clone::<LocalSymTable>(tail, localSymTable_clone),
        ),
    }
}

/// Clone the stack of local scopes.
fn localSymTableStack_clone(stack: &LocalSymTableStack) -> LocalSymTableStack {
    match stack {
        LocalSymTableStack::Nil => LocalSymTableStack::Nil,
        LocalSymTableStack::Cons(local, tail) => LocalSymTableStack::Cons(
            localSymTable_clone(local),
            box_clone::<LocalSymTableStack>(tail, localSymTableStack_clone),
        ),
    }
}

/// Clone a function signature.
fn fnSignature_clone(signature: &FnSignature) -> FnSignature {
    match signature {
        FnSignature::Fn(parameter_types, return_type) => FnSignature::Fn(
            list_clone::<Type>(parameter_types, type_clone),
            type_clone(return_type),
        ),
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
        Type::Reference(inner, mutable) => {
            Type::Reference(box_clone::<Type>(inner, type_clone), *mutable)
        }
        Type::RawPointerMut(inner) => Type::RawPointerMut(box_clone::<Type>(inner, type_clone)),
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

/// Clone one LLVM local symbol table entry.
fn llvmLocalSymTableEntry_clone(entry: &LlvmLocalSymTableEntry) -> LlvmLocalSymTableEntry {
    match entry {
        LlvmLocalSymTableEntry::Register(name, value) => {
            LlvmLocalSymTableEntry::Register(string_clone(name), *value)
        }
    }
}

/// Clone an LLVM local symbol table.
fn llvmLocalSymTable_clone(symtable: &LlvmLocalSymTable) -> LlvmLocalSymTable {
    match symtable {
        LlvmLocalSymTable::Registers(entries) => {
            LlvmLocalSymTable::Registers(list_clone::<LlvmLocalSymTableEntry>(
                entries,
                llvmLocalSymTableEntry_clone,
            ))
        }
    }
}

/// Clone a list using an explicit element clone function.
fn list_clone<T>(values: &List<T>, item_clone: fn(&T) -> T) -> List<T> {
    match values {
        List::Nil => List::Nil,
        List::Cons(head, tail) => List::Cons(
            item_clone(head),
            box_new::<List<T>>(list_clone::<T>(box_deref::<List<T>>(tail), item_clone)),
        ),
    }
}

/// Clone a string.
fn string_clone(string: &String) -> String {
    let len: usize = string_len(string);

    let mut clone: String = string_with_capacity(len);
    let mut i: usize = 0;
    while i < len {
        let character: char = unwrap::<char>(string_get(string, i));
        string_push(&mut clone, character);
        i = i + 1;
    }
    clone
}

// ------------------------- String -------------------------------

/// A growable ASCII string.
#[derive(Debug)]
enum String {
    Inner(Vec<u8>),
}

/// Create a new empty string.
fn string_new() -> String {
    string_with_capacity(10)
}

/// Create a new string with the specified capacity
fn string_with_capacity(initial_capacity: usize) -> String {
    String::Inner(vec_with_capacity::<u8>(initial_capacity))
}

/// Create a string from a string slice.
fn string_from_str(str: &str) -> String {
    let mut s: String = string_new();
    string_push_str(&mut s, str);
    s
}

/// Get the length of the string.
fn string_len(string: &String) -> usize {
    let String::Inner(bytes): &String = string;
    vec_len::<u8>(bytes)
}

/// Get the character at the given index.
fn string_get(string: &String, index: usize) -> Option<char> {
    let String::Inner(bytes): &String = string;
    match vec_get::<u8>(bytes, index) {
        Option::Some(value) => Option::Some(*value as char),
        Option::None => Option::None,
    }
}

/// Set a character in a string. Return false if the index is out of bounds.
fn string_set(string: &mut String, index: usize, character: char) -> bool {
    let String::Inner(vec): &mut String = string;
    vec_set::<u8>(vec, index, character as u8)
}

/// Append a character to the string.
fn string_push(string: &mut String, character: char) {
    let String::Inner(bytes): &mut String = string;
    vec_push::<u8>(bytes, character as u8);
}

/// Append a string slice to the string.
fn string_push_str(string: &mut String, str: &str) {
    let str_len: usize = str.len();
    let String::Inner(bytes): &mut String = string;
    vec_accomodate_extra_space::<u8>(bytes, str_len);

    let str_ptr: *mut u8 = str.as_ptr() as *mut u8;
    let len: usize = vec_len::<u8>(bytes);
    let dest: *mut u8 = ptr_add::<u8>(vec_ptr::<u8>(bytes), len);

    unsafe { memcopy::<u8>(dest, str_ptr, str_len) }
    vec_set_len::<u8>(bytes, len + str_len);
}

/// Push a string onto another string.
fn string_push_string(string: &mut String, other: &String) {
    let String::Inner(bytes): &mut String = string;
    let String::Inner(other_bytes): &String = other;
    vec_extend::<u8>(bytes, other_bytes);
}

/// Converts a string into an integer given the base.
/// Returns None if the integer contained in the string is invalid for 64-bit integers.
fn string_to_integer(string: &mut String, base: usize) -> Option<usize> {
    let mut value: usize = 0;

    let mut i: usize = 0;
    while i < string_len(string) {
        let digit: char = unwrap::<char>(string_get(string, i));

        let digit_value: usize = if is_digit(digit) {
            digit as usize - '0' as usize
        } else {
            digit as usize - 'A' as usize + 10
        };

        let max: usize = 18446744073709551615; // 2^64 - 1

        if or(digit_value > base - 1, value > max / base) {
            return Option::None;
        }

        value = value * base + digit_value;

        i = i + 1;
    }
    Option::Some(value)
}

/// Convert an integer into a string.
fn integer_to_string(mut integer: usize) -> String {
    let mut string: String = string_new();

    if integer == 0 {
        string_push(&mut string, '0');
        return string;
    }

    while integer > 0 {
        let digit: u8 = (integer % 10) as u8;
        let character: char = ('0' as u8 + digit) as char;
        string_push(&mut string, character);
        integer = integer / 10;
    }

    string_reverse(&mut string);
    string
}

/// Reverse a String in place.
fn string_reverse(string: &mut String) {
    let len: usize = string_len(string);
    let mut i: usize = 0;
    while i < len / 2 {
        let a: char = unwrap::<char>(string_get(string, i));
        let b: char = unwrap::<char>(string_get(string, len - 1 - i));
        string_set(string, i, b);
        string_set(string, len - 1 - i, a);
        i = i + 1;
    }
}

/// Hash a String.
fn string_hash(string: &String, bucket_count: usize) -> usize {
    if bucket_count == 0 {
        return 0;
    }

    let mut hash: usize = 0;
    let mut i: usize = 0;
    while i < string_len(string) {
        let character: usize = unwrap::<char>(string_get(string, i)) as usize;
        hash = hash * 67 + character;
        i = i + 1;
    }
    hash % bucket_count
}

/// Ensure the string has space for additional bytes.
fn string_accomodate_extra_space(string: &mut String, space: usize) {
    let String::Inner(bytes): &mut String = string;
    vec_accomodate_extra_space::<u8>(bytes, space);
}

// ------------------------- Memory -------------------------------

/// Copy n bytes from src to dest.
///
/// It must hold: forall 0 <= i < n, dest[i] can be written
/// and src[i] can be read safely.
unsafe fn memcopy<T>(dest: *mut T, src: *mut T, n: usize) {
    let byte_count: usize = n * std::mem::size_of::<T>();
    let dest_u8: *mut u8 = dest as *mut u8;
    let src_u8: *mut u8 = src as *mut u8;
    let mut i: usize = 0;
    while i < byte_count {
        unsafe {
            *ptr_add::<u8>(dest_u8, i) = *ptr_add::<u8>(src_u8, i);
        }
        i = i + 1;
    }
}

/// Increment a pointer by n elements.
fn ptr_add<T>(ptr: *mut T, n: usize) -> *mut T {
    (ptr as usize + n * std::mem::size_of::<T>()) as *mut T
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
