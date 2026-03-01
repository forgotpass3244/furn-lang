use std::iter::Peekable;

use crate::{lexer::tokens::{Token, Tokens, TokensIterator}, parser::ast::{Expr, Stmt}, tok::token_other::TokenOther};


pub struct Parser<'a> {
    tok: Peekable<TokensIterator<'a, TokenOther>>,
    err_count: usize,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: &'a Tokens<TokenOther>) -> Self {
        Self {
            tok: tokens.iter().peekable(),
            err_count: 0,
        }
    }
    
    pub fn has_errors(&self) -> bool {
        self.err_count > 0
    }

    fn emit_diagnostic(&mut self, message: &str) {
        self.err_count += 1;
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

    fn match_terminator(&mut self) -> Option<&TokenOther> {
        self.match_token(TokenOther::Semicolon)
    }

    fn expect_token(&mut self, expected: TokenOther) {
        if self.match_token(expected).is_none() {
            self.emit_diagnostic("wrong token lol");
        }
    }

    #[allow(dead_code)]
    fn expect_terminator(&mut self) {
        if self.match_terminator().is_none() {
            self.emit_diagnostic("expected terminator: `;`");
        }
    }

    fn unnecessary_terminator(&mut self) {
        if self.match_terminator().is_some() {
            self.emit_diagnostic("unnecessary terminator `;`, remove it");
        }
    }

    fn parse_name(&mut self) -> String {
        match self.tok.peek() {
            Some(token) => match token {
                Token::Ident(name) => {
                    self.tok.next();
                    name.clone()
                },
                
                _ => {
                    self.emit_diagnostic("expected name");
                    "(err)".to_string()
                },
            },

            None => {
                self.emit_diagnostic("expected name, but got EOF");
                "(eof)".to_string()
            },
        }
    }

    fn is_name(&mut self) -> bool {
        match self.tok.peek() {
            Some(token) => match token {
                Token::Ident(..) => true,
                _ => false,
            },

            None => false,
        }
    }

    pub fn parse(&mut self) -> Vec<Stmt> {
        let mut ast = Vec::new();
        while self.tok.peek().is_some() {
            let stmt = self.parse_stmt();
            if let Ok(stmt) = stmt {
                ast.push(stmt);
            }
        }
        
        ast
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
        } else if self.match_token(TokenOther::Alias).is_some() {

            let expr = self.parse_primary_expr2()?;
            let is_call = if self.match_token(TokenOther::OParen).is_some() {
                self.expect_token(TokenOther::CParen);
                true
            } else {
                false
            };

            if self.match_token(TokenOther::As).is_some() {
                let name = self.parse_name();
                self.expect_terminator();
                Ok(Stmt::AliasDecl(Some(name), expr, is_call))
            } else {
                self.expect_terminator();
                Ok(Stmt::AliasDecl(None, expr, is_call))
            }
        } else if self.match_token(TokenOther::Package).is_some() {
            let name = self.parse_name();
            self.expect_terminator();
            Ok(Stmt::PackageDecl(name))
        } else {

            let expr = self.parse_expr()?;

            let is_final_expr = if self.is_token(TokenOther::CBrace) {
                true
            } else if expr.is_block() {
                self.unnecessary_terminator();
                false
            } else {
                self.expect_terminator();
                false
            };

            Ok(Stmt::Expr(expr, is_final_expr))
        }
    }

    fn parse_const_decl(&mut self, is_exported: bool) -> Result<Stmt, ()> {
        let name = self.parse_name();

        let type_expr = if !self.is_terminator() && !self.is_token(TokenOther::Equal) {
            Some(self.parse_type_expr()?)
        } else {
            None
        };

        if self.match_terminator().is_some() {
            Ok(Stmt::ConstDecl(name, None, type_expr, is_exported))
        } else if self.match_token(TokenOther::Equal).is_some() {
            let expr = self.parse_expr()?;
            if expr.is_block() {
                self.unnecessary_terminator();
            } else {
                self.expect_terminator();
            }
            Ok(Stmt::ConstDecl(name, Some(expr), type_expr, is_exported))
        } else {
            self.emit_diagnostic("expected one of `;`, `:` or `=`");
            Ok(Stmt::ConstDecl(name, None, type_expr, is_exported))
        }
    }
    
    fn parse_var_decl(&mut self, is_exported: bool) -> Result<Stmt, ()> {
        let name = self.parse_name();

        let type_expr = if !self.is_terminator() && !self.is_token(TokenOther::Equal) {
            Some(self.parse_type_expr()?)
        } else {
            None
        };

        if self.match_terminator().is_some() {
            Ok(Stmt::VarDecl(name, None, type_expr, is_exported))
        } else if self.match_token(TokenOther::Equal).is_some() {
            let expr = self.parse_expr()?;
            if expr.is_block() {
                self.unnecessary_terminator();
            } else {
                self.expect_terminator();
            }
            Ok(Stmt::VarDecl(name, Some(expr), type_expr, is_exported))
        } else {
            self.emit_diagnostic("expected one of `;`, `:` or `=`");
            Ok(Stmt::VarDecl(name, None, type_expr, is_exported))
        }
    }
}

impl Parser<'_> {
    fn parse_type_expr(&mut self) -> Result<Expr, ()> {
        self.parse_primary_expr()
    }

    fn parse_expr(&mut self) -> Result<Expr, ()> {
        self.parse_primary_expr()
    }

    fn parse_primary_expr(&mut self) -> Result<Expr, ()> {
        let mut expr = self.parse_primary_expr2()?;
        loop {
            if self.match_token(TokenOther::OParen).is_some() {
                // parse args
                let mut args = Vec::new();
                loop {
                    if self.match_token(TokenOther::CParen).is_some() {
                        break
                    } else {
                        let arg = self.parse_expr()?;
                        args.push(arg);

                        if !self.is_token(TokenOther::CParen) {
                            self.expect_token(TokenOther::Comma);
                        }
                    } 
                }

                expr = Expr::Call(Box::new(expr), args);
            } else {
                break
            }
        }

        Ok(expr)
    }

    fn parse_primary_expr2(&mut self) -> Result<Expr, ()> {
        if self.match_token(TokenOther::Ampersand).is_some() {
            let ref_expr = self.parse_primary_expr2()?;
            return Ok(Expr::Reference(Box::new(ref_expr)))
        } else if self.match_token(TokenOther::Star).is_some() {
            let deref_expr = self.parse_primary_expr2()?;
            return Ok(Expr::Dereference(Box::new(deref_expr)))
        }
        
        let mut expr = self.parse_secondary_expr()?;
        loop {
            if self.match_token(TokenOther::ColonColon).is_some() {
                let name = self.parse_name();
                expr = Expr::NamespaceAccess(Box::new(expr), name);
            } else if self.match_token(TokenOther::Ampersand).is_some() {
                let ref_expr = self.parse_primary_expr2()?;
                expr = Expr::Reference(Box::new(ref_expr));
            } else {
                break
            }
        }

        Ok(expr)
    }

    fn parse_secondary_expr(&mut self) -> Result<Expr, ()> {
        if self.match_token(TokenOther::OParen).is_some() {
            let mut temp_tok = self.tok.clone();
            temp_tok.next();

            let next_is_colon = match temp_tok.peek() {
                Some(Token::Other(TokenOther::Colon)) => true,
                _ => false,
            };

            if self.is_token(TokenOther::CParen) || (self.is_name() && next_is_colon) {
                let mut params = Vec::new();
                if self.match_token(TokenOther::CParen).is_none() {
                    
                    let name = self.parse_name();
                    self.expect_token(TokenOther::Colon);
                    let type_expr = self.parse_type_expr()?;
                    params.push(Stmt::ConstDecl(name, None, Some(type_expr), false));

                    loop {
                        if self.match_token(TokenOther::Comma).is_some() {
                            let name = self.parse_name();
                            self.expect_token(TokenOther::Colon);
                            let type_expr = self.parse_type_expr()?;
                            params.push(Stmt::ConstDecl(name, None, Some(type_expr), false));
                        } else {
                            self.expect_token(TokenOther::CParen);
                            break
                        }
                    }
                }

                let return_type = if self.match_token(TokenOther::Colon).is_some() {
                    Some(Box::new(self.parse_type_expr()?))
                } else {
                    None
                };

                let expr = self.parse_expr()?;
                if return_type.is_some() && !expr.is_block() {
                    self.emit_diagnostic("expected block after return type");
                    Err(())
                } else {
                    Ok(Expr::Function(Box::new(expr), return_type, params))
                }
            } else {
                todo!()
            }
        } else if self.match_token(TokenOther::OParen).is_some() {
            self.expect_token(TokenOther::CParen);
            Ok(Expr::TypeUnit)
        } else if self.match_token(TokenOther::If).is_some() {
            self.expect_token(TokenOther::OParen);
            let condition = self.parse_expr()?;
            self.expect_token(TokenOther::CParen);

            let body = self.parse_expr()?;
            let else_body = if self.match_token(TokenOther::Else).is_some() {
                Some(Box::new(self.parse_expr()?))
            } else {
                None
            };

            Ok(Expr::If(Box::new(condition), Box::new(body), else_body))
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
        } else if self.match_token(TokenOther::TypeUInt64).is_some() {
            Ok(Expr::TypeUInt64)
        } else if self.match_token(TokenOther::TypeString).is_some() {
            Ok(Expr::TypeString)
        } else if let Some(token) = self.tok.peek() {
            match token {
                Token::IntLiteral(int) => {
                    self.tok.next();
                    Ok(Expr::IntLit(*int))
                }
                Token::StringLiteral(string) => {
                    self.tok.next();
                    Ok(Expr::StringLit(string.clone()))
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

