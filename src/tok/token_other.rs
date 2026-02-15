use core::fmt;

use crate::lexer::token_map::TokenMap;

#[derive(Clone, PartialEq, Eq)]
pub enum TokenOther {
    // keywords
    Let,
    Var,
    Public,
    Package,
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
}

impl TokenOther {
    pub fn make_token_map() -> TokenMap<Self> {
        let mut token_map = TokenMap::new();

        token_map.make_keyword("let", TokenOther::Let);
        token_map.make_keyword("var", TokenOther::Var);
        token_map.make_keyword("public", TokenOther::Public);
        token_map.make_keyword("package", TokenOther::Package);
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

        token_map
    }
}

impl fmt::Display for TokenOther {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TokenOther::Let => write!(f, "kw:let"),
            TokenOther::Var => write!(f, "kw:var"),
            TokenOther::Public => write!(f, "kw:public"),
            TokenOther::Package => write!(f, "kw:package"),
            TokenOther::TypeUInt64 => write!(f, "kw:u64"),
            TokenOther::TypeString => write!(f, "kw:str"),

            TokenOther::OParen => write!(f, "'('"),
            TokenOther::CParen => write!(f, "')'"),
            TokenOther::OBrace => write!(f, "'{{'"),
            TokenOther::CBrace => write!(f, "'}}'"),
            TokenOther::Semicolon => write!(f, "';'"),
            TokenOther::Equal => write!(f, "'='"),
            TokenOther::Colon => write!(f, "':'"),
            TokenOther::ColonColon => write!(f, "'::'"),
            TokenOther::Dot => write!(f, "'.'"),
            TokenOther::Comma => write!(f, "','"),
        }
    }
}



