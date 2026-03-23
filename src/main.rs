#![allow(warnings)]
mod runtime;
mod lexer;
mod token;

#[cfg(test)]
mod lexer_tests;

use lexer::lex;

use runtime::OptionToken;

fn main() {
    // Basic test of lexer
    let code = String::from("fn main() { let x = 42; }");
    println!("Lexing: {}", code);
    
    let tokens = lex(code);
    
    println!("Tokens:");
    let mut i = 0;
    while i < tokens.len() {
        if let OptionToken::Some(token) = tokens.get(i) {
            println!("{:?}", token);
        }
        i = i + 1;
    }
}
