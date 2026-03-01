use std::collections::{HashMap, VecDeque};

use crate::{ir_gen::{cmpld_program::CompiledProgram, ctimeval::CTimeVal, external::ExternalInfo, global::GlobalInfo, ir::IRNode, scope::Scope, symbol::CmplSymbol, typeval::TypeVal, typeval::TypeValEnum, variable::Variable}, parser::ast::{Expr, Stmt}};

const ADDRESS_SIZE: usize = 8;
const SIZE_64: usize = 8;


pub struct IRGen<'a> {
    cprog: CompiledProgram<'a>,
    global_scope: Scope,
    scopes: VecDeque<Scope>,
    stack_sz: usize,
    global_sz: usize,
    symbol_cache: HashMap<*const Expr, CmplSymbol>,
}

impl<'a> IRGen<'a> {
    pub fn new() -> Self {
        Self {
            cprog: CompiledProgram::new(),
            global_scope: Scope::new(),
            scopes: VecDeque::new(),
            stack_sz: 0,
            global_sz: 0,
            symbol_cache: HashMap::new(),
        }
    }

    fn open_scope(&mut self) {
        self.scopes.push_front(Scope::new());
    }

    fn scope_locals_stackp(&self) -> Option<usize> {
        let mut stack_loc = self.stack_sz;
        if let Some(scope) = self.scopes.front() {
            for var in scope.iter() {
                if !var.is_alias {
                    if let Some(_) = var.stack_loc {
                        stack_loc -= var.type_val.size_of();
                    }
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
                if !var.is_alias {
                    if let Some(_) = var.stack_loc {
                        locals_size_total += var.type_val.size_of()
                    }
                }
            }

            self.emit_node(IRNode::StackDealloc(locals_size_total));
            self.stack_sz -= locals_size_total;

        } else {
            panic!("warning (this shouldn't happen): attempted to close scope but there is are no scopes to close")
        }
    }

    fn close_scope_noclean(&mut self) -> usize {
        if let Some(scope) = self.scopes.pop_front() {
            let mut locals_size_total = 0;

            for var in scope.iter() {
                if !var.is_alias {
                    if let Some(_) = var.stack_loc {
                        locals_size_total += var.type_val.size_of()
                    }
                }
            }

            locals_size_total
        } else {
            panic!("warning (this shouldn't happen): attempted to close (noclean) scope but there is are no scopes to close")
        }
    }

    fn has_local_scope(&self) -> bool {
        !self.scopes.is_empty()
    }

    fn lookup_var(&self, name: &str) -> Option<&Variable> {
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
    fn lookup_var_mut(&mut self, name: &str) -> Option<&mut Variable> {
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

    fn add_var(&mut self, var: Variable) {
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

    fn add_global(&mut self, var: Variable) {
        self.global_scope.add(var);
    }

    fn new_global_pos(&mut self, type_val: &TypeVal) -> usize {
        let prev = self.global_sz;
        self.global_sz += type_val.size_of();
        prev
    }

    pub fn generate(&mut self, ast: &'a Vec<Stmt>) -> &mut CompiledProgram<'a> {
        let mut rt_map = HashMap::new();

        self.cprog.add_external(ExternalInfo::new("print".to_string(), "Rt".to_string(), true));
        let print_var = Variable {
            name: "print".to_string(),
            type_val: TypeValEnum::FunctionPointer(vec![TypeValEnum::UInt64.to_tval()], Box::new(TypeValEnum::Unit.to_tval())).to_tval(),
            global_pos: None,
            stack_loc: None,
            const_val: None,
            external: Some(ExternalInfo::new("print".to_string(), "Rt".to_string(), true)),
            is_alias: false,
        };
        rt_map.insert("print".to_string(), print_var);

        self.cprog.add_external(ExternalInfo::new("print_char".to_string(), "Rt".to_string(), true));
        let print_var = Variable {
            name: "print_char".to_string(),
            type_val: TypeValEnum::FunctionPointer(vec![TypeValEnum::UInt64.to_tval()], Box::new(TypeValEnum::Unit.to_tval())).to_tval(),
            global_pos: None,
            stack_loc: None,
            const_val: None,
            external: Some(ExternalInfo::new("print_char".to_string(), "Rt".to_string(), true)),
            is_alias: false,
        };
        rt_map.insert("print_char".to_string(), print_var);

        self.cprog.add_external(ExternalInfo::new("print_digit".to_string(), "Rt".to_string(), true));
        let print_var = Variable {
            name: "print_digit".to_string(),
            type_val: TypeValEnum::FunctionPointer(vec![TypeValEnum::UInt64.to_tval()], Box::new(TypeValEnum::Unit.to_tval())).to_tval(),
            global_pos: None,
            stack_loc: None,
            const_val: None,
            external: Some(ExternalInfo::new("print_digit".to_string(), "Rt".to_string(), true)),
            is_alias: false,
        };
        rt_map.insert("print_digit".to_string(), print_var);
        
        self.cprog.add_external(ExternalInfo::new("print_u64".to_string(), "Rt".to_string(), true));
        let print_var = Variable {
            name: "print_u64".to_string(),
            type_val: TypeValEnum::FunctionPointer(vec![TypeValEnum::UInt64.to_tval()], Box::new(TypeValEnum::Unit.to_tval())).to_tval(),
            global_pos: None,
            stack_loc: None,
            const_val: None,
            external: Some(ExternalInfo::new("print_u64".to_string(), "Rt".to_string(), true)),
            is_alias: false,
        };
        rt_map.insert("print_u64".to_string(), print_var);

        let rt_namespace = CTimeVal::Namespace(rt_map);
        let rt_var = Variable {
            name: "Rt".to_string(),
            type_val: TypeValEnum::Unit.to_tval(),
            global_pos: None,
            stack_loc: None,
            const_val: Some(rt_namespace),
            external: None,
            is_alias: false,
        };

        self.add_global(rt_var);

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

            Stmt::AliasDecl(alias_name, expr, is_call) => {
                let var = self.resolve_expr(expr).var;
                if let Some(var) = var {
                    match var.type_val.as_enum() {
                        TypeValEnum::FunctionPointer(..) => if !(*is_call) {
                            todo!("add compile errors (add `()` to function alias)");
                        },
                        _ => if *is_call {
                            todo!("add compile errors (remove `()` from this alias)");
                        },
                    }

                    let mut new_binding = var.clone();
                    new_binding.is_alias = true;
                    if let Some(alias_name) = alias_name {
                        new_binding.name = alias_name.clone();
                    }
                    
                    if self.has_local_scope() {
                        self.add_var(new_binding);
                    } else {
                        self.add_global(new_binding);
                    }
                } else {
                    todo!("add compile errors (alias failed, name does not exist here)");
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

        let type_val;
        if let Some(type_expr) = type_expr {
            type_val = self.resolve_expr(type_expr).type_val;
        } else if let Some(init) = init {
            type_val = self.resolve_expr(init).type_val;
        } else {
            todo!("add compile errors (type cannot be inferred here)");
            // todo!("add compile errors (type cannot be inferred in this version of the compiler due to some subtle bugs)");
        }

        let is_global_var = !self.has_local_scope();
        let global_pos = if is_global_var { Some(self.new_global_pos(&type_val)) } else { None };

        let mut var = Variable {
            name: name.to_string(),
            type_val: type_val.clone(),
            global_pos,
            stack_loc: None,
            const_val: None,
            external: None,
            is_alias: false,
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
                    todo!("add compile errors (global variable must be initialized with a compile-time constant)");
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
                let offset: i64 = (self.cprog.count_ir() - *address).try_into().unwrap();
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

            Expr::TypeUnit => {
                todo!("add compile errors (type `.()` / unit is not allowed as a generatable expression)");
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

            Expr::Dereference(expr) => {
                let symbol = self.resolve_expr(expr);
                if let Some(const_val) = symbol.const_val {
                    match const_val {
                        CTimeVal::Type(_) => {
                            todo!("add compile errors (unable to dereference a type (did you mean: `&u64`))");
                        },
                        
                        _ => {
                            // cannot deref to a ctimeval
                            todo!("add compile errors (cannot dereference to a compile time constant)");
                        },
                    }
                } else if !symbol.type_val.is_ref {
                    todo!("add compile errors (deref on a non-reference type)");
                } else {
                    self.gen_expr(expr);
                    self.stack_sz -= symbol.type_val.size_of();
                    self.emit_node(IRNode::Deref64);
                    self.stack_sz += SIZE_64;
                }
            },
            
            Expr::Reference(expr) => {
                let symbol = self.resolve_expr(expr);
                if let Some(const_val) = symbol.const_val {
                    match const_val {
                        CTimeVal::Type(_) => {
                            // to ref type
                            todo!("add compile errors (type cannot be used as a generatable expression)");
                        },
                        
                        _ => {
                            // cannot grab a reference to a ctimeval
                            todo!("add compile errors (cannot grab a reference to a compile time constant)");
                        },
                    }
                } else if symbol.type_val.is_ref {
                    // already a ref
                    todo!("add compile errors (double references are not allowed)");
                } else {
                    // ref to
                    if let Some(var) = symbol.var {
                        // ref to
                        if let Some(_global_pos) = var.global_pos {
                            todo!()
                        } else if let Some(stack_loc) = var.stack_loc {
                            self.emit_node(IRNode::PushStackPointer(self.stack_sz - stack_loc));
                            self.stack_sz += SIZE_64;
                        } else {
                            unreachable!()
                        }
                    } else {
                        todo!("add compile errors (expected a variable to grab a reference to)");
                    }
                }
            },

            Expr::Call(expr, args) => {
                let symbol = self.resolve_expr(expr);
                match symbol.type_val.as_enum() {
                    TypeValEnum::FunctionPointer(_param_type_vals, return_type_val) => {
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
                    _ => todo!("add compile errors (cannot perform call on this type) {:?}", symbol.type_val),
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

            Expr::NamespaceAccess(expr, name) => {
                let symbol = self.resolve_expr(expr);
                if let Some(const_val) = symbol.const_val {
                    match const_val {
                        CTimeVal::Namespace(map) => {
                            let var = map.get(name);
                            if let Some(var) = var {
                                let type_val = var.type_val.clone();

                                if let Some(const_val) = var.const_val.clone() {
                                    self.gen_const_val(&const_val);
                                } else if let Some(external) = var.external.clone() {
                                    self.external_read_push(&type_val, external.clone());
                                } else {
                                    if let Some(pos) = var.global_pos {
                                        self.global_read_push(&type_val, pos);
                                    } else if let Some(stack_loc) = var.stack_loc {
                                        self.stack_read_push(&type_val, self.stack_sz - stack_loc);
                                    }
                                }
                            } else {
                                todo!("add compile errors (member not found)")
                            }
                        },
                        _ => todo!("add compile errors (expected a namespace to access)"),
                    }
                } else {
                    todo!("add compile errors (not a constant value to access)")
                }
            },

            Expr::If(condition, body, else_body) => {
                // let condition_symbol = self.resolve_expr(condition);
                self.gen_expr(condition);

                // TODO: make 64bits not hardcoded
                let jump_ifn_label = self.cprog.count_ir();
                self.emit_node(IRNode::JumpIfNot64FromOffset(0));
                self.stack_sz -= 8;

                self.gen_expr(body);
                let skipover_else_label = self.cprog.count_ir();
                if else_body.is_some() {
                    self.emit_node(IRNode::JumpFromOffset(0));
                }

                let else_label = self.cprog.count_ir() as i64;
                match self.cprog.node_mut_at(jump_ifn_label) {
                    IRNode::JumpIfNot64FromOffset(offset) => {
                        *offset = else_label - (jump_ifn_label as i64);
                    },
                    _ => unreachable!(),
                }

                if let Some(else_body) = else_body {
                    self.gen_expr(else_body);

                    let after_else_label = self.cprog.count_ir() as i64;
                    match self.cprog.node_mut_at(skipover_else_label) {
                        IRNode::JumpFromOffset(offset) => {
                            *offset = after_else_label - (skipover_else_label as i64);
                        },
                        _ => unreachable!(),
                    }
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

            Expr::Function(..) => {
                todo!("add compile errors (functions must be inlined at compile time)");
            },
        }
    }

    fn resolve_expr(&mut self, expr: &'a Expr) -> CmplSymbol {
        match expr {
            Expr::Function(..) => self.resolve_expr_cached(expr),
            Expr::Block(..) => self.resolve_expr_cached(expr),
            _ => self.resolve_expr_uncached(expr),
        }
    }

    fn resolve_expr_cached(&mut self, expr: &'a Expr) -> CmplSymbol {
        if let Some(cached_symbol) = self.symbol_cache.get(&(expr as *const Expr)) {
            cached_symbol.clone()
        } else {
            let symbol = self.resolve_expr_uncached(expr);
            self.symbol_cache.insert(expr as *const Expr, symbol.clone());
            symbol
        }
    }
    
    fn resolve_expr_uncached(&mut self, expr: &'a Expr) -> CmplSymbol {
        match expr {
            Expr::TypeUInt64 => CmplSymbol {
                const_val: Some(CTimeVal::Type(TypeValEnum::UInt64.to_tval())),
                type_val: TypeValEnum::UInt64.to_tval(),
                var: None,
            },

            Expr::TypeString => CmplSymbol {
                const_val: Some(CTimeVal::Type(TypeValEnum::StringSlice.to_tval())),
                type_val: TypeValEnum::StringSlice.to_tval(),
                var: None,
            },

            Expr::TypeUnit => CmplSymbol {
                const_val: Some(CTimeVal::Type(TypeValEnum::Unit.to_tval())),
                type_val: TypeValEnum::Unit.to_tval(),
                var: None,
            },
            
            Expr::IntLit(int) => CmplSymbol {
                const_val: Some(CTimeVal::UInt(*int)),
                type_val: TypeValEnum::UInt64.to_tval(),
                var: None,
            },

            Expr::StringLit(string) => CmplSymbol {
                const_val: Some(CTimeVal::StringSlice(self.new_static_string(string), string.len())),
                type_val: TypeValEnum::StringSlice.to_tval(),
                var: None,
            },

            Expr::Dereference(expr) => {
                let symbol = self.resolve_expr(expr);
                CmplSymbol {
                    const_val: None,
                    type_val: symbol.type_val.to_nonref(),
                    var: None,
                }
            },
            
            Expr::Reference(expr) => {
                let symbol = self.resolve_expr(expr);
                if let Some(const_val) = symbol.const_val {
                    match const_val {
                        // to ref type
                        CTimeVal::Type(type_val) => {
                            if type_val.is_ref {
                                todo!("add compile errors (ref-to-ref type (`&&`) is not allowed)");
                            }
                            
                            CmplSymbol {
                                const_val: Some(CTimeVal::Type(type_val.clone().to_ref())),
                                type_val: type_val.to_ref(),
                                var: None,
                            }
                        },
                        
                        // cannot grab a reference to a ctimeval
                        _ => CmplSymbol {
                            const_val: None,
                            type_val: TypeValEnum::Unit.to_tval(),
                            var: None,
                        },
                    }
                } else if symbol.type_val.is_ref {
                    // already a ref
                    symbol
                } else {
                    // ref to
                    CmplSymbol {
                        const_val: None,
                        type_val: symbol.type_val.to_ref(),
                        var: None,
                    }
                }
            },

            Expr::Call(function, ..) => {
                let symbol = self.resolve_expr(function);
                match symbol.type_val.as_enum() {
                    TypeValEnum::FunctionPointer(_param_type_vals, return_type_val) => CmplSymbol {
                        const_val: None,
                        type_val: *return_type_val.clone(),
                        var: None,
                    },

                    _ => CmplSymbol {
                        const_val: None,
                        type_val: TypeValEnum::Unit.to_tval(),
                        var: None,
                    },
                }
            },

            Expr::Variable(name) => {
                let var = self.lookup_var(name);
                if let Some(var) = var {
                    CmplSymbol {
                        const_val: var.const_val.clone(),
                        type_val: var.type_val.clone(),
                        var: Some(var.clone())
                    }
                } else {
                    CmplSymbol {
                        const_val: None,
                        type_val: TypeValEnum::Unit.to_tval(),
                        var: None,
                    }
                }
            },

            Expr::NamespaceAccess(namespace_expr, name) => {
                let namespace_symbol = self.resolve_expr(namespace_expr);
                if let Some(const_val) = namespace_symbol.const_val {
                    match const_val {
                        CTimeVal::Namespace(map) => {
                            let var = map.get(name);
                            if let Some(var) = var {
                                CmplSymbol {
                                    const_val: var.const_val.clone(),
                                    type_val: var.type_val.clone(),
                                    var: Some(var.clone()),
                                }
                            } else {
                                CmplSymbol {
                                    const_val: None,
                                    type_val: TypeValEnum::Unit.to_tval(),
                                    var: None,
                                }
                            }
                        },
                        _ => unimplemented!(),
                    }
                } else {
                    CmplSymbol {
                        const_val: None,
                        type_val: TypeValEnum::Unit.to_tval(),
                        var: None,
                    }
                }
            },

            Expr::If(_condition, body, _else_body) => {
                let symbol = self.resolve_expr(body);
                CmplSymbol {
                    const_val: None,
                    type_val: symbol.type_val,
                    var: symbol.var,
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
                        type_val: TypeValEnum::Unit.to_tval(),
                        var: None,
                    }
                }
            }

            Expr::Function(return_expr, return_type, params) => {                
                let symbol = self.resolve_expr(if let Some(return_type) = return_type {
                    return_type
                } else {
                    return_expr
                });

                let is_global_scope = self.has_local_scope();
                let jump_over = self.cprog.count_ir();
                if is_global_scope {
                    self.emit_node(IRNode::JumpFromOffset(0));
                }

                self.open_scope();

                let mut param_types = Vec::new();
                for param in params {
                    match param {
                        Stmt::ConstDecl(name, init, type_expr, _) => {
                            let type_val = if let Some(type_expr) = type_expr {
                                self.resolve_expr(type_expr).type_val
                            } else if let Some(init) = init {
                                self.resolve_expr(init).type_val
                            } else {
                                TypeValEnum::Unit.to_tval()
                            };

                            param_types.push(type_val.clone());
                            self.stack_sz += type_val.size_of();
                            let param_var = Variable {
                                name: name.clone(),
                                type_val,
                                global_pos: None,
                                stack_loc: Some(self.stack_sz),
                                const_val: None,
                                external: None,
                                is_alias: false,
                            };

                            self.add_var(param_var);
                        },
                        _ => unreachable!(),
                    }
                }

                // return address
                self.stack_sz += SIZE_64;

                let address = self.cprog.count_ir();
                self.gen_expr(return_expr);
                
                let params_size = self.close_scope_noclean();
                self.stack_sz -= params_size + (SIZE_64 /* return address */);

                self.pop_to_stack(&symbol.type_val, symbol.type_val.size_of() + (params_size + SIZE_64 /* return address */));
                self.emit_node(IRNode::Return { params_size });

                if is_global_scope {
                    let after_func = self.cprog.count_ir();

                    match self.cprog.node_mut_at(jump_over) {
                        IRNode::JumpFromOffset(offset_ref) => {
                            let offset: i64 = (after_func - jump_over).try_into().unwrap();
                            *offset_ref = offset;
                        },
                        _ => unreachable!(),
                    }
                }

                let function_symbol = CmplSymbol {
                    const_val: Some(CTimeVal::Function {
                        address,
                        return_type_val: symbol.type_val.clone(),
                    }),
                    type_val: TypeValEnum::FunctionPointer(param_types, Box::new(symbol.type_val)).to_tval(),
                    var: None,
                };

                function_symbol
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
        match type_val.as_enum() {
            TypeValEnum::UInt64 => self.emit_node(IRNode::Pop64ToStack(offset)),
            TypeValEnum::FunctionPointer(..) => self.emit_node(IRNode::Pop64ToStack(offset)),
            TypeValEnum::MethodPointer | TypeValEnum::StringSlice => {
                self.emit_node(IRNode::Pop64ToStack(offset));
                self.emit_node(IRNode::Pop64ToStack(offset));
            },

            TypeValEnum::Unit => (),
        }

        self.stack_sz -= type_val.size_of();
    }

    fn global_read_push(&mut self, type_val: &TypeVal, global_pos: usize) {
        match type_val.as_enum() {
            TypeValEnum::UInt64 | TypeValEnum::FunctionPointer(..) => {
                self.emit_node(IRNode::GlobalReadPush64(global_pos));
            },

            TypeValEnum::MethodPointer | TypeValEnum::StringSlice => {
                self.emit_node(IRNode::GlobalReadPush64(global_pos));
                self.emit_node(IRNode::GlobalReadPush64(global_pos + 8));
            },

            TypeValEnum::Unit => (),
        }

        self.stack_sz += type_val.size_of();
    }

    fn external_read_push(&mut self, type_val: &TypeVal, external: ExternalInfo) {
        match type_val.as_enum() {
            TypeValEnum::UInt64 | TypeValEnum::FunctionPointer(..) => {
                self.emit_node(IRNode::ExternalReadPush64(external));
            },

            TypeValEnum::MethodPointer | TypeValEnum::StringSlice => {
                self.emit_node(IRNode::ExternalReadPush64(external));
            },

            TypeValEnum::Unit => (),
        }

        self.stack_sz += type_val.size_of();
    }

    fn stack_read_push(&mut self, type_val: &TypeVal, offset: usize) {
        match type_val.as_enum() {
            TypeValEnum::UInt64 | TypeValEnum::FunctionPointer(..) => {
                self.emit_node(IRNode::StackReadPush64(offset));
            },

            TypeValEnum::MethodPointer | TypeValEnum::StringSlice => {
                self.emit_node(IRNode::StackReadPush64(offset + 8));
                self.emit_node(IRNode::StackReadPush64(offset + 8));
            },

            TypeValEnum::Unit => (),
        }

        self.stack_sz += type_val.size_of();
    }
}

