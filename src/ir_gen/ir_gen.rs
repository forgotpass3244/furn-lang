use std::collections::VecDeque;

use crate::{ir_gen::{cmpld_program::CompiledProgram, ctimeval::CTimeVal, external::ExternalInfo, global::GlobalInfo, ir::IRNode, scope::Scope, symbol::CmplSymbol, typeval::TypeVal, variable::Variable}, parser::ast::{Expr, Stmt}};

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

    fn open_scope(&mut self) {
        self.scopes.push_front(Scope::new());
    }

    fn scope_locals_stackp(&self) -> Option<usize> {
        let mut stack_loc = self.stack_sz;
        if let Some(scope) = self.scopes.front() {
            for var in scope.iter() {
                if let Some(_) = var.stack_loc {
                    stack_loc -= var.type_val.size_of();
                }
            }
        } else {
            panic!()
        }

        if stack_loc == self.stack_sz {
            None
        } else {
            Some(stack_loc)
        }
    }

    fn close_scope(&mut self) {
        if let Some(scope) = self.scopes.pop_front() {

            let mut locals_size_total = 0;

            for var in scope.iter() {
                if let Some(_) = var.stack_loc {
                    locals_size_total += var.type_val.size_of()
                }
            }

            self.emit_node(IRNode::StackDealloc(locals_size_total));
            self.stack_sz -= locals_size_total;

        } else {
            panic!("warning (this shouldn't happen): attempted to close scope but there is are no scopes to close")
        }
    }

    fn has_local_scope(&self) -> bool {
        !self.scopes.is_empty()
    }

    fn lookup_var(&self, name: &str) -> Option<&Variable<'a>> {
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

    #[allow(dead_code)]
    fn lookup_var_mut(&mut self, name: &str) -> Option<&mut Variable<'a>> {
        for scope in &mut self.scopes {
            if let Some(var) = scope.lookup_mut(name) {
                return Some(var)
            }
        }

        if let Some(var) = self.global_scope.lookup_mut(name) {
            return Some(var)
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

    fn new_global_pos(&mut self, type_val: &TypeVal) -> usize {
        let prev = self.global_sz;
        self.global_sz += type_val.size_of();
        prev
    }

    pub fn generate(&mut self, ast: &'a Vec<Stmt>) -> &mut CompiledProgram<'a> {
        self.cprog.add_external(ExternalInfo::new("print", "rt", true));
        self.add_global(Variable {
            name: "print",
            type_val: TypeVal::FunctionPointer(Box::new(TypeVal::Unit)),
            global_pos: None,
            stack_loc: None,
            const_val: None,
            external: Some(ExternalInfo::new("print", "rt", true)),
        });

        for stmt in ast {
            self.gen_stmt(&stmt);
        }

        if self.cprog.get_package_name().is_none() {
            for global in self.cprog.globals_iter() {
                    if global.is_exported && global.name != "main" {
                    todo!("add compile errors (unable to export any symbols if a package name was never declared)");
                }
            }
        }
        
        &mut self.cprog
    }

    fn emit_node(&mut self, node: IRNode<'a>) {
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

            Stmt::ConstDecl(name, init, type_expr, is_exported) => self.gen_decl(name, init, type_expr, true, *is_exported),
            Stmt::VarDecl(name, init, type_expr, is_exported) => self.gen_decl(name, init, type_expr, false, *is_exported),
        }
    }

    fn gen_decl(&mut self, name: &'a str, init: &'a Option<Expr>, type_expr: &'a Option<Expr>, is_const: bool, is_exported: bool) {

        if self.has_local_scope() && is_exported {
            todo!("add compile errors (exports must be global variables)");
        }

        let mut type_val = TypeVal::UInt64;
        if let Some(type_expr) = type_expr {
            type_val = self.resolve_expr(type_expr).type_val;
        }

        let is_global_var = !self.has_local_scope();
        let global_pos = if is_global_var { Some(self.new_global_pos(&type_val)) } else { None };

        let mut var = Variable {
            name,
            type_val: type_val.clone(),
            global_pos,
            stack_loc: None,
            const_val: None,
            external: None,
        };

        if let Some(expr) = init {

            let symbol = self.resolve_expr(expr);
            if type_expr.is_none() {
                var.type_val = symbol.type_val;
            }

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
                var.stack_loc = Some(self.stack_sz);
            }

        } else if is_exported {
            todo!("add compile errors (exported symbol is not initialized)");
        } else if !is_const {
            self.emit_node(IRNode::StackAlloc(type_val.size_of()));
            self.stack_sz += type_val.size_of();
            var.stack_loc = Some(self.stack_sz);
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

            CTimeVal::StringSlice(pointer, len) => {
                self.emit_node(IRNode::PushStaticStringPointer(*pointer));
                self.emit_node(IRNode::Push64(*len as u64));
                self.stack_sz += 16;
            },

            CTimeVal::Function { address, .. } => {
                let offset: i16 = (self.cprog.count_ir() - *address).try_into().unwrap();
                self.emit_node(IRNode::PushAddressFromOffset(-offset));
                self.stack_sz += ADDRESS_SIZE;
            },

            _ => todo!(),
        }
    }

    fn gen_expr(&mut self, expr: &'a Expr) {
        match expr {
            Expr::TypeUInt64 => {
                todo!("add compile errors (type `u64` is not allowed as a generatable expression)");
            },

            Expr::TypeString => {
                todo!("add compile errors (type `str` is not allowed as a generatable expression)");
            },
            
            Expr::IntLit(int) => {
                self.emit_node(IRNode::Push64(*int));
                self.stack_sz += SIZE_64;
            },

            Expr::StringLit(string) => {
                let static_string_pointer = self.new_static_string(string);
                self.emit_node(IRNode::PushStaticStringPointer(static_string_pointer));
                self.emit_node(IRNode::Push64(string.len() as u64));
                self.stack_sz += 16;
            },

            Expr::Call(expr, args) => {
                let symbol = self.resolve_expr(expr);
                match symbol.type_val {
                    TypeVal::FunctionPointer(return_type_val) => {
                        // alloc return value
                        self.emit_node(IRNode::StackAlloc(return_type_val.size_of()));
                        self.stack_sz += return_type_val.size_of();

                        // push args
                        let prev_stack_sz = self.stack_sz;
                        for arg in args {
                            self.gen_expr(arg);
                        }
                        
                        // actually perform the function call
                        self.gen_expr(expr);
                        self.emit_node(IRNode::Call);
                        self.stack_sz = prev_stack_sz;
                    },
                    _ => todo!("add compile errors (cannot perform call on this type)"),
                }
            },

            Expr::Variable(name) => {
                let var = self.lookup_var(name).cloned();

                if let Some(var) = var {
                    let type_val = var.type_val;

                    if let Some(const_val) = var.const_val {
                        self.gen_const_val(&const_val);
                    } else if let Some(external) = var.external {
                        self.external_read_push(&type_val, external.clone());
                    } else {
                        if let Some(pos) = var.global_pos {
                            self.global_read_push(&type_val, pos);
                        } else if let Some(stack_loc) = var.stack_loc {
                            self.stack_read_push(&type_val, self.stack_sz - stack_loc);
                        }
                    }
                } else {
                    todo!("add compile errors (variable not found)")
                }

            },

            Expr::Block(body, return_expr) => {

                self.open_scope();

                let prev_stack_sz = self.stack_sz;

                // alloc for return (final expr)
                let stack_alloc_label = self.cprog.count_ir();
                self.emit_node(IRNode::StackAlloc(0));

                for stmt in body {
                    self.gen_stmt(stmt);
                }

                if let Some(return_expr) = return_expr {
                    self.gen_expr(return_expr);
                }

                let mut return_type_val = None;
                let mut return_size = 0;

                if let Some(return_expr) = return_expr {
                    let type_val = self.resolve_expr(return_expr).type_val;
                    return_size = type_val.size_of();
                    return_type_val = Some(type_val);

                    // alloc for return (final expr)
                    match self.cprog.node_mut_at(stack_alloc_label) {
                        IRNode::StackAlloc(size) => {
                            *size = return_size;
                        },
                        _ => unreachable!(),
                    }

                    self.cprog.realign_stack_offsets(stack_alloc_label, self.stack_sz - prev_stack_sz, return_size);
                    self.stack_sz += return_size;
                }

                if let Some(type_val) = return_type_val {
                    match self.scope_locals_stackp() {
                        Some(locals_begin_stackp) => self.pop_to_stack(&type_val, (self.stack_sz - locals_begin_stackp) + return_size),
                        None => self.pop_to_stack(&type_val, return_size),
                    }
                }

                self.close_scope();
            },

            Expr::Function(_) => {
                todo!("add compile errors (functions must be inlined at compile time)");
            },
        }
    }
    
    fn resolve_expr(&mut self, expr: &'a Expr) -> CmplSymbol {
        match expr {
            Expr::TypeUInt64 => CmplSymbol {
                const_val: Some(CTimeVal::Type(TypeVal::UInt64)),
                type_val: TypeVal::UInt64,
            },

            Expr::TypeString => CmplSymbol {
                const_val: Some(CTimeVal::Type(TypeVal::StringSlice)),
                type_val: TypeVal::StringSlice,
            },
            
            Expr::IntLit(int) => CmplSymbol {
                const_val: Some(CTimeVal::UInt(*int)),
                type_val: TypeVal::UInt64,
            },

            Expr::StringLit(string) => CmplSymbol {
                const_val: Some(CTimeVal::StringSlice(self.new_static_string(string), string.len())),
                type_val: TypeVal::StringSlice,
            },

            Expr::Call(..) => {
                println!("TODO: resolve_expr(Expr::Call(..))");
                CmplSymbol {
                    const_val: None,
                    type_val: TypeVal::UInt64,
                }
            },

            Expr::Variable(name) => {
                let var = self.lookup_var(name);
                if let Some(var) = var {
                    CmplSymbol {
                        const_val: var.const_val.clone(),
                        type_val: var.type_val.clone(),
                    }
                } else {
                    CmplSymbol {
                        const_val: None,
                        type_val: TypeVal::UInt64,
                    }
                }
            },

            Expr::Block(body, return_expr) => {
                if let Some(return_expr) = return_expr {
                    // return value might be a variable
                    // that is only declared inside this block
                    // so we must rollback changes after
                    let prev_ir_count = self.cprog.count_ir();
                    self.open_scope();
                    for stmt in body {
                        match stmt {
                            Stmt::ConstDecl(..) => self.gen_stmt(stmt),
                            Stmt::VarDecl(..) => self.gen_stmt(stmt),
                            _ => (),
                        }
                    }

                    let symbol = self.resolve_expr(return_expr);
                    self.close_scope();
                    self.cprog.shift_nodes(prev_ir_count..=(self.cprog.ir_pos()));

                    symbol
                } else {
                    CmplSymbol {
                        const_val: None,
                        type_val: TypeVal::UInt64,
                    }
                }
            }

            Expr::Function(expr) => 
            {
                let symbol = self.resolve_expr(expr);

                let is_global_scope = self.has_local_scope();

                let jump_over = self.cprog.count_ir();
                if is_global_scope {
                    self.emit_node(IRNode::JumpFromOffset(0));
                }

                let address = self.cprog.count_ir();
                self.gen_expr(expr);
                self.pop_to_stack(&symbol.type_val, symbol.type_val.size_of() + (SIZE_64 /* return address */));
                self.emit_node(IRNode::Return { params_size: 0 });

                if is_global_scope {
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
                        return_type_val: symbol.type_val.clone(),
                    }),
                    type_val: TypeVal::FunctionPointer(Box::new(symbol.type_val)),
                }
            }
        }
    }
}


impl<'a> IRGen<'a> {
    #[must_use]
    fn new_static_string(&mut self, string: &'a str) -> usize {
        self.cprog.add_static_string(string)
    }
}


impl<'a> IRGen<'a> {
    fn pop_to_stack(&mut self, type_val: &TypeVal, offset: usize) {
        match type_val {
            TypeVal::UInt64 => self.emit_node(IRNode::Pop64ToStack(offset)),
            TypeVal::FunctionPointer(..) => self.emit_node(IRNode::Pop64ToStack(offset)),
            TypeVal::MethodPointer | TypeVal::StringSlice => {
                self.emit_node(IRNode::Pop64ToStack(offset));
                self.emit_node(IRNode::Pop64ToStack(offset));
            },

            TypeVal::Unit => (),
        }

        self.stack_sz -= type_val.size_of();
    }

    fn global_read_push(&mut self, type_val: &TypeVal, global_pos: usize) {
        match type_val {
            TypeVal::UInt64 | TypeVal::FunctionPointer(..) => {
                self.emit_node(IRNode::GlobalReadPush64(global_pos));
            },

            TypeVal::MethodPointer | TypeVal::StringSlice => {
                self.emit_node(IRNode::GlobalReadPush64(global_pos));
                self.emit_node(IRNode::GlobalReadPush64(global_pos + 8));
            },

            TypeVal::Unit => (),
        }

        self.stack_sz += type_val.size_of();
    }

    fn external_read_push(&mut self, type_val: &TypeVal, external: ExternalInfo<'a>) {
        match type_val {
            TypeVal::UInt64 | TypeVal::FunctionPointer(..) => {
                self.emit_node(IRNode::ExternalReadPush64(external));
            },

            TypeVal::MethodPointer | TypeVal::StringSlice => {
                self.emit_node(IRNode::ExternalReadPush64(external));
            },

            TypeVal::Unit => (),
        }

        self.stack_sz += type_val.size_of();
    }

    fn stack_read_push(&mut self, type_val: &TypeVal, offset: usize) {
        match type_val {
            TypeVal::UInt64 | TypeVal::FunctionPointer(..) => {
                self.emit_node(IRNode::StackReadPush64(offset));
            },

            TypeVal::MethodPointer | TypeVal::StringSlice => {
                self.emit_node(IRNode::StackReadPush64(offset + 8));
                self.emit_node(IRNode::StackReadPush64(offset + 8));
            },

            TypeVal::Unit => (),
        }

        self.stack_sz += type_val.size_of();
    }
}

