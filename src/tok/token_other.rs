use core::fmt;

use crate::{lexer::token_map::TokenMap, parser::ast::Operator};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum TokenOther {
    // keywords
    Let,
    Var,
    Public,
    Package,
    Alias,
    As,
    If,
    Else,
    Do,
    End,
    Unsafe,
    TypeVoid,
    TypeUInt64,
    TypeString,

    // symbols
    OParen,
    CParen,
    OBrace,
    CBrace,
    Semicolon,
    Equal,
    Colon,
    ColonColon,
    Dot,
    Comma,
    Ampersand,
    Star,
    Plus,
    Minus,
    Pipe,
}

impl TokenOther {
    pub fn make_token_map() -> TokenMap<Self> {
        let mut token_map = TokenMap::new();

        token_map.make_keyword("let", TokenOther::Let);
        token_map.make_keyword("var", TokenOther::Var);
        token_map.make_keyword("public", TokenOther::Public);
        token_map.make_keyword("package", TokenOther::Package);
        token_map.make_keyword("alias", TokenOther::Alias);
        token_map.make_keyword("as", TokenOther::As);
        token_map.make_keyword("if", TokenOther::If);
        token_map.make_keyword("else", TokenOther::Else);
        token_map.make_keyword("do", TokenOther::Do);
        token_map.make_keyword("end", TokenOther::End);
        token_map.make_keyword("unsafe", TokenOther::Unsafe);
        token_map.make_keyword("void", TokenOther::TypeVoid);
        token_map.make_keyword("u64", TokenOther::TypeUInt64);
        token_map.make_keyword("str", TokenOther::TypeString);

        token_map.make("(", TokenOther::OParen);
        token_map.make(")", TokenOther::CParen);
        token_map.make("{", TokenOther::OBrace);
        token_map.make("}", TokenOther::CBrace);
        token_map.make(";", TokenOther::Semicolon);
        token_map.make("=", TokenOther::Equal);
        token_map.make(":", TokenOther::Colon);
        token_map.make("::", TokenOther::ColonColon);
        token_map.make(".", TokenOther::Dot);
        token_map.make(",", TokenOther::Comma);
        token_map.make("&", TokenOther::Ampersand);
        token_map.make("*", TokenOther::Star);
        token_map.make("+", TokenOther::Plus);
        token_map.make("-", TokenOther::Minus);
        token_map.make("|", TokenOther::Pipe);

        token_map
    }
}

impl TokenOther {
    pub fn to_operator(&self) -> Option<Operator> {
        match self {
            Self::Plus => Some(Operator::Add),
            Self::Minus => Some(Operator::Sub),
            Self::Pipe => Some(Operator::BitOr),
            Self::Equal => Some(Operator::Assign),
            _ => None,
        }
    }
}

impl fmt::Display for TokenOther {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TokenOther::Let => write!(f, "let"),
            TokenOther::Var => write!(f, "var"),
            TokenOther::Public => write!(f, "public"),
            TokenOther::Package => write!(f, "package"),
            TokenOther::Alias => write!(f, "alias"),
            TokenOther::As => write!(f, "as"),
            TokenOther::If => write!(f, "if"),
            TokenOther::Else => write!(f, "else"),
            TokenOther::Do => write!(f, "do"),
            TokenOther::End => write!(f, "end"),
            TokenOther::Unsafe => write!(f, "unsafe"),
            TokenOther::TypeVoid => write!(f, "void"),
            TokenOther::TypeUInt64 => write!(f, "u64"),
            TokenOther::TypeString => write!(f, "str"),

            TokenOther::OParen => write!(f, "("),
            TokenOther::CParen => write!(f, ")"),
            TokenOther::OBrace => write!(f, "{{"),
            TokenOther::CBrace => write!(f, "}}"),
            TokenOther::Semicolon => write!(f, ";"),
            TokenOther::Equal => write!(f, "="),
            TokenOther::Colon => write!(f, ":"),
            TokenOther::ColonColon => write!(f, "::"),
            TokenOther::Dot => write!(f, "."),
            TokenOther::Comma => write!(f, ","),
            TokenOther::Ampersand => write!(f, "&"),
            TokenOther::Star => write!(f, "*"),
            TokenOther::Plus => write!(f, "+"),
            TokenOther::Minus => write!(f, "-"),
            TokenOther::Pipe => write!(f, "|"),
        }
    }
}



