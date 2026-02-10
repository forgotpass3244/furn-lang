use std::iter::Peekable;

use crate::{lexer::tokens::{Token, Tokens, TokensIterator}, parser::ast::{Expr, Stmt}, tok::token_other::TokenOther};


pub struct Parser<'a> {
    tok: Peekable<TokensIterator<'a, TokenOther>>,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: &'a Tokens<TokenOther>) -> Self {
        Self {
            tok: tokens.iter().peekable(),
        }
    }

    fn emit_error(&self, message: &str) {
        println!("{message}");
    }

    #[warn(unused_results)]
    fn is_token(&mut self, expected: TokenOther) -> bool {
        match self.tok.peek() {
            Some(token) => {
                match token {
                    Token::Other(token) => *token == expected,
                    _ => false,
                }
            }

            None => false,
        }
    }

    #[warn(unused_results)]
    #[allow(dead_code)]
    fn is_terminator(&mut self) -> bool {
        self.is_token(TokenOther::Semicolon)
    }

    #[warn(unused_results)]
    fn match_token(&mut self, expected: TokenOther) -> Option<&TokenOther> {
        if self.is_token(expected) {
            let token = self.tok.next().unwrap();
            match token {
                Token::Other(token) => Some(token),
                _ => unreachable!(),
            }
        } else {
            None
        }
    }

    #[warn(unused_results)]
    fn match_terminator(&mut self) -> Option<&TokenOther> {
        self.match_token(TokenOther::Semicolon)
    }

    fn expect_token(&mut self, expected: TokenOther) -> Result<(), ()> {
        if self.match_token(expected).is_some() {
            Ok(())
        } else {
            self.emit_error("wrong token lol");
            Err(())
        }
    }

    fn expect_terminator(&mut self) -> Result<(), ()> {
        if self.match_terminator().is_some() {
            Ok(())
        } else {
            self.emit_error("expected terminator: `;`");
            Err(())
        }
    }

    fn parse_name(&mut self) -> Result<String, ()> {
        match self.tok.peek() {
            Some(token) => match token {
                Token::Ident(name) => {
                    self.tok.next();
                    Ok(name.clone())
                },
                _ => Err(()),
            },

            None => Err(()),
        }
    }

    pub fn parse(&mut self) -> Result<Vec<Stmt>, ()> {
        let mut ast = Vec::new();
        while self.tok.peek().is_some() {
            let stmt = self.parse_stmt()?;
            ast.push(stmt);
        }
        Ok(ast)
    }
}

impl Parser<'_> {
    /// Returns a result: Stmt
    pub fn parse_stmt(&mut self) -> Result<Stmt, ()> {
        if self.match_token(TokenOther::Let).is_some() {
            self.parse_const_decl(false)
        } else if self.match_token(TokenOther::Var).is_some() {
            self.parse_var_decl(false)
        } else if self.match_token(TokenOther::Public).is_some() {
            if self.match_token(TokenOther::Var).is_some() {
                self.parse_var_decl(true)
            } else {
                self.parse_const_decl(true)
            }
        } else if self.match_token(TokenOther::Package).is_some() {
            let name = self.parse_name()?;
            self.expect_terminator()?;
            Ok(Stmt::PackageDecl(name))
        } else {

            let expr = self.parse_expr()?;

            let is_final_expr = if self.is_token(TokenOther::CBrace) {
                true
            } else if expr.is_block() {
                self.match_terminator();
                false
            } else {
                self.expect_terminator()?;
                false
            };

            Ok(Stmt::Expr(expr, is_final_expr))
        }
    }

    fn parse_const_decl(&mut self, is_exported: bool) -> Result<Stmt, ()> {
        let name = self.parse_name()?;

        if self.match_terminator().is_some() {
            Ok(Stmt::ConstDecl(name, None, is_exported))
        } else if self.match_token(TokenOther::Equal).is_some() {
            let expr = self.parse_expr()?;
            if expr.is_block() {
                self.match_terminator();
            } else {
                self.expect_terminator()?;
            }
            Ok(Stmt::ConstDecl(name, Some(expr), is_exported))
        } else {
            self.emit_error("expected one of `;`, `:` or `=`");
            Err(())
        }
    }
    
    fn parse_var_decl(&mut self, is_exported: bool) -> Result<Stmt, ()>{
        let name = self.parse_name()?;

        if self.match_terminator().is_some() {
            Ok(Stmt::VarDecl(name, None, is_exported))
        } else if self.match_token(TokenOther::Equal).is_some() {
            let expr = self.parse_expr()?;
            if expr.is_block() {
                self.match_terminator();
            } else {
                self.expect_terminator()?;
            }
            Ok(Stmt::VarDecl(name, Some(expr), is_exported))
        } else {
            self.emit_error("expected one of `;`, `:` or `=`");
            Err(())
        }
    }
}

impl Parser<'_> {
    fn parse_expr(&mut self) -> Result<Expr, ()> {
        if self.match_token(TokenOther::OParen).is_some() {
            self.expect_token(TokenOther::CParen)?;
            let expr = self.parse_expr()?;
            Ok(Expr::Function(Box::new(expr)))
        } else if self.match_token(TokenOther::OBrace).is_some() {

            let mut body = Vec::new();
            let mut return_expr = None;

            loop {

                let stmt = self.parse_stmt()?;

                if self.match_token(TokenOther::CBrace).is_some() {

                    match stmt {
                        Stmt::Expr(expr, true) => {
                            return_expr = Some(Box::new(expr));
                        },
                        _ => body.push(stmt),
                    }

                    break
                } else {
                    body.push(stmt);
                }
            }

            Ok(Expr::Block(body, return_expr))
        } else if let Some(token) = self.tok.peek() {
            match token {
                Token::IntLiteral(int) => {
                    self.tok.next();
                    Ok(Expr::IntLit(*int))
                }
                Token::Ident(name) => {
                    self.tok.next();
                    Ok(Expr::Variable(name.clone()))
                }
                _ => Err(()),
            }
        } else {
            Err(())
        }
    }
}

