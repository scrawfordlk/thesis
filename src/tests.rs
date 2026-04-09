// Tests for main.rs
// Note: Tests are written in full Rust (unlike the code in main.rs, which
// is written in the restricted subset of Rust).

mod tests {
    #[allow(unused_imports)]
    use super::{
        CharOption, Comparison, Lexer, Literal, SourceFile, SourceLocation, String, Token, Type,
        alloc, and, identifier_to_token, is_alpha, is_alphanumeric, is_digit, is_whitespace,
        lexer_consume, lexer_error, lexer_expect_char, lexer_location, lexer_peek,
        lexer_sourcefile, memcopy, next_token, or, ptr_add, scan_bang, scan_char_literal,
        scan_colon, scan_equals,
        scan_escape_char, scan_greater, scan_identifier, scan_integer, scan_less, scan_minus,
        scan_slash, scan_string_literal, scan_symbol, skip_line_comment, skip_whitespace,
        string_accomodate_extra_space, string_capacity, string_eq, string_from_str, string_get,
        string_len, string_new, string_ptr, string_push, string_push_str, unwrap_char,
    };

    // Helper to convert our String to std::string::String for easy comparison
    fn to_std_string(s: &String) -> std::string::String {
        (0..string_len(s))
            .map(|i| unwrap_char(string_get(s, i)))
            .collect()
    }

    // ----------------------- CharOption --------------------------

    #[test]
    fn test_unwrap_char_some() {
        assert_eq!(unwrap_char(CharOption::Some('a')), 'a');
    }

    #[test]
    #[should_panic(expected = "unwrap failed")]
    fn test_unwrap_char_none() {
        unwrap_char(CharOption::None);
    }

    // ------------------------- String ----------------------------

    #[test]
    fn test_string_new() {
        let s = string_new();
        assert_eq!(string_len(&s), 0);
        assert_eq!(string_capacity(&s), 1);
    }

    #[test]
    fn test_string_push() {
        let mut s = string_new();
        string_push(&mut s, 'H');
        assert_eq!(string_len(&s), 1);
        assert_eq!(to_std_string(&s), "H");
    }

    #[test]
    fn test_string_push_multiple() {
        let mut s = string_new();
        for c in ['a', 'b', 'c'] {
            string_push(&mut s, c);
        }
        assert_eq!(string_len(&s), 3);
        assert_eq!(to_std_string(&s), "abc");
    }

    #[test]
    fn test_string_push_str() {
        let mut s = string_new();
        string_push_str(&mut s, "Hello");
        assert_eq!(string_len(&s), 5);
        assert_eq!(to_std_string(&s), "Hello");
    }

    #[test]
    fn test_string_push_str_empty() {
        let mut s = string_new();
        string_push_str(&mut s, "");
        assert_eq!(string_len(&s), 0);
        assert_eq!(to_std_string(&s), "");
    }

    #[test]
    fn test_string_push_and_push_str_combined() {
        let mut s = string_new();
        string_push_str(&mut s, "Hi");
        string_push(&mut s, '!');
        assert_eq!(to_std_string(&s), "Hi!");
    }

    #[test]
    fn test_string_get_out_of_bounds() {
        let s = string_new();
        assert!(matches!(string_get(&s, 0), CharOption::None));
    }

    #[test]
    fn test_string_get_out_of_bounds_nonempty() {
        let mut s = string_new();
        string_push(&mut s, 'x');
        assert!(matches!(string_get(&s, 1), CharOption::None));
    }

    #[test]
    fn test_string_capacity_grows() {
        let mut s = string_new();
        let initial_cap = string_capacity(&s);
        for _ in 0..(initial_cap + 5) {
            string_push(&mut s, 'x');
        }
        assert!(string_capacity(&s) > initial_cap);
        assert_eq!(string_len(&s), initial_cap + 5);
    }

    // ------------------------- Memory ----------------------------

    #[test]
    fn test_ptr_add() {
        let data: [u8; 4] = [10, 20, 30, 40];
        let ptr = data.as_ptr() as *mut u8;
        unsafe {
            for (i, &expected) in data.iter().enumerate() {
                assert_eq!(*ptr_add(ptr, i), expected);
            }
        }
    }

    #[test]
    fn test_memcopy() {
        let src = [1u8, 2, 3, 4];
        let mut dest = [0u8; 4];
        unsafe { memcopy(dest.as_mut_ptr(), src.as_ptr() as *mut u8, 4) };
        assert_eq!(dest, src);
    }

    #[test]
    fn test_memcopy_partial() {
        let src = [5u8, 6, 7, 8];
        let mut dest = [0u8; 4];
        unsafe { memcopy(dest.as_mut_ptr(), src.as_ptr() as *mut u8, 2) };
        assert_eq!(dest, [5, 6, 0, 0]);
    }

    #[test]
    fn test_memcopy_zero() {
        let src = [1u8, 2, 3, 4];
        let mut dest = [0u8; 4];
        unsafe { memcopy(dest.as_mut_ptr(), src.as_ptr() as *mut u8, 0) };
        assert_eq!(dest, [0; 4]);
    }

    #[test]
    fn test_alloc() {
        let ptr = alloc(16, 1);
        assert!(!ptr.is_null());
        // Verify zeroed allocation
        unsafe {
            for i in 0..16 {
                assert_eq!(*ptr_add(ptr, i), 0);
            }
        }
    }

    // ------------------------- Lexer Helpers ----------------------------

    fn make_lexer(input: &str) -> Lexer {
        let mut content = string_new();
        string_push_str(&mut content, input);
        let source = SourceFile::SourceFile(content, 0, SourceLocation::Coords(1, 1));
        Lexer::Lexer(source, Token::Eof)
    }

    fn collect_tokens(lexer: &mut Lexer) -> Vec<Token> {
        let mut tokens = Vec::new();
        loop {
            let tok = next_token(lexer);
            let is_eof = matches!(tok, Token::Eof);
            tokens.push(tok);
            if is_eof {
                break;
            }
        }
        tokens
    }

    fn ident(s: &str) -> Token {
        let mut string = string_new();
        string_push_str(&mut string, s);
        Token::Identifier(string)
    }

    fn str_lit(s: &str) -> Token {
        let mut string = string_new();
        string_push_str(&mut string, s);
        Token::Literal(Literal::String(string))
    }

    fn tokens_match(a: &Token, b: &Token) -> bool {
        match (a, b) {
            (Token::Fn, Token::Fn) => true,
            (Token::Enum, Token::Enum) => true,
            (Token::Let, Token::Let) => true,
            (Token::If, Token::If) => true,
            (Token::Else, Token::Else) => true,
            (Token::While, Token::While) => true,
            (Token::Return, Token::Return) => true,
            (Token::Match, Token::Match) => true,
            (Token::As, Token::As) => true,
            (Token::Mut, Token::Mut) => true,
            (Token::Ampersand, Token::Ampersand) => true,
            (Token::LBrace, Token::LBrace) => true,
            (Token::RBrace, Token::RBrace) => true,
            (Token::LParen, Token::LParen) => true,
            (Token::RParen, Token::RParen) => true,
            (Token::Colon, Token::Colon) => true,
            (Token::DoubleColon, Token::DoubleColon) => true,
            (Token::SemiColon, Token::SemiColon) => true,
            (Token::Comma, Token::Comma) => true,
            (Token::Assign, Token::Assign) => true,
            (Token::ArmArrow, Token::ArmArrow) => true,
            (Token::Plus, Token::Plus) => true,
            (Token::Minus, Token::Minus) => true,
            (Token::Star, Token::Star) => true,
            (Token::Slash, Token::Slash) => true,
            (Token::Remainder, Token::Remainder) => true,
            (Token::TypeArrow, Token::TypeArrow) => true,
            (Token::Eof, Token::Eof) => true,
            (Token::Comparison(c1), Token::Comparison(c2)) => comparisons_match(c1, c2),
            (Token::Type(t1), Token::Type(t2)) => types_match(t1, t2),
            (Token::Literal(l1), Token::Literal(l2)) => literals_match(l1, l2),
            (Token::Identifier(s1), Token::Identifier(s2)) => string_eq(s1, s2),
            (Token::SizeOf(n1), Token::SizeOf(n2)) => n1 == n2,
            _ => false,
        }
    }

    fn comparisons_match(a: &Comparison, b: &Comparison) -> bool {
        matches!(
            (a, b),
            (Comparison::Eq, Comparison::Eq)
                | (Comparison::Neq, Comparison::Neq)
                | (Comparison::Gt, Comparison::Gt)
                | (Comparison::Lt, Comparison::Lt)
                | (Comparison::Geq, Comparison::Geq)
                | (Comparison::Leq, Comparison::Leq)
        )
    }

    fn types_match(a: &Type, b: &Type) -> bool {
        matches!(
            (a, b),
            (Type::Usize, Type::Usize)
                | (Type::U8, Type::U8)
                | (Type::Char, Type::Char)
                | (Type::Str, Type::Str)
        )
    }

    fn literals_match(a: &Literal, b: &Literal) -> bool {
        match (a, b) {
            (Literal::Integer(n1), Literal::Integer(n2)) => n1 == n2,
            (Literal::Character(c1), Literal::Character(c2)) => c1 == c2,
            (Literal::String(s1), Literal::String(s2)) => string_eq(s1, s2),
            _ => false,
        }
    }

    fn assert_tokens(actual: Vec<Token>, expected: Vec<Token>) {
        assert_eq!(
            actual.len(),
            expected.len(),
            "token count mismatch: got {}, expected {}",
            actual.len(),
            expected.len()
        );
        for (i, (a, e)) in actual.iter().zip(expected.iter()).enumerate() {
            assert!(
                tokens_match(a, e),
                "token {} mismatch",
                i
            );
        }
    }

    // ------------------------- Bool ----------------------------

    #[test]
    fn test_and() {
        assert_eq!(and(true, true), true);
        assert_eq!(and(true, false), false);
        assert_eq!(and(false, true), false);
        assert_eq!(and(false, false), false);
    }

    #[test]
    fn test_or() {
        assert_eq!(or(true, true), true);
        assert_eq!(or(true, false), true);
        assert_eq!(or(false, true), true);
        assert_eq!(or(false, false), false);
    }

    // ------------------------- Char Classification ----------------------------

    #[test]
    fn test_is_whitespace() {
        assert!(is_whitespace(' '));
        assert!(is_whitespace('\t'));
        assert!(is_whitespace('\n'));
        assert!(is_whitespace('\r'));
        assert!(!is_whitespace('a'));
        assert!(!is_whitespace('0'));
    }

    #[test]
    fn test_is_digit() {
        for c in '0'..='9' {
            assert!(is_digit(c));
        }
        assert!(!is_digit('a'));
        assert!(!is_digit(' '));
    }

    #[test]
    fn test_is_alpha() {
        for c in 'a'..='z' {
            assert!(is_alpha(c));
        }
        for c in 'A'..='Z' {
            assert!(is_alpha(c));
        }
        assert!(is_alpha('_'));
        assert!(!is_alpha('0'));
        assert!(!is_alpha(' '));
    }

    #[test]
    fn test_is_alphanumeric() {
        assert!(is_alphanumeric('a'));
        assert!(is_alphanumeric('Z'));
        assert!(is_alphanumeric('_'));
        assert!(is_alphanumeric('0'));
        assert!(is_alphanumeric('9'));
        assert!(!is_alphanumeric(' '));
        assert!(!is_alphanumeric('+'));
    }

    // ------------------------- Lexer Helpers ----------------------------

    #[test]
    fn test_lexer_peek() {
        let lexer = make_lexer("a");
        assert!(matches!(lexer_peek(&lexer), CharOption::Some('a')));
    }

    #[test]
    fn test_lexer_peek_empty() {
        let lexer = make_lexer("");
        assert!(matches!(lexer_peek(&lexer), CharOption::None));
    }

    #[test]
    fn test_lexer_consume() {
        let mut lexer = make_lexer("ab");
        assert!(matches!(lexer_consume(&mut lexer), CharOption::Some('a')));
        assert!(matches!(lexer_consume(&mut lexer), CharOption::Some('b')));
        assert!(matches!(lexer_consume(&mut lexer), CharOption::None));
    }

    #[test]
    fn test_lexer_eof_detection() {
        let mut lexer = make_lexer("a");
        assert!(matches!(lexer_peek(&lexer), CharOption::Some('a')));
        lexer_consume(&mut lexer);
        assert!(matches!(lexer_peek(&lexer), CharOption::None));
    }

    #[test]
    fn test_lexer_location_tracks_line_col() {
        let mut lexer = make_lexer("a\nb");
        let loc = lexer_location(&lexer);
        let SourceLocation::Coords(line, col) = loc;
        assert_eq!((*line, *col), (1, 1));

        lexer_consume(&mut lexer); // 'a'
        let loc = lexer_location(&lexer);
        let SourceLocation::Coords(line, col) = loc;
        assert_eq!((*line, *col), (1, 2));

        lexer_consume(&mut lexer); // '\n'
        let loc = lexer_location(&lexer);
        let SourceLocation::Coords(line, col) = loc;
        assert_eq!((*line, *col), (2, 1));
    }

    #[test]
    fn test_lexer_sourcefile() {
        let lexer = make_lexer("abc");
        let SourceFile::SourceFile(_, index, _) = lexer_sourcefile(&lexer);
        assert_eq!(*index, 0);
    }

    #[test]
    fn test_lexer_expect_char_success() {
        let mut lexer = make_lexer("xyz");
        lexer_expect_char(&mut lexer, 'x');
        assert!(matches!(lexer_peek(&lexer), CharOption::Some('y')));
    }

    #[test]
    fn test_scan_identifier_direct() {
        let mut lexer = make_lexer("hello_42!");
        let ident = scan_identifier(&mut lexer);
        assert!(string_eq(&ident, &string_from_str("hello_42")));
        assert!(matches!(lexer_peek(&lexer), CharOption::Some('!')));
    }

    #[test]
    fn test_identifier_to_token_direct_keyword() {
        let tok = identifier_to_token(string_from_str("usize"));
        assert!(matches!(tok, Token::Type(Type::Usize)));
    }

    #[test]
    fn test_identifier_to_token_direct_identifier() {
        let tok = identifier_to_token(string_from_str("my_var"));
        match tok {
            Token::Identifier(s) => assert!(string_eq(&s, &string_from_str("my_var"))),
            _ => assert!(false, "expected identifier token"),
        }
    }

    #[test]
    fn test_scan_integer_direct() {
        let mut lexer = make_lexer("123abc");
        let value = scan_integer(&mut lexer);
        assert_eq!(value, 123);
        assert!(matches!(lexer_peek(&lexer), CharOption::Some('a')));
    }

    #[test]
    fn test_scan_char_literal_direct() {
        let mut lexer = make_lexer("'x'");
        assert_eq!(scan_char_literal(&mut lexer), 'x');
        assert!(matches!(lexer_peek(&lexer), CharOption::None));
    }

    #[test]
    fn test_scan_string_literal_direct() {
        let mut lexer = make_lexer("\"ab\\n\"");
        let s = scan_string_literal(&mut lexer);
        assert!(string_eq(&s, &string_from_str("ab\n")));
        assert!(matches!(lexer_peek(&lexer), CharOption::None));
    }

    #[test]
    fn test_scan_escape_char_direct() {
        let mut lexer = make_lexer("n");
        assert_eq!(scan_escape_char(&mut lexer), '\n');
    }

    #[test]
    fn test_scan_symbol_direct() {
        let mut lexer = make_lexer("+");
        let tok = scan_symbol(&mut lexer);
        assert!(matches!(tok, Token::Plus));
    }

    #[test]
    fn test_scan_slash_direct() {
        let mut lexer = make_lexer("x");
        assert!(matches!(scan_slash(&mut lexer), Token::Slash));
        assert!(matches!(lexer_peek(&lexer), CharOption::Some('x')));
    }

    #[test]
    fn test_scan_colon_direct() {
        let mut lexer = make_lexer("x");
        assert!(matches!(scan_colon(&mut lexer), Token::Colon));
        assert!(matches!(lexer_peek(&lexer), CharOption::Some('x')));
    }

    #[test]
    fn test_scan_equals_direct() {
        let mut lexer = make_lexer(">");
        assert!(matches!(scan_equals(&mut lexer), Token::ArmArrow));
    }

    #[test]
    fn test_scan_minus_direct() {
        let mut lexer = make_lexer(">");
        assert!(matches!(scan_minus(&mut lexer), Token::TypeArrow));
    }

    #[test]
    fn test_scan_bang_direct() {
        let mut lexer = make_lexer("=");
        assert!(matches!(
            scan_bang(&mut lexer),
            Token::Comparison(Comparison::Neq)
        ));
    }

    #[test]
    fn test_scan_less_direct() {
        let mut lexer = make_lexer("=");
        assert!(matches!(
            scan_less(&mut lexer),
            Token::Comparison(Comparison::Leq)
        ));
    }

    #[test]
    fn test_scan_greater_direct() {
        let mut lexer = make_lexer("=");
        assert!(matches!(
            scan_greater(&mut lexer),
            Token::Comparison(Comparison::Geq)
        ));
    }

    #[test]
    fn test_skip_whitespace_direct() {
        let mut lexer = make_lexer("  \n\tabc");
        skip_whitespace(&mut lexer);
        assert!(matches!(lexer_peek(&lexer), CharOption::Some('a')));
    }

    #[test]
    fn test_skip_line_comment_direct() {
        let mut lexer = make_lexer("comment text\nz");
        skip_line_comment(&mut lexer);
        assert!(matches!(lexer_peek(&lexer), CharOption::Some('z')));
    }

    #[test]
    fn test_string_ptr_non_null() {
        let s = string_new();
        assert!(!string_ptr(&s).is_null());
    }

    #[test]
    fn test_string_accomodate_extra_space_direct() {
        let mut s = string_new();
        let before = string_capacity(&s);
        string_accomodate_extra_space(&mut s, before + 5);
        assert!(string_capacity(&s) >= before + 5);
    }

    #[test]
    fn test_string_from_str_direct() {
        let s = string_from_str("hello");
        assert!(string_eq(&s, &string_from_str("hello")));
    }

    #[test]
    fn test_lexer_error_exits() {
        if std::env::var("LEXER_ERROR_CHILD").as_deref() == Ok("1") {
            let lexer = make_lexer("");
            lexer_error(&lexer, "boom");
        }

        let output = std::process::Command::new(std::env::current_exe().expect("current exe"))
            .env("LEXER_ERROR_CHILD", "1")
            .arg("--exact")
            .arg("tests::test_lexer_error_exits")
            .output()
            .expect("spawn child");

        assert!(!output.status.success());
        assert_eq!(output.status.code(), Some(1));
    }

    // ------------------------- Keywords ----------------------------

    #[test]
    fn test_keyword_fn() {
        let mut lexer = make_lexer("fn");
        assert_tokens(collect_tokens(&mut lexer), vec![Token::Fn, Token::Eof]);
    }

    #[test]
    fn test_keyword_enum() {
        let mut lexer = make_lexer("enum");
        assert_tokens(collect_tokens(&mut lexer), vec![Token::Enum, Token::Eof]);
    }

    #[test]
    fn test_keyword_let() {
        let mut lexer = make_lexer("let");
        assert_tokens(collect_tokens(&mut lexer), vec![Token::Let, Token::Eof]);
    }

    #[test]
    fn test_keyword_if() {
        let mut lexer = make_lexer("if");
        assert_tokens(collect_tokens(&mut lexer), vec![Token::If, Token::Eof]);
    }

    #[test]
    fn test_keyword_else() {
        let mut lexer = make_lexer("else");
        assert_tokens(collect_tokens(&mut lexer), vec![Token::Else, Token::Eof]);
    }

    #[test]
    fn test_keyword_while() {
        let mut lexer = make_lexer("while");
        assert_tokens(collect_tokens(&mut lexer), vec![Token::While, Token::Eof]);
    }

    #[test]
    fn test_keyword_return() {
        let mut lexer = make_lexer("return");
        assert_tokens(collect_tokens(&mut lexer), vec![Token::Return, Token::Eof]);
    }

    #[test]
    fn test_keyword_match() {
        let mut lexer = make_lexer("match");
        assert_tokens(collect_tokens(&mut lexer), vec![Token::Match, Token::Eof]);
    }

    #[test]
    fn test_keyword_as() {
        let mut lexer = make_lexer("as");
        assert_tokens(collect_tokens(&mut lexer), vec![Token::As, Token::Eof]);
    }

    #[test]
    fn test_keyword_mut() {
        let mut lexer = make_lexer("mut");
        assert_tokens(collect_tokens(&mut lexer), vec![Token::Mut, Token::Eof]);
    }

    // ------------------------- Types ----------------------------

    #[test]
    fn test_type_usize() {
        let mut lexer = make_lexer("usize");
        assert_tokens(
            collect_tokens(&mut lexer),
            vec![Token::Type(Type::Usize), Token::Eof],
        );
    }

    #[test]
    fn test_type_u8() {
        let mut lexer = make_lexer("u8");
        assert_tokens(
            collect_tokens(&mut lexer),
            vec![Token::Type(Type::U8), Token::Eof],
        );
    }

    #[test]
    fn test_type_char() {
        let mut lexer = make_lexer("char");
        assert_tokens(
            collect_tokens(&mut lexer),
            vec![Token::Type(Type::Char), Token::Eof],
        );
    }

    #[test]
    fn test_type_str() {
        let mut lexer = make_lexer("str");
        assert_tokens(
            collect_tokens(&mut lexer),
            vec![Token::Type(Type::Str), Token::Eof],
        );
    }

    // ------------------------- Identifiers ----------------------------

    #[test]
    fn test_identifier_simple() {
        let mut lexer = make_lexer("foo");
        assert_tokens(collect_tokens(&mut lexer), vec![ident("foo"), Token::Eof]);
    }

    #[test]
    fn test_identifier_with_underscore() {
        let mut lexer = make_lexer("foo_bar");
        assert_tokens(
            collect_tokens(&mut lexer),
            vec![ident("foo_bar"), Token::Eof],
        );
    }

    #[test]
    fn test_identifier_with_numbers() {
        let mut lexer = make_lexer("foo123");
        assert_tokens(
            collect_tokens(&mut lexer),
            vec![ident("foo123"), Token::Eof],
        );
    }

    #[test]
    fn test_identifier_starting_with_underscore() {
        let mut lexer = make_lexer("_private");
        assert_tokens(
            collect_tokens(&mut lexer),
            vec![ident("_private"), Token::Eof],
        );
    }

    // ------------------------- Integer Literals ----------------------------

    #[test]
    fn test_integer_zero() {
        let mut lexer = make_lexer("0");
        assert_tokens(
            collect_tokens(&mut lexer),
            vec![Token::Literal(Literal::Integer(0)), Token::Eof],
        );
    }

    #[test]
    fn test_integer_single_digit() {
        let mut lexer = make_lexer("7");
        assert_tokens(
            collect_tokens(&mut lexer),
            vec![Token::Literal(Literal::Integer(7)), Token::Eof],
        );
    }

    #[test]
    fn test_integer_multi_digit() {
        let mut lexer = make_lexer("12345");
        assert_tokens(
            collect_tokens(&mut lexer),
            vec![Token::Literal(Literal::Integer(12345)), Token::Eof],
        );
    }

    // ------------------------- Character Literals ----------------------------

    #[test]
    fn test_char_literal_simple() {
        let mut lexer = make_lexer("'a'");
        assert_tokens(
            collect_tokens(&mut lexer),
            vec![Token::Literal(Literal::Character('a')), Token::Eof],
        );
    }

    #[test]
    fn test_char_literal_escape_n() {
        let mut lexer = make_lexer("'\\n'");
        assert_tokens(
            collect_tokens(&mut lexer),
            vec![Token::Literal(Literal::Character('\n')), Token::Eof],
        );
    }

    #[test]
    fn test_char_literal_escape_t() {
        let mut lexer = make_lexer("'\\t'");
        assert_tokens(
            collect_tokens(&mut lexer),
            vec![Token::Literal(Literal::Character('\t')), Token::Eof],
        );
    }

    #[test]
    fn test_char_literal_escape_r() {
        let mut lexer = make_lexer("'\\r'");
        assert_tokens(
            collect_tokens(&mut lexer),
            vec![Token::Literal(Literal::Character('\r')), Token::Eof],
        );
    }

    #[test]
    fn test_char_literal_escape_backslash() {
        let mut lexer = make_lexer("'\\\\'");
        assert_tokens(
            collect_tokens(&mut lexer),
            vec![Token::Literal(Literal::Character('\\')), Token::Eof],
        );
    }

    #[test]
    fn test_char_literal_escape_quote() {
        let mut lexer = make_lexer("'\\''");
        assert_tokens(
            collect_tokens(&mut lexer),
            vec![Token::Literal(Literal::Character('\'')), Token::Eof],
        );
    }

    #[test]
    fn test_char_literal_escape_null() {
        let mut lexer = make_lexer("'\\0'");
        assert_tokens(
            collect_tokens(&mut lexer),
            vec![Token::Literal(Literal::Character('\0')), Token::Eof],
        );
    }

    // ------------------------- String Literals ----------------------------

    #[test]
    fn test_string_literal_empty() {
        let mut lexer = make_lexer("\"\"");
        assert_tokens(collect_tokens(&mut lexer), vec![str_lit(""), Token::Eof]);
    }

    #[test]
    fn test_string_literal_simple() {
        let mut lexer = make_lexer("\"hello\"");
        assert_tokens(
            collect_tokens(&mut lexer),
            vec![str_lit("hello"), Token::Eof],
        );
    }

    #[test]
    fn test_string_literal_with_escapes() {
        let mut lexer = make_lexer("\"a\\nb\\tc\"");
        assert_tokens(
            collect_tokens(&mut lexer),
            vec![str_lit("a\nb\tc"), Token::Eof],
        );
    }

    #[test]
    fn test_string_literal_escaped_quote() {
        let mut lexer = make_lexer("\"say \\\"hi\\\"\"");
        assert_tokens(
            collect_tokens(&mut lexer),
            vec![str_lit("say \"hi\""), Token::Eof],
        );
    }

    // ------------------------- Symbol ----------------------------

    #[test]
    fn test_symbol_braces() {
        let mut lexer = make_lexer("{}");
        assert_tokens(
            collect_tokens(&mut lexer),
            vec![Token::LBrace, Token::RBrace, Token::Eof],
        );
    }

    #[test]
    fn test_symbol_parens() {
        let mut lexer = make_lexer("()");
        assert_tokens(
            collect_tokens(&mut lexer),
            vec![Token::LParen, Token::RParen, Token::Eof],
        );
    }

    #[test]
    fn test_symbol_colon() {
        let mut lexer = make_lexer(":");
        assert_tokens(collect_tokens(&mut lexer), vec![Token::Colon, Token::Eof]);
    }

    #[test]
    fn test_symbol_double_colon() {
        let mut lexer = make_lexer("::");
        assert_tokens(
            collect_tokens(&mut lexer),
            vec![Token::DoubleColon, Token::Eof],
        );
    }

    #[test]
    fn test_symbol_semicolon() {
        let mut lexer = make_lexer(";");
        assert_tokens(
            collect_tokens(&mut lexer),
            vec![Token::SemiColon, Token::Eof],
        );
    }

    #[test]
    fn test_symbol_comma() {
        let mut lexer = make_lexer(",");
        assert_tokens(collect_tokens(&mut lexer), vec![Token::Comma, Token::Eof]);
    }

    #[test]
    fn test_symbol_assign() {
        let mut lexer = make_lexer("=");
        assert_tokens(collect_tokens(&mut lexer), vec![Token::Assign, Token::Eof]);
    }

    #[test]
    fn test_symbol_arm_arrow() {
        let mut lexer = make_lexer("=>");
        assert_tokens(
            collect_tokens(&mut lexer),
            vec![Token::ArmArrow, Token::Eof],
        );
    }

    #[test]
    fn test_symbol_type_arrow() {
        let mut lexer = make_lexer("->");
        assert_tokens(
            collect_tokens(&mut lexer),
            vec![Token::TypeArrow, Token::Eof],
        );
    }

    #[test]
    fn test_symbol_ampersand() {
        let mut lexer = make_lexer("&");
        assert_tokens(
            collect_tokens(&mut lexer),
            vec![Token::Ampersand, Token::Eof],
        );
    }

    // ------------------------- Operators ----------------------------

    #[test]
    fn test_operator_plus() {
        let mut lexer = make_lexer("+");
        assert_tokens(collect_tokens(&mut lexer), vec![Token::Plus, Token::Eof]);
    }

    #[test]
    fn test_operator_minus() {
        let mut lexer = make_lexer("-");
        assert_tokens(collect_tokens(&mut lexer), vec![Token::Minus, Token::Eof]);
    }

    #[test]
    fn test_operator_star() {
        let mut lexer = make_lexer("*");
        assert_tokens(collect_tokens(&mut lexer), vec![Token::Star, Token::Eof]);
    }

    #[test]
    fn test_operator_slash() {
        let mut lexer = make_lexer("/");
        assert_tokens(collect_tokens(&mut lexer), vec![Token::Slash, Token::Eof]);
    }

    #[test]
    fn test_operator_remainder() {
        let mut lexer = make_lexer("%");
        assert_tokens(
            collect_tokens(&mut lexer),
            vec![Token::Remainder, Token::Eof],
        );
    }

    // ------------------------- Comparisons ----------------------------

    #[test]
    fn test_comparison_eq() {
        let mut lexer = make_lexer("==");
        assert_tokens(
            collect_tokens(&mut lexer),
            vec![Token::Comparison(Comparison::Eq), Token::Eof],
        );
    }

    #[test]
    fn test_comparison_neq() {
        let mut lexer = make_lexer("!=");
        assert_tokens(
            collect_tokens(&mut lexer),
            vec![Token::Comparison(Comparison::Neq), Token::Eof],
        );
    }

    #[test]
    fn test_comparison_gt() {
        let mut lexer = make_lexer(">");
        assert_tokens(
            collect_tokens(&mut lexer),
            vec![Token::Comparison(Comparison::Gt), Token::Eof],
        );
    }

    #[test]
    fn test_comparison_lt() {
        let mut lexer = make_lexer("<");
        assert_tokens(
            collect_tokens(&mut lexer),
            vec![Token::Comparison(Comparison::Lt), Token::Eof],
        );
    }

    #[test]
    fn test_comparison_geq() {
        let mut lexer = make_lexer(">=");
        assert_tokens(
            collect_tokens(&mut lexer),
            vec![Token::Comparison(Comparison::Geq), Token::Eof],
        );
    }

    #[test]
    fn test_comparison_leq() {
        let mut lexer = make_lexer("<=");
        assert_tokens(
            collect_tokens(&mut lexer),
            vec![Token::Comparison(Comparison::Leq), Token::Eof],
        );
    }

    // ------------------------- Whitespace and Comments ----------------------------

    #[test]
    fn test_skip_whitespace() {
        let mut lexer = make_lexer("   fn");
        assert_tokens(collect_tokens(&mut lexer), vec![Token::Fn, Token::Eof]);
    }

    #[test]
    fn test_skip_tabs_and_newlines() {
        let mut lexer = make_lexer("\t\n\r  fn");
        assert_tokens(collect_tokens(&mut lexer), vec![Token::Fn, Token::Eof]);
    }

    #[test]
    fn test_skip_line_comment() {
        let mut lexer = make_lexer("// comment\nfn");
        assert_tokens(collect_tokens(&mut lexer), vec![Token::Fn, Token::Eof]);
    }

    #[test]
    fn test_skip_multiple_comments() {
        let mut lexer = make_lexer("// first\n// second\nfn");
        assert_tokens(collect_tokens(&mut lexer), vec![Token::Fn, Token::Eof]);
    }

    #[test]
    fn test_comment_at_eof() {
        let mut lexer = make_lexer("// comment");
        assert_tokens(collect_tokens(&mut lexer), vec![Token::Eof]);
    }

    // ------------------------- Complex Sequences ----------------------------

    #[test]
    fn test_function_signature() {
        let mut lexer = make_lexer("fn foo(x: usize) -> u8");
        assert_tokens(
            collect_tokens(&mut lexer),
            vec![
                Token::Fn,
                ident("foo"),
                Token::LParen,
                ident("x"),
                Token::Colon,
                Token::Type(Type::Usize),
                Token::RParen,
                Token::TypeArrow,
                Token::Type(Type::U8),
                Token::Eof,
            ],
        );
    }

    #[test]
    fn test_let_statement() {
        let mut lexer = make_lexer("let x: usize = 42;");
        assert_tokens(
            collect_tokens(&mut lexer),
            vec![
                Token::Let,
                ident("x"),
                Token::Colon,
                Token::Type(Type::Usize),
                Token::Assign,
                Token::Literal(Literal::Integer(42)),
                Token::SemiColon,
                Token::Eof,
            ],
        );
    }

    #[test]
    fn test_match_arm() {
        let mut lexer = make_lexer("match x { 1 => 2, }");
        assert_tokens(
            collect_tokens(&mut lexer),
            vec![
                Token::Match,
                ident("x"),
                Token::LBrace,
                Token::Literal(Literal::Integer(1)),
                Token::ArmArrow,
                Token::Literal(Literal::Integer(2)),
                Token::Comma,
                Token::RBrace,
                Token::Eof,
            ],
        );
    }

    #[test]
    fn test_comparison_expression() {
        let mut lexer = make_lexer("a == b != c < d > e <= f >= g");
        assert_tokens(
            collect_tokens(&mut lexer),
            vec![
                ident("a"),
                Token::Comparison(Comparison::Eq),
                ident("b"),
                Token::Comparison(Comparison::Neq),
                ident("c"),
                Token::Comparison(Comparison::Lt),
                ident("d"),
                Token::Comparison(Comparison::Gt),
                ident("e"),
                Token::Comparison(Comparison::Leq),
                ident("f"),
                Token::Comparison(Comparison::Geq),
                ident("g"),
                Token::Eof,
            ],
        );
    }

    #[test]
    fn test_enum_definition() {
        let mut lexer = make_lexer("enum Foo { A, B(usize) }");
        assert_tokens(
            collect_tokens(&mut lexer),
            vec![
                Token::Enum,
                ident("Foo"),
                Token::LBrace,
                ident("A"),
                Token::Comma,
                ident("B"),
                Token::LParen,
                Token::Type(Type::Usize),
                Token::RParen,
                Token::RBrace,
                Token::Eof,
            ],
        );
    }

    #[test]
    fn test_path_with_double_colon() {
        let mut lexer = make_lexer("Foo::Bar");
        assert_tokens(
            collect_tokens(&mut lexer),
            vec![ident("Foo"), Token::DoubleColon, ident("Bar"), Token::Eof],
        );
    }

    #[test]
    fn test_reference_and_mut() {
        let mut lexer = make_lexer("&mut x");
        assert_tokens(
            collect_tokens(&mut lexer),
            vec![Token::Ampersand, Token::Mut, ident("x"), Token::Eof],
        );
    }

    // ------------------------- Full Programs ----------------------------

    #[test]
    fn test_full_program_hello_world() {
        let program = r#"
fn main() {
    let msg: &str = "Hello, World!";
}
"#;
        let mut lexer = make_lexer(program);
        assert_tokens(
            collect_tokens(&mut lexer),
            vec![
                Token::Fn,
                ident("main"),
                Token::LParen,
                Token::RParen,
                Token::LBrace,
                Token::Let,
                ident("msg"),
                Token::Colon,
                Token::Ampersand,
                Token::Type(Type::Str),
                Token::Assign,
                str_lit("Hello, World!"),
                Token::SemiColon,
                Token::RBrace,
                Token::Eof,
            ],
        );
    }

    #[test]
    fn test_full_program_enum_and_match() {
        let program = r#"
enum Option {
    Some(usize),
    None,
}

fn unwrap(opt: Option) -> usize {
    match opt {
        Option::Some(x) => x,
        Option::None => 0,
    }
}
"#;
        let mut lexer = make_lexer(program);
        assert_tokens(
            collect_tokens(&mut lexer),
            vec![
                // enum Option { Some(usize), None, }
                Token::Enum,
                ident("Option"),
                Token::LBrace,
                ident("Some"),
                Token::LParen,
                Token::Type(Type::Usize),
                Token::RParen,
                Token::Comma,
                ident("None"),
                Token::Comma,
                Token::RBrace,
                // fn unwrap(opt: Option) -> usize {
                Token::Fn,
                ident("unwrap"),
                Token::LParen,
                ident("opt"),
                Token::Colon,
                ident("Option"),
                Token::RParen,
                Token::TypeArrow,
                Token::Type(Type::Usize),
                Token::LBrace,
                // match opt {
                Token::Match,
                ident("opt"),
                Token::LBrace,
                // Option::Some(x) => x,
                ident("Option"),
                Token::DoubleColon,
                ident("Some"),
                Token::LParen,
                ident("x"),
                Token::RParen,
                Token::ArmArrow,
                ident("x"),
                Token::Comma,
                // Option::None => 0,
                ident("Option"),
                Token::DoubleColon,
                ident("None"),
                Token::ArmArrow,
                Token::Literal(Literal::Integer(0)),
                Token::Comma,
                // closing braces
                Token::RBrace,
                Token::RBrace,
                Token::Eof,
            ],
        );
    }

    #[test]
    fn test_full_program_while_loop() {
        let program = r#"
fn factorial(n: usize) -> usize {
    let mut result: usize = 1;
    let mut i: usize = 1;
    while i <= n {
        result = result * i;
        i = i + 1;
    }
    return result;
}
"#;
        let mut lexer = make_lexer(program);
        assert_tokens(
            collect_tokens(&mut lexer),
            vec![
                // fn factorial(n: usize) -> usize {
                Token::Fn,
                ident("factorial"),
                Token::LParen,
                ident("n"),
                Token::Colon,
                Token::Type(Type::Usize),
                Token::RParen,
                Token::TypeArrow,
                Token::Type(Type::Usize),
                Token::LBrace,
                // let mut result: usize = 1;
                Token::Let,
                Token::Mut,
                ident("result"),
                Token::Colon,
                Token::Type(Type::Usize),
                Token::Assign,
                Token::Literal(Literal::Integer(1)),
                Token::SemiColon,
                // let mut i: usize = 1;
                Token::Let,
                Token::Mut,
                ident("i"),
                Token::Colon,
                Token::Type(Type::Usize),
                Token::Assign,
                Token::Literal(Literal::Integer(1)),
                Token::SemiColon,
                // while i <= n {
                Token::While,
                ident("i"),
                Token::Comparison(Comparison::Leq),
                ident("n"),
                Token::LBrace,
                // result = result * i;
                ident("result"),
                Token::Assign,
                ident("result"),
                Token::Star,
                ident("i"),
                Token::SemiColon,
                // i = i + 1;
                ident("i"),
                Token::Assign,
                ident("i"),
                Token::Plus,
                Token::Literal(Literal::Integer(1)),
                Token::SemiColon,
                // }
                Token::RBrace,
                // return result;
                Token::Return,
                ident("result"),
                Token::SemiColon,
                Token::RBrace,
                Token::Eof,
            ],
        );
    }

    #[test]
    fn test_full_program_if_else() {
        let program = r#"
fn max(a: usize, b: usize) -> usize {
    if a > b {
        return a;
    } else {
        return b;
    }
}
"#;
        let mut lexer = make_lexer(program);
        assert_tokens(
            collect_tokens(&mut lexer),
            vec![
                // fn max(a: usize, b: usize) -> usize {
                Token::Fn,
                ident("max"),
                Token::LParen,
                ident("a"),
                Token::Colon,
                Token::Type(Type::Usize),
                Token::Comma,
                ident("b"),
                Token::Colon,
                Token::Type(Type::Usize),
                Token::RParen,
                Token::TypeArrow,
                Token::Type(Type::Usize),
                Token::LBrace,
                // if a > b {
                Token::If,
                ident("a"),
                Token::Comparison(Comparison::Gt),
                ident("b"),
                Token::LBrace,
                // return a;
                Token::Return,
                ident("a"),
                Token::SemiColon,
                Token::RBrace,
                // else {
                Token::Else,
                Token::LBrace,
                // return b;
                Token::Return,
                ident("b"),
                Token::SemiColon,
                Token::RBrace,
                Token::RBrace,
                Token::Eof,
            ],
        );
    }

    #[test]
    fn test_full_program_pointer_arithmetic() {
        let program = r#"
fn write_byte(ptr: *mut u8, offset: usize, value: u8) {
    let target: *mut u8 = ptr as usize + offset as *mut u8;
    *target = value;
}
"#;
        let mut lexer = make_lexer(program);
        assert_tokens(
            collect_tokens(&mut lexer),
            vec![
                // fn write_byte(ptr: *mut u8, offset: usize, value: u8) {
                Token::Fn,
                ident("write_byte"),
                Token::LParen,
                ident("ptr"),
                Token::Colon,
                Token::Star,
                Token::Mut,
                Token::Type(Type::U8),
                Token::Comma,
                ident("offset"),
                Token::Colon,
                Token::Type(Type::Usize),
                Token::Comma,
                ident("value"),
                Token::Colon,
                Token::Type(Type::U8),
                Token::RParen,
                Token::LBrace,
                // let target: *mut u8 = ptr as usize + offset as *mut u8;
                Token::Let,
                ident("target"),
                Token::Colon,
                Token::Star,
                Token::Mut,
                Token::Type(Type::U8),
                Token::Assign,
                ident("ptr"),
                Token::As,
                Token::Type(Type::Usize),
                Token::Plus,
                ident("offset"),
                Token::As,
                Token::Star,
                Token::Mut,
                Token::Type(Type::U8),
                Token::SemiColon,
                // *target = value;
                Token::Star,
                ident("target"),
                Token::Assign,
                ident("value"),
                Token::SemiColon,
                Token::RBrace,
                Token::Eof,
            ],
        );
    }

    #[test]
    fn test_full_program_with_comments() {
        let program = r#"
// This is a comment
fn add(a: usize, b: usize) -> usize {
    // Add two numbers
    return a + b; // return sum
}
"#;
        let mut lexer = make_lexer(program);
        assert_tokens(
            collect_tokens(&mut lexer),
            vec![
                Token::Fn,
                ident("add"),
                Token::LParen,
                ident("a"),
                Token::Colon,
                Token::Type(Type::Usize),
                Token::Comma,
                ident("b"),
                Token::Colon,
                Token::Type(Type::Usize),
                Token::RParen,
                Token::TypeArrow,
                Token::Type(Type::Usize),
                Token::LBrace,
                Token::Return,
                ident("a"),
                Token::Plus,
                ident("b"),
                Token::SemiColon,
                Token::RBrace,
                Token::Eof,
            ],
        );
    }
}
