use std::iter::Peekable;

use crate::{lexer::tokens::{SourceLocation, TokenEnum, Tokens, TokensIterator}, parser::ast::{AstBlock, Expr, ExprEnum, IfKind, Stmt, StmtEnum}, tok::token_other::TokenOther};


pub struct Parser<'a> {
    tok: Peekable<TokensIterator<'a, TokenOther>>,
    err_count: usize,
    cur_end_tokens: Vec<TokenOther>,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: &'a Tokens<TokenOther>) -> Self {
        Self {
            tok: tokens.iter().peekable(),
            err_count: 0,
            cur_end_tokens: vec![TokenOther::End],
        }
    }
    
    pub fn has_errors(&self) -> bool {
        self.err_count > 0
    }

    fn emit_diagnostic(&mut self, message: &str, loc: SourceLocation) {
        self.err_count += 1;
        println!("(Line {}, Col {}): {}", loc.line, loc.col, message);
    }

    fn emit_diagnostic_here(&mut self, message: &str) {
        let loc = self.cur_loc();
        self.emit_diagnostic(message, loc);
    }

    fn cur_loc(&mut self) -> SourceLocation {
        if let Some(token) = self.tok.peek() {
            token.get_loc()
        } else {
            SourceLocation::garbage()
        }
    }

    #[warn(unused_results)]
    fn is_token(&mut self, expected: TokenOther) -> bool {
        match self.tok.peek() {
            Some(token) => {
                match token.as_enum() {
                    TokenEnum::Other(token) => *token == expected,
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
            match token.as_enum() {
                TokenEnum::Other(token) => Some(token),
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
        let mut temp_tok = self.tok.clone();
        let token = temp_tok.peek();
        if self.match_token(expected).is_none() {
            self.emit_diagnostic_here(if let Some(token) = token {
                format!("expected token '{expected}' but got {token}")
            } else {
                format!("expected token '{expected}' but got EOF")
            }.as_str());
        }
    }

    #[allow(dead_code)]
    fn expect_terminator(&mut self) {
        if self.match_terminator().is_none() {
            self.emit_diagnostic_here("expected terminator: `;`");
        }
    }

    fn unnecessary_terminator(&mut self) {
        let message = "unnecessary terminator `;`, remove it";

        let mut temp_tok = self.tok.clone();
        temp_tok.next();
        if let Some(token) = temp_tok.peek() {
            if (
                match token.as_enum() {
                    TokenEnum::Other(token) => !self.cur_end_tokens.contains(token),
                    _ => true,
                }
            ) && self.match_terminator().is_some() {
                self.emit_diagnostic_here(message);
            } else {
                self.match_terminator();
            }
        } else if self.match_terminator().is_some() {
            self.emit_diagnostic_here(message);
        }
    }

    fn parse_name(&mut self) -> String {
        let loc = self.cur_loc();

        match self.tok.next() {
            Some(token) => match token.as_enum() {
                TokenEnum::Ident(name) => {
                    name.clone()
                },
                
                _ => {
                    self.emit_diagnostic("expected name", loc);
                    "(err)".to_string()
                },
            },

            None => {
                self.emit_diagnostic_here("expected name, but got EOF");
                "(eof)".to_string()
            },
        }
    }

    fn is_name(&mut self) -> bool {
        match self.tok.peek() {
            Some(token) => match token.as_enum() {
                TokenEnum::Ident(..) => true,
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

        let loc = self.cur_loc();

        if self.match_token(TokenOther::Let).is_some() {
            self.parse_const_decl(false, loc)
        } else if self.match_token(TokenOther::Var).is_some() {
            self.parse_var_decl(false, loc)
        } else if self.match_token(TokenOther::Public).is_some() {
            if self.match_token(TokenOther::Var).is_some() {
                self.parse_var_decl(true, loc)
            } else {
                self.parse_const_decl(true, loc)
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
                Ok(StmtEnum::AliasDecl(Some(name), expr, is_call).to_stmt(loc))
            } else {
                self.expect_terminator();
                Ok(StmtEnum::AliasDecl(None, expr, is_call).to_stmt(loc))
            }
        } else if self.match_token(TokenOther::Package).is_some() {
            let name = self.parse_name();
            self.expect_terminator();
            Ok(StmtEnum::PackageDecl(name).to_stmt(loc))
        } else {

            let expr = self.parse_expr()?;

            let is_final_expr = if {
                if let Some(token) = self.tok.peek() {
                    match token.as_enum() {
                        TokenEnum::Other(token) => self.cur_end_tokens.contains(token),
                        _ => false,
                    }
                } else {
                    false
                }
            } {
                true
            } else if expr.is_block() {
                self.unnecessary_terminator();
                false
            } else {
                self.expect_terminator();
                false
            };

            Ok(StmtEnum::Expr(expr, is_final_expr).to_stmt(loc))
        }

    }

    fn parse_const_decl(&mut self, is_exported: bool, loc: SourceLocation) -> Result<Stmt, ()> {
        let name = self.parse_name();

        let type_expr = if !self.is_terminator() && !self.is_token(TokenOther::ColonColon) {
            Some(self.parse_type_expr()?)
        } else {
            None
        };

        if self.match_terminator().is_some() {
            Ok(StmtEnum::ConstDecl(name, None, type_expr, is_exported).to_stmt(loc))
        } else if self.match_token(TokenOther::ColonColon).is_some() {
            let expr = self.parse_expr()?;
            if expr.is_block() {
                self.unnecessary_terminator();
            } else {
                self.expect_terminator();
            }
            Ok(StmtEnum::ConstDecl(name, Some(expr), type_expr, is_exported).to_stmt(loc))
        } else {
            self.emit_diagnostic_here("expected either `;`, `::` or type annotation (use `::` for initializer)");
            Ok(StmtEnum::ConstDecl(name, None, type_expr, is_exported).to_stmt(loc))
        }
    }
    
    fn parse_var_decl(&mut self, is_exported: bool, loc: SourceLocation) -> Result<Stmt, ()> {
        let name = self.parse_name();

        let type_expr = if !self.is_terminator() && !self.is_token(TokenOther::ColonColon) {
            Some(self.parse_type_expr()?)
        } else {
            None
        };

        if self.match_terminator().is_some() {
            Ok(StmtEnum::VarDecl(name, None, type_expr, is_exported).to_stmt(loc))
        } else if self.match_token(TokenOther::ColonColon).is_some() {
            let expr = self.parse_expr()?;
            if expr.is_block() {
                self.unnecessary_terminator();
            } else {
                self.expect_terminator();
            }
            Ok(StmtEnum::VarDecl(name, Some(expr), type_expr, is_exported).to_stmt(loc))
        } else {
            self.emit_diagnostic_here("expected either `;`, `::` or type annotation (use `::` for initializer)");
            Ok(StmtEnum::VarDecl(name, None, type_expr, is_exported).to_stmt(loc))
        }
    }
}

impl Parser<'_> {
    fn parse_type_expr(&mut self) -> Result<Expr, ()> {
        self.parse_primary_expr()
    }

    fn parse_expr(&mut self) -> Result<Expr, ()> {
        self.parse_expr_internal(0)
    }

    fn parse_expr_internal(&mut self, min_prec: u8) -> Result<Expr, ()> {
        let mut lhs = self.parse_primary_expr()?;

        while let Some(tok) = self.tok.peek() {
            let op = match tok.as_enum() {
                TokenEnum::Other(tok) => tok.to_operator(),
                _ => None,
            };

            let Some(op) = op else { break };

            let prec = op.precedence();
            if prec < min_prec {
                break
            }

            self.tok.next(); // consume operator

            let rhs = self.parse_expr_internal(prec + 1)?;

            lhs = ExprEnum::BinaryOp {
                operands: Box::new((lhs, rhs)),
                op,
            }.to_expr(self.cur_loc());
        }

        Ok(lhs)
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

                expr = ExprEnum::Call(Box::new(expr), args).to_expr(self.cur_loc());
            } else {
                break
            }
        }

        Ok(expr)
    }

    fn parse_primary_expr2(&mut self) -> Result<Expr, ()> {
        let loc = self.cur_loc();
        if self.match_token(TokenOther::Ampersand).is_some() {
            let ref_expr = self.parse_primary_expr2()?;
            return Ok(ExprEnum::Reference(Box::new(ref_expr)).to_expr(loc));
        } else if self.match_token(TokenOther::Star).is_some() {
            let deref_expr = self.parse_primary_expr2()?;
            return Ok(ExprEnum::Dereference(Box::new(deref_expr)).to_expr(loc));
        }
        
        let mut expr = self.parse_secondary_expr()?;
        loop {
            if self.match_token(TokenOther::Dot).is_some() {
                let name = self.parse_name();
                expr = ExprEnum::MemberAccess(Box::new(expr), name).to_expr(self.cur_loc());
            } else {
                break
            }
        }

        Ok(expr)
    }

    fn parse_secondary_expr(&mut self) -> Result<Expr, ()> {
        let loc = self.cur_loc();
        
        if self.match_token(TokenOther::OParen).is_some() {
            let mut temp_tok = self.tok.clone();
            temp_tok.next();

            let next_is_colon = if let Some(tok) = temp_tok.peek() {
                match tok.as_enum() {
                    TokenEnum::Other(TokenOther::Colon) => true,
                    _ => false,
                }
            } else {
                false
            };

            if self.is_token(TokenOther::CParen) || (self.is_name() && next_is_colon) {
                let mut params = Vec::new();
                if self.match_token(TokenOther::CParen).is_none() {
                    
                    let name = self.parse_name();
                    self.expect_token(TokenOther::Colon);
                    let type_expr = self.parse_type_expr()?;
                    params.push(StmtEnum::ConstDecl(name, None, Some(type_expr), false).to_stmt(loc));

                    loop {
                        if self.match_token(TokenOther::Comma).is_some() {
                            let name = self.parse_name();
                            self.expect_token(TokenOther::Colon);
                            let type_expr = self.parse_type_expr()?;
                            params.push(StmtEnum::ConstDecl(name, None, Some(type_expr), false).to_stmt(loc));
                        } else {
                            self.expect_token(TokenOther::CParen);
                            break
                        }
                    }
                }

                let return_type = if self.match_token(TokenOther::Colon).is_some() {
                    let type_expr = self.parse_type_expr()?;
                    self.expect_token(TokenOther::Colon);
                    Some(Box::new(type_expr))
                } else {
                    None
                };

                let result = self.parse_block(vec![TokenOther::End])?;
                Ok(ExprEnum::Function(result.0, return_type, params).to_expr(loc))
            } else {
                todo!()
            }
        } else if self.match_token(TokenOther::TypeVoid).is_some() {
            Ok(ExprEnum::TypeUnit.to_expr(loc))
        } else if self.match_token(TokenOther::If).is_some() {
            self.expect_token(TokenOther::OParen);

            let condition = self.parse_expr()?;
            let if_kind = if self.match_token(TokenOther::ColonColon).is_some() {
                let name = self.parse_name();
                let type_expr = self.parse_type_expr()?;
                IfKind::ConstBinding { name, type_expr, init: condition }
            } else {
                IfKind::Conditional(condition)
            };

            self.expect_token(TokenOther::CParen);

            let result = self.parse_block(vec![TokenOther::Else, TokenOther::End])?;
            let body = result.0;
            let else_body = match result.1 {
                TokenOther::Else => Some(self.parse_block(vec![TokenOther::End])?.0),
                TokenOther::End => None,
                _ => unreachable!(),
            };

            Ok(ExprEnum::If(Box::new(if_kind), body, else_body).to_expr(loc))
        } else if self.is_token(TokenOther::Unsafe) || self.is_token(TokenOther::Do) {

            let is_unsafe_block = if self.match_token(TokenOther::Unsafe).is_some() {
                true
            } else {
                self.expect_token(TokenOther::Do);
                false
            };

            let result = self.parse_block(vec![TokenOther::End])?;
            let block = result.0;

            Ok(ExprEnum::Block(block, is_unsafe_block).to_expr(loc))
        } else if self.match_token(TokenOther::TypeUInt64).is_some() {
            Ok(ExprEnum::TypeUInt64.to_expr(loc))
        } else if self.match_token(TokenOther::TypeString).is_some() {
            Ok(ExprEnum::TypeString.to_expr(loc))
        } else if let Some(token) = self.tok.peek() {
            match token.as_enum() {
                TokenEnum::IntLiteral(int) => {
                    self.tok.next();
                    Ok(ExprEnum::IntLit(*int).to_expr(loc))
                }
                TokenEnum::StringLiteral(string) => {
                    self.tok.next();
                    Ok(ExprEnum::StringLit(string.clone()).to_expr(loc))
                }
                TokenEnum::Ident(name) => {
                    self.tok.next();
                    Ok(ExprEnum::Variable(name.clone()).to_expr(loc))
                }
                _ => Err(()),
            }
        } else {
            Err(())
        }
    }

    fn parse_block(&mut self, end_tokens: Vec<TokenOther>) -> Result<(AstBlock, TokenOther), ()> {
        let prev_end_token = self.cur_end_tokens.clone();
        let result = self.parse_block_internal(end_tokens);
        self.cur_end_tokens = prev_end_token;
        result
    }

    fn parse_block_internal(&mut self, end_tokens: Vec<TokenOther>) -> Result<(AstBlock, TokenOther), ()> {
        let mut block = AstBlock {
            body: Vec::new(),
            return_expr: None,
        };

        self.cur_end_tokens = end_tokens;

        let end_token = loop {

            let stmt = self.parse_stmt()?;

            if let Some(end_token) = {
                if let Some(token) = self.tok.peek() {
                    match token.as_enum() {
                        TokenEnum::Other(token) => if self.cur_end_tokens.contains(token) {
                            Some(token)
                        } else {
                            None
                        },
                        _ => None,
                    }
                } else {
                    None
                }
            } {

                match stmt.as_enum() {
                    StmtEnum::Expr(expr, true) => {
                        block.return_expr = Some(Box::new(expr.clone()));
                    },
                    _ => block.body.push(stmt),
                }

                self.tok.next();
                break end_token
            } else {
                block.body.push(stmt);
            }
        };

        Ok((block, *end_token))
    }
}

