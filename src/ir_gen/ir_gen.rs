use std::collections::VecDeque;

use crate::{ir_gen::{ctimeval::CTimeVal, global::GlobalInfo, ir::IRNode, scope::Scope, symbol::CmplSymbol, variable::Variable}, parser::ast::{Expr, Stmt}};

const ADDRESS_SIZE: usize = 8;
const SIZE_64: usize = 8;

pub struct IRGen<'a> {
    ir: Vec<IRNode>,
    pub globals: Vec<GlobalInfo<'a>>,
    global_scope: Scope<'a>,
    scopes: VecDeque<Scope<'a>>,
    stack_sz: usize,
}

impl<'a> IRGen<'a> {
    pub fn new() -> Self {
        Self {
            ir: Vec::new(),
            globals: Vec::new(),
            global_scope: Scope::new(),
            scopes: VecDeque::new(),
            stack_sz: 0,
        }
    }

    fn has_local_scope(&self) -> bool {
        !self.scopes.is_empty()
    }

    fn lookup_var(&self, name: &str) -> Option<&Variable<'_>> {
        for scope in &self.scopes {
            if let Some(var) = scope.lookup(name) {
                return Some(&var)
            }
        }

        if let Some(var) = self.global_scope.lookup(name) {
            return Some(&var)
        } else {
            None
        }
    }

    fn add_var(&mut self, var: Variable<'a>) {
        let scope = self.scopes.front_mut();
        if let Some(scope) = scope {
            scope.add(var);
        } else {
            let mut scope = Scope::new();
            scope.add(var);
            self.scopes.push_front(scope);

            panic!("warning (this shouldn't happen): added new local before opening a scope")
        }
    }

    fn add_global(&mut self, var: Variable<'a>) {
        self.global_scope.add(var);
    }

    pub fn generate(&mut self, ast: &'a Vec<Stmt>) -> (&Vec<IRNode>, &Vec<GlobalInfo<'a>>) {
        for stmt in ast {
            self.gen_stmt(&stmt);
        }
        (&self.ir, &self.globals)
    }

    fn emit_node(&mut self, node: IRNode) -> &mut IRNode {
        self.ir.push(node);
        self.ir.last_mut().unwrap()
    }

    fn gen_stmt(&mut self, stmt: &'a Stmt) {
        match stmt {
            Stmt::Expr(expr) => {
                let prev_stack_sz = self.stack_sz;
                self.gen_expr(expr);

                let stack_diff = self.stack_sz - prev_stack_sz;
                if stack_diff > 0 {
                    self.emit_node(IRNode::StackDealloc(stack_diff));
                    self.stack_sz -= stack_diff;
                }

            },
            Stmt::ConstDecl(name, init, is_exported) => self.gen_decl(name, init, true, *is_exported),
            Stmt::VarDecl(name, init, is_exported) => self.gen_decl(name, init, false, *is_exported),
        }
    }

    fn gen_decl(&mut self, name: &'a str, init: &'a Option<Expr>, is_const: bool, is_exported: bool) {

        if self.has_local_scope() && is_exported {
            todo!("add compile errors (exports must be global variables)");
        }

        let mut var = Variable {
            name,
            const_val: None,
        };

        if let Some(expr) = init {

            let symbol = self.resolve_expr(expr);
            if symbol.const_val.is_some() && is_const {
                var.const_val = symbol.const_val.clone();
            }
            
            if is_exported {
                if let Some(const_val) = symbol.const_val {
                    let global_info = GlobalInfo::new(name, is_exported, const_val);
                    self.globals.push(global_info);
                } else {
                    todo!("add compile errors (exported variable must be initialized with a compile-time constant)")
                }
            } else if symbol.const_val.is_none() || !is_const {
                self.gen_expr(expr);
            }

        } else if !is_const {
            self.emit_node(IRNode::Push64(0));
        }

        if self.has_local_scope() {
            self.add_var(var);
        } else {
            self.add_global(var);
        }
    }

    fn gen_const_val(&mut self, const_val: &CTimeVal) {
        match const_val {
            CTimeVal::UInt(int) => {
                self.emit_node(IRNode::Push64(*int));
                self.stack_sz += SIZE_64;
            },

            CTimeVal::Function { address } => {
                let offset: i16 = (self.ir.len() - *address).try_into().unwrap();
                self.emit_node(IRNode::PushAddressFromOffset(-offset));
                self.stack_sz += ADDRESS_SIZE;
            },

            _ => todo!(),
        }
    }

    fn gen_expr(&mut self, expr: &'a Expr) {
        match expr {
            Expr::IntLit(int) => {
                self.emit_node(IRNode::Push64(*int));
                self.stack_sz += SIZE_64;
            },

            Expr::Variable(name) => {
                let var = self.lookup_var(name).cloned();

                if let Some(var) = var {
                    if let Some(const_val) = &var.const_val {
                        self.gen_const_val(const_val);
                    }
                } else {
                    todo!("add compile errors (variable not found)")
                }

            },

            Expr::Block(body, return_expr) => {

                for stmt in body {
                    self.gen_stmt(stmt);
                }

                if let Some(return_expr) = return_expr {
                    self.gen_expr(return_expr);
                }
            },

            Expr::Function(_) => {
                panic!("add compile errors (functions must be inlined at compile time)");
            },
        }
    }
    
    fn resolve_expr(&mut self, expr: &'a Expr) -> CmplSymbol {
        match expr {
            Expr::IntLit(int) => CmplSymbol {
                const_val: Some(CTimeVal::UInt(*int)),
            },

            Expr::Variable(name) => {
                let var = self.lookup_var(name);
                if let Some(var) = var {
                    CmplSymbol {
                        const_val: var.const_val.clone(),
                    }
                } else {
                    CmplSymbol {
                        const_val: None,
                    }
                }
            },

            Expr::Function(expr) => 
            {
                let jump_over = self.ir.len();
                if self.has_local_scope() {
                    self.emit_node(IRNode::JumpFromOffset(0));
                }

                let address = self.ir.len();
                self.gen_expr(expr);
                self.emit_node(IRNode::Pop64ToStack(16));
                self.emit_node(IRNode::Return);

                if self.has_local_scope() {
                    let after_func = self.ir.len();

                    match &mut self.ir[jump_over] {
                        IRNode::JumpFromOffset(offset_ref) => {
                            let offset: i16 = (after_func - jump_over).try_into().unwrap();
                            *offset_ref = offset;
                        },
                        _ => unreachable!(),
                    }
                }

                CmplSymbol {
                    const_val: Some(CTimeVal::Function {
                        address,
                    }),
                }
            }

            _ => todo!(),
        }
    }
}




