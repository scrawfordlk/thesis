use crate::token::Token;

// TODO: You may not use derive macros other than Debug (for development)

// The purpose of this file is to wrap standard library features that are not
// directly supported by the subset (e.g. Vec<T> due to generics).
// String is now considered supported natively, so Text wrapper was removed.

#[derive(Clone, Debug, PartialEq)]
pub enum OptionToken {
    Some(Token),
    None,
}

#[derive(Clone, Debug, PartialEq)]
pub enum OptionChar {
    Some(char),
    None,
}

#[derive(Clone, Debug)]
pub struct TokenList {
    inner: Vec<Token>,
}

impl TokenList {
    pub fn new() -> Self {
        TokenList { inner: Vec::new() }
    }

    pub fn push(&mut self, token: Token) {
        self.inner.push(token);
    }

    pub fn get(&self, index: usize) -> OptionToken {
        match self.inner.get(index) {
            Some(t) => OptionToken::Some(t.clone()),
            None => OptionToken::None,
        }
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }
}

pub struct CharList {
    inner: Vec<char>,
}

impl CharList {
    pub fn from_string(s: String) -> Self {
        CharList {
            inner: s.chars().collect(),
        }
    }

    pub fn get(&self, index: usize) -> OptionChar {
        match self.inner.get(index) {
            Some(c) => OptionChar::Some(*c),
            None => OptionChar::None,
        }
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }
}
