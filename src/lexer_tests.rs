#[cfg(test)]
mod tests {
    use crate::lexer::lex;
    use crate::token::Token;

use crate::runtime::OptionToken;

    fn assert_tokens(source: &str, expected: &[Token]) {
        let text = String::from(source);
        let tokens = lex(text);
        
        let mut i = 0;
        // Verify all expected tokens match
        while i < expected.len() {
            let actual_opt = tokens.get(i);
            
            match actual_opt {
                OptionToken::Some(actual) => {
                    let want = &expected[i];
                    assert_eq!(&actual, want, "Token mismatch at index {}", i);
                },
                OptionToken::None => {
                    panic!("Not enough tokens generated");
                }
            }
            i = i + 1;
        }
        
        // Verify we hit EOF right after
        let last_opt = tokens.get(i);
        match last_opt {
            OptionToken::Some(last) => {
                assert_eq!(last, Token::EOF, "Expected EOF at end");
                assert_eq!(tokens.len(), expected.len() + 1, "Token count mismatch (including EOF)");
            },
            OptionToken::None => {
                panic!("Missing EOF token");
            }
        }
    }

    #[test]
    fn test_empty() {
        assert_tokens("", &[]);
        assert_tokens("   \n\t  ", &[]);
    }

    #[test]
    fn test_single_tokens() {
        assert_tokens("fn", &[Token::Fn]);
        assert_tokens("let", &[Token::Let]);
        assert_tokens("if", &[Token::If]);
        assert_tokens("while", &[Token::While]);
        assert_tokens("return", &[Token::Return]);
        assert_tokens("enum", &[Token::Enum]);
        assert_tokens("match", &[Token::Match]);
        
        assert_tokens("{", &[Token::LBrace]);
        assert_tokens("}", &[Token::RBrace]);
        assert_tokens("(", &[Token::LParen]);
        assert_tokens(")", &[Token::RParen]);
        assert_tokens(":", &[Token::Colon]);
        assert_tokens(";", &[Token::SemiColon]);
        assert_tokens(",", &[Token::Comma]);
        assert_tokens("+", &[Token::Plus]);
        assert_tokens("-", &[Token::Minus]);
        assert_tokens("*", &[Token::Star]);
        assert_tokens("/", &[Token::Slash]);
        assert_tokens("=", &[Token::Assign]);
        assert_tokens("==", &[Token::Eq]);
        assert_tokens("->", &[Token::Arrow]);
    }

    #[test]
    fn test_identifiers_and_literals() {
        assert_tokens("foo", &[Token::Identifier("foo".to_string())]);
        assert_tokens("bar_baz", &[Token::Identifier("bar_baz".to_string())]);
        assert_tokens("_unused", &[Token::Identifier("_unused".to_string())]);
        
        assert_tokens("123", &[Token::Integer(123)]);
        assert_tokens("0", &[Token::Integer(0)]);
        
        assert_tokens("\"hello\"", &[Token::StringLiteral("hello".to_string())]);
        assert_tokens("\"\"", &[Token::StringLiteral("".to_string())]);
    }

    #[test]
    fn test_comments() {
        assert_tokens("// this is a comment", &[]);
        assert_tokens("let // comment\nx", &[
            Token::Let,
            Token::Identifier("x".to_string())
        ]);
        assert_tokens("// comment 1\n// comment 2\nfn", &[Token::Fn]);
    }

    #[test]
    fn test_complex_statement() {
        // let x: u64 = 42;
        assert_tokens("let x: u64 = 42;", &[
            Token::Let,
            Token::Identifier("x".to_string()),
            Token::Colon,
            Token::Identifier("u64".to_string()),
            Token::Assign,
            Token::Integer(42),
            Token::SemiColon,
        ]);
    }

    #[test]
    fn test_function_decl() {
        // fn add(a: u64, b: u64) -> u64 { a + b }
        assert_tokens("fn add(a: u64, b: u64) -> u64 { a + b }", &[
            Token::Fn,
            Token::Identifier("add".to_string()),
            Token::LParen,
            Token::Identifier("a".to_string()),
            Token::Colon,
            Token::Identifier("u64".to_string()),
            Token::Comma,
            Token::Identifier("b".to_string()),
            Token::Colon,
            Token::Identifier("u64".to_string()),
            Token::RParen,
            Token::Arrow,
            Token::Identifier("u64".to_string()),
            Token::LBrace,
            Token::Identifier("a".to_string()),
            Token::Plus,
            Token::Identifier("b".to_string()),
            Token::RBrace,
        ]);
    }

    #[test]
    fn test_multi_char_operators() {
        // -> vs -
        assert_tokens("-> -", &[Token::Arrow, Token::Minus]);
        // == vs =
        assert_tokens("== =", &[Token::Eq, Token::Assign]);
    }

    #[test]
    fn test_whole_program_transform() {
        let source = "
        fn main() {
            let x = 10;
            if x == 10 {
                return 1;
            }
        }
        ";
        
        let expected = &[
            Token::Fn, 
            Token::Identifier("main".to_string()), 
            Token::LParen, 
            Token::RParen, 
            Token::LBrace,
            
            Token::Let, 
            Token::Identifier("x".to_string()), 
            Token::Assign, 
            Token::Integer(10), 
            Token::SemiColon,
            
            Token::If, 
            Token::Identifier("x".to_string()), 
            Token::Eq, 
            Token::Integer(10), 
            Token::LBrace,
            
            Token::Return, 
            Token::Integer(1), 
            Token::SemiColon,
            
            Token::RBrace,
            Token::RBrace
        ];
        
        assert_tokens(source, expected);
    }
}
