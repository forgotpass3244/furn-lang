use std::fs;

use crate::lexer::{token_map::TokenMap, tokens::{Token, Tokens}};


pub struct Lexer {
    input: Vec<char>,
    pos: usize,
}

impl Lexer {
    pub fn from_file(file_name: &str) -> Self {
        let file_data = fs::read_to_string(file_name).unwrap().chars().collect();
        Self {
            input: file_data,
            pos: 0,
        }
    }

    pub fn tokenize<TokT: Clone>(&mut self, token_map: TokenMap<TokT>) -> Tokens<TokT> {
        let mut tokens = Tokens::new();
        while !self.is_eof() {
            if self.is_space() {
                self.advance();
            } else if self.is_alpha() {
                let ident = self.lex_ident();
                let token_keyword = self.map_to_keyword(&token_map, &ident);

                if let Some(other) = token_keyword {
                    tokens.push(Token::from_other(other));
                } else {
                    tokens.push(Token::Ident(ident));
                }
            } else if self.is_digit() {
                let int = self.lex_int();
                tokens.push(Token::IntLiteral(int));
            } else {
                match self.map_to_token(&token_map) {
                    Some(token_other) => tokens.push(Token::from_other(token_other)),
                    None => {
                        if self.is_eof() {
                            panic!("Unexpected end of file at position {}", self.pos);
                        } else {
                            let ch = self.peek();
                            panic!("Unexpected character '{ch}' at position {}", self.pos);
                        }
                    }
                }
            }
        }
        
        tokens
    }

    fn map_to_keyword<TokT: Clone>(&mut self, token_map: &TokenMap<TokT>, ident: &String) -> Option<TokT> {
        token_map.get_keyword(ident).cloned()
    }

    fn map_to_token<TokT: Clone>(&mut self, token_map: &TokenMap<TokT>) -> Option<TokT> {
        if self.is_eof() { return None }

        let mut key = String::new();
        
        while !self.is_eof() {
            key.push(self.peek());

            if token_map.get(&key).is_some() {
                self.advance();
                break
            } else if token_map.any_key(|k| k.starts_with(&key)) {
                self.advance();
            } else {
                return None
            }
        };
        
        token_map.get(&key).cloned()
    }

    fn is_eof(&self) -> bool {
        self.pos >= self.input.len()
    }

    fn peek(&self) -> char {
        self.input[self.pos]
    }

    fn advance(&mut self) -> char {
        let ch = self.peek();
        self.pos += 1;
        ch
    }

    fn is_space(&self) -> bool {
        !self.is_eof() && self.peek().is_whitespace()
    }

    fn is_alpha(&self) -> bool {
        if self.is_eof() { return false }
        let ch = self.peek();
        ch.is_alphabetic() || ch == '_'
    }

    fn is_digit(&self) -> bool {
        !self.is_eof() && self.peek().is_ascii_digit()
    }

    fn lex_ident(&mut self) -> String {
        let mut text = String::new();
        while self.is_alpha() {
            let ch = self.advance();
            text.push(ch);
        }

        text
    }

    fn lex_int(&mut self) -> u64 {
        let mut text = String::new();
        while self.is_digit() {
            let ch = self.advance();
            text.push(ch);
        }

        text.parse().unwrap()
    }
}
