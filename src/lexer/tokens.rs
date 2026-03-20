use core::fmt;
use std::ops::Index;


#[derive(PartialEq, Clone)]
pub enum TokenEnum<T> {
    Ident(String),
    StringLiteral(String),
    CharLiteral(String),
    IntLiteral(u64),
    FloatLiteral(f64),
    Other(T),
}

impl<T> TokenEnum<T>
where T: Clone {
    pub fn from_other(other: T) -> Self {
        TokenEnum::Other(other)
    }

    pub fn to_tok(self, loc: SourceLocation) -> Token<T> {
        Token {
            t_enum: self,
            loc,
        }
    }
}

pub struct Token<T> {
    t_enum: TokenEnum<T>,
    loc: SourceLocation,
}

impl<T> Token<T> {
    pub fn as_enum(&self) -> &TokenEnum<T> {
        &self.t_enum
    }

    pub fn get_loc(&self) -> SourceLocation {
        self.loc
    }
}

impl<T> fmt::Display for Token<T>
where T: fmt::Display {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.as_enum() {
            TokenEnum::Ident(string) => write!(f, "`{string}`"),
            TokenEnum::StringLiteral(string) => write!(f, "\"{string}\""),
            TokenEnum::CharLiteral(string) => write!(f, "'{string}'"),
            TokenEnum::IntLiteral(int) => write!(f, "`{int}`"),
            TokenEnum::FloatLiteral(float) => write!(f, "`{float}`"),
            TokenEnum::Other(x) => write!(f, "'{x}'"),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct SourceLocation {
    pub line: isize,
    pub col: isize,
}

impl SourceLocation {
    pub fn new(line: isize, col: isize) -> Self {
        Self {
            line,
            col,
        }
    }

    pub fn garbage() -> Self {
        Self::new(1, 1)
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

#[derive(Clone)]
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
