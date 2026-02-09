use core::fmt;
use std::ops::Index;


#[derive(PartialEq)]
pub enum Token<T> {
    Ident(String),
    StringLiteral(String),
    CharLiteral(String),
    IntLiteral(u64),
    FloatLiteral(f64),
    Other(T),
}

impl<T> Token<T> {
    pub fn from_other(other: T) -> Self {
        Self::Other(other)
    }
}

impl<T> fmt::Display for Token<T>
where T: fmt::Display {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::Ident(string) => write!(f, "{string}"),
            Token::StringLiteral(string) => write!(f, "\"{string}\""),
            Token::CharLiteral(string) => write!(f, "'{string}'"),
            Token::IntLiteral(int) => write!(f, "{int}i"),
            Token::FloatLiteral(float) => write!(f, "{float}f"),
            Token::Other(x) => write!(f, "{x}"),
        }
    }
}

pub struct Tokens<T> {
    vec: Vec<Token<T>>,
}

impl<T> Tokens<T> {
    pub fn new() -> Self {
        Self {
            vec: Vec::new(),
        }
    }

    pub fn iter(&self) -> TokensIterator<'_, T> {
        TokensIterator {
            tokens: self,
            index: 0,
        }
    }

    pub fn push(&mut self, token: Token<T>) {
        self.vec.push(token);
    }
}

impl<T> Index<usize> for Tokens<T> {
    type Output = Token<T>;

    fn index(&self, index: usize) -> &Self::Output {
        &self.vec[index]
    }
}

impl<T> fmt::Display for Tokens<T>
where T: fmt::Display {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Tokens [ ").unwrap();
        for token in self.iter() {
            write!(f, "{token} ").unwrap();
        }
        write!(f, "]")
    }
}

pub struct TokensIterator<'a, T> {
    tokens: &'a Tokens<T>,
    index: usize,
}

impl<'a, T> Iterator for TokensIterator<'a, T> {
    type Item = &'a Token<T>;
    
    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.tokens.vec.len() {
            let result = &self.tokens[self.index];
            self.index += 1;
            Some(result)
        } else {
            None
        }
    }
}
