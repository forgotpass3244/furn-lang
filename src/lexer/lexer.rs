use std::fs;

use crate::lexer::{token_map::TokenMap, tokens::{SourceLocation, TokenEnum, Tokens}};


pub struct Lexer {
    input: Vec<char>,
    pos: usize,
    loc: SourceLocation,
}

impl Lexer {
    pub fn from_file(file_name: &str) -> Self {
        let file_data = fs::read_to_string(file_name).unwrap_or_else(|_| {
            println!("{file_name}");
            "public main = () { print (\"Hello, furn!\") }".to_string()
        }).chars().collect();
        Self {
            input: file_data,
            pos: 0,
            loc: SourceLocation::new(1, 1),
        }
    }

    pub fn tokenize<TokT: Clone>(&mut self, token_map: TokenMap<TokT>) -> Tokens<TokT> {
        let mut tokens = Tokens::new();
        while !self.is_eof() {
            let loc = self.loc;
            if self.is_space() {
                self.advance();
            } else if self.peek() == '"' {
                let text = self.lex_quoted();
                tokens.push(TokenEnum::StringLiteral(text).to_tok(loc));
            } else if self.peek() == '#' {
                self.advance();
                if self.peek() == '{' {
                    self.advance();
                    
                    let mut brace_nest_level = 0;
                    loop {
                        let ch = self.advance();
                        if ch == '{' {
                            brace_nest_level += 1;
                        } else if ch == '}' {
                            if brace_nest_level <= 0 {
                                break
                            } else {
                                brace_nest_level -= 1;
                            }
                        }
                    }
                } else {
                    while self.advance() != '\n' {}
                }
            } else if self.is_alpha() {
                let ident = self.lex_ident();
                let token_keyword = self.map_to_keyword(&token_map, &ident);

                if let Some(other) = token_keyword {
                    tokens.push(TokenEnum::from_other(other).to_tok(loc));
                } else {
                    tokens.push(TokenEnum::Ident(ident).to_tok(loc));
                }
            } else if self.is_digit() {
                let int = self.lex_int();
                tokens.push(TokenEnum::IntLiteral(int).to_tok(loc));
            } else {
                match self.map_to_token(&token_map) {
                    Some(token_other) => tokens.push(TokenEnum::from_other(token_other).to_tok(loc)),
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
            } else {
                key.pop();
                break
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
        if ch == '\n' {
            self.loc.line += 1;
            self.loc.col = 1;
        } else {
            self.loc.col += 1;
        }
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

    fn is_alphanum(&self) -> bool {
        if self.is_eof() { return false }
        self.is_alpha() || self.is_digit()
    }

    fn is_digit(&self) -> bool {
        !self.is_eof() && self.peek().is_ascii_digit()
    }

    fn lex_quoted(&mut self) -> String {
        let quote = self.advance();

        let mut text = String::new();
        while self.peek() != quote {
            let ch = self.advance();
            if ch == '\\' {

                let escaped_ch = match self.advance() {
                    '\\' => '\\',
                    'n' => '\n',
                    ch => ch,
                };

                text.push(escaped_ch);
            } else {
                text.push(ch);
            }
        }

        self.advance();
        text
    }

    fn lex_ident(&mut self) -> String {
        let mut text = String::new();
        while self.is_alphanum() {
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
