use std::collections::VecDeque;

use crate::{ir_gen::{ctimeval::CTimeVal, global::GlobalInfo, ir::IRNode, cmpld_program::CompiledProgram, scope::Scope, symbol::CmplSymbol, variable::Variable}, parser::ast::{Expr, Stmt}};

const ADDRESS_SIZE: usize = 8;
const SIZE_64: usize = 8;

pub struct IRGen<'a> {
    cprog: CompiledProgram<'a>,
    global_scope: Scope<'a>,
    scopes: VecDeque<Scope<'a>>,
    stack_sz: usize,
    global_sz: usize,
}

impl<'a> IRGen<'a> {
    pub fn new() -> Self {
        Self {
            cprog: CompiledProgram::new(),
            global_scope: Scope::new(),
            scopes: VecDeque::new(),
            stack_sz: 0,
            global_sz: 0,
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

    fn new_global_pos(&mut self) -> usize {
        let prev = self.global_sz;
        self.global_sz += SIZE_64;
        prev
    }

    pub fn generate(&mut self, ast: &'a Vec<Stmt>) -> &mut CompiledProgram<'a> {
        for stmt in ast {
            self.gen_stmt(&stmt);
        }

        if self.cprog.get_package_name().is_none() {
            if let Some(first_global) = self.cprog.first_global() {
                if self.cprog.global_count() > 1 || first_global.name != "main" {
                    todo!("add compile errors (unable to export any symbols if a package name was never declared)");
                }
            }
        }
        
        &mut self.cprog
    }

    fn emit_node(&mut self, node: IRNode) {
        self.cprog.app_node(node);
    }

    fn gen_stmt(&mut self, stmt: &'a Stmt) {
        match stmt {
            Stmt::Expr(expr, _) => {
                let prev_stack_sz = self.stack_sz;
                self.gen_expr(expr);

                let stack_diff = self.stack_sz - prev_stack_sz;
                if stack_diff > 0 {
                    self.emit_node(IRNode::StackDealloc(stack_diff));
                    self.stack_sz -= stack_diff;
                }

            },

            Stmt::PackageDecl(name) => {
                if let Some(name) = self.cprog.get_package_name() {
                    todo!("add compile errors (package name was previously declared as {name})");
                } else {
                    self.cprog.set_package_name(Some(name));
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

        let is_global_var = !self.has_local_scope();
        let global_pos = if is_global_var { Some(self.new_global_pos()) } else { None };

        let mut var = Variable {
            name,
            global_pos,
            const_val: None,
        };

        if let Some(expr) = init {

            let symbol = self.resolve_expr(expr);
            if symbol.const_val.is_some() && is_const {
                var.const_val = symbol.const_val.clone();
            }
            
            if is_global_var {
                if let Some(const_val) = symbol.const_val {
                    let global_info = GlobalInfo::new(global_pos.unwrap_or_default(), name, is_exported, const_val, is_const);
                    self.cprog.add_global(global_info);
                } else {
                    todo!("add compile errors (global variable must be initialized with a compile-time constant)")
                }
            } else if symbol.const_val.is_none() || !is_const {
                self.gen_expr(expr);
            }

        } else if is_exported {
            todo!("add compile errors (exported symbol is not initialized)");
        } else if !is_const {
            self.emit_node(IRNode::Push64(0));
        }

        if is_global_var {
            self.add_global(var);
        } else {
            self.add_var(var);
        }
    }

    fn gen_const_val(&mut self, const_val: &CTimeVal) {
        match const_val {
            CTimeVal::UInt(int) => {
                self.emit_node(IRNode::Push64(*int));
                self.stack_sz += SIZE_64;
            },

            CTimeVal::Function { address } => {
                let offset: i16 = (self.cprog.count_ir() - *address).try_into().unwrap();
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
                    } else {
                        if let Some(pos) = var.global_pos {
                            self.emit_node(IRNode::GlobalReadPush64(pos));
                            self.stack_sz += SIZE_64;
                        } else {
                            todo!("local var read")
                        }
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
                } else {
                    // TODO: if it doesnt have a final expr
                    // then dont even push anything
                    self.emit_node(IRNode::Push64(0));
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
                let jump_over = self.cprog.count_ir();
                if self.has_local_scope() {
                    self.emit_node(IRNode::JumpFromOffset(0));
                }

                let address = self.cprog.count_ir();
                self.gen_expr(expr);
                self.emit_node(IRNode::Pop64ToStack(16));
                self.emit_node(IRNode::Return);

                if self.has_local_scope() {
                    let after_func = self.cprog.count_ir();

                    match self.cprog.node_mut_at(jump_over) {
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




