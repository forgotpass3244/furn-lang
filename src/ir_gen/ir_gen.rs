use std::collections::{HashMap, VecDeque};

use crate::{ir_gen::{cmpld_program::CompiledProgram, ctimeval::CTimeVal, external::ExternalInfo, global::GlobalInfo, ir::IRNode, lifetime::Lifetime, scope::Scope, symbol::CmplSymbol, typeval::{TypeVal, TypeValEnum}, variable::Variable}, lexer::tokens::SourceLocation, parser::ast::{AstBlock, Expr, ExprEnum, IfKind, Operator, Stmt, StmtEnum}};

const ADDRESS_SIZE: usize = 8;
const SIZE_64: usize = 8;

fn is_pascal_case(name: &String) -> bool {
    if let Some(first_letter) = name.chars().nth(0) {
        if first_letter.is_uppercase() {
            for char in name.chars() {
                if char == '_' {
                    return false
                }
            }
            true
        } else {
            false
        }
    } else {
        false
    }
}

fn as_pascal_case(name: &String) -> String {
    let mut result = String::new();

    let mut iter = name.chars().peekable();
    let mut is_first_char = true;
    while let Some(char) = iter.next() {
        match char {
            '_' => if let Some(next_letter) = iter.clone().next() {
                if next_letter != '_' {
                    iter.next();
                    for char in next_letter.to_uppercase() {
                        result.push(char);
                    }
                }
            },
            other => if is_first_char {
                for char in other.to_uppercase() {
                    result.push(char)
                }
            } else {
                result.push(other)
            },
        }

        is_first_char = false;
    }

    result
}

pub struct IRGen<'a> {
    cprog: CompiledProgram<'a>,
    global_scope: Scope<Variable>,
    scopes: VecDeque<Scope<Variable>>,
    stack_sz: usize,
    global_sz: usize,
    symbol_cache: HashMap<*const Expr, CmplSymbol>,
    lifetime_id_counter: usize,
    err_count: usize,
    unsafe_depth: usize,
    diagnostics_lock: usize,
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
            lifetime_id_counter: 0,
            err_count: 0,
            unsafe_depth: 0,
            diagnostics_lock: 0,
        }
    }

    fn emit_diagnostic(&mut self, loc: &SourceLocation, message: &str) {
        if self.diagnostics_lock > 0 {
            return
        }

        if message.contains("⚠️") {
            println!("(Line {}, Col {}) [Warning]: {}", loc.line, loc.col, message);
        } else {
            self.err_count += 1;
            println!("(Line {}, Col {}) [Error]: ❗ {}", loc.line, loc.col, message);
        }
    }

    pub fn has_errors(&self) -> bool {
        self.err_count > 0
    }

    fn open_scope(&mut self) {
        self.scopes.push_front(Scope::new());
    }

    fn is_unsafe_allowed(&self) -> bool {
        self.unsafe_depth > 0
    }

    fn scope_locals_stackp(&self) -> Option<usize> {
        let mut stack_loc = self.stack_sz;
        if let Some(scope) = self.scopes.front() {
            for var in scope.iter() {
                if !var.is_alias {
                    if let Some(_) = var.stack_loc {
                        stack_loc -= var.typeval.size_of();
                    }
                }
            }
        } else {
            unreachable!()
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
                        locals_size_total += var.typeval.size_of()
                    }
                }
            }

            self.emit_node(IRNode::StackDealloc(locals_size_total));
            self.stack_sz -= locals_size_total;

        } else {
            unreachable!("warning (this shouldn't happen): attempted to close scope but there is are no scopes to close")
        }
    }

    fn close_scope_noclean(&mut self) -> usize {
        if let Some(scope) = self.scopes.pop_front() {
            let mut locals_size_total = 0;

            for var in scope.iter() {
                if !var.is_alias {
                    if let Some(_) = var.stack_loc {
                        locals_size_total += var.typeval.size_of()
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

    fn new_global_pos(&mut self, typeval: &TypeVal) -> usize {
        let prev = self.global_sz;
        self.global_sz += typeval.size_of();
        prev
    }

    pub fn generate(&mut self, ast: &'a Vec<Stmt>) -> &mut CompiledProgram<'a> {
        let mut rt_map = HashMap::new();

        self.cprog.add_external(ExternalInfo::new("print_str".to_string(), "Rt".to_string(), true));
        let print_str = Variable {
            name: "print_str".to_string(),
            typeval: TypeValEnum::FunctionPointer(vec![TypeValEnum::StringSlice.to_tval()], Box::new(TypeValEnum::Unit.to_tval())).to_tval(),
            global_pos: None,
            stack_loc: None,
            const_val: None,
            external: Some(ExternalInfo::new("print_str".to_string(), "Rt".to_string(), true)),
            is_alias: false,
            lifetime: self.lifetime_here(),
            is_unsafe: false,
        };
        rt_map.insert("print_str".to_string(), print_str.clone());

        self.cprog.add_external(ExternalInfo::new("print_newline".to_string(), "Rt".to_string(), true));
        let print_newline = Variable {
            name: "print_newline".to_string(),
            typeval: TypeValEnum::FunctionPointer(vec![], Box::new(TypeValEnum::Unit.to_tval())).to_tval(),
            global_pos: None,
            stack_loc: None,
            const_val: None,
            external: Some(ExternalInfo::new("print_newline".to_string(), "Rt".to_string(), true)),
            is_alias: false,
            lifetime: self.lifetime_here(),
            is_unsafe: false,
        };
        rt_map.insert("print_newline".to_string(), print_newline.clone());

        self.cprog.add_external(ExternalInfo::new("print_unit".to_string(), "Rt".to_string(), true));
        let print_unit = Variable {
            name: "print_unit".to_string(),
            typeval: TypeValEnum::FunctionPointer(vec![TypeValEnum::Unit.to_tval()], Box::new(TypeValEnum::Unit.to_tval())).to_tval(),
            global_pos: None,
            stack_loc: None,
            const_val: None,
            external: Some(ExternalInfo::new("print_unit".to_string(), "Rt".to_string(), true)),
            is_alias: false,
            lifetime: self.lifetime_here(),
            is_unsafe: false,
        };
        rt_map.insert("print_unit".to_string(), print_unit.clone());

        self.cprog.add_external(ExternalInfo::new("print_char".to_string(), "Rt".to_string(), true));
        let print_var = Variable {
            name: "print_char".to_string(),
            typeval: TypeValEnum::FunctionPointer(vec![TypeValEnum::UInt64.to_tval()], Box::new(TypeValEnum::Unit.to_tval())).to_tval(),
            global_pos: None,
            stack_loc: None,
            const_val: None,
            external: Some(ExternalInfo::new("print_char".to_string(), "Rt".to_string(), true)),
            is_alias: false,
            lifetime: self.lifetime_here(),
            is_unsafe: false,
        };
        rt_map.insert("print_char".to_string(), print_var);

        self.cprog.add_external(ExternalInfo::new("print_digit".to_string(), "Rt".to_string(), true));
        let print_var = Variable {
            name: "print_digit".to_string(),
            typeval: TypeValEnum::FunctionPointer(vec![TypeValEnum::UInt64.to_tval()], Box::new(TypeValEnum::Unit.to_tval())).to_tval(),
            global_pos: None,
            stack_loc: None,
            const_val: None,
            external: Some(ExternalInfo::new("print_digit".to_string(), "Rt".to_string(), true)),
            is_alias: false,
            lifetime: self.lifetime_here(),
            is_unsafe: false,
        };
        rt_map.insert("print_digit".to_string(), print_var);
        
        self.cprog.add_external(ExternalInfo::new("print_u64".to_string(), "Rt".to_string(), true));
        let print_u64 = Variable {
            name: "print_u64".to_string(),
            typeval: TypeValEnum::FunctionPointer(vec![TypeValEnum::UInt64.to_tval()], Box::new(TypeValEnum::Unit.to_tval())).to_tval(),
            global_pos: None,
            stack_loc: None,
            const_val: None,
            external: Some(ExternalInfo::new("print_u64".to_string(), "Rt".to_string(), true)),
            is_alias: false,
            lifetime: self.lifetime_here(),
            is_unsafe: false,
        };
        rt_map.insert("print_u64".to_string(), print_u64.clone());

        let mut dynamic_print_map = HashMap::new();

        dynamic_print_map.insert(TypeValEnum::StringSlice.to_tval(), CmplSymbol {
            const_val: None,
            typeval: print_str.typeval.clone(),
            var: Some(print_str),
            lifetime: None,
            is_unsafe: false,
        });

        dynamic_print_map.insert(TypeValEnum::Unit.to_tval(), CmplSymbol {
            const_val: None,
            typeval: print_unit.typeval.clone(),
            var: Some(print_unit),
            lifetime: None,
            is_unsafe: false,
        });

        dynamic_print_map.insert(TypeValEnum::UInt64.to_tval(), CmplSymbol {
            const_val: None,
            typeval: print_u64.typeval.clone(),
            var: Some(print_u64),
            lifetime: None,
            is_unsafe: false,
        });

        let print_var = Variable {
            name: "print".to_string(),
            typeval: TypeValEnum::Unit.to_tval(),
            global_pos: None,
            stack_loc: None,
            const_val: Some(
                CTimeVal::DynamicFnDispatcher {
                    map: dynamic_print_map,
                    meta_funcs: Box::new(
                        (
                            CmplSymbol::void(),
                            CmplSymbol::void(),
                            CmplSymbol {
                                const_val: None,
                                typeval: print_newline.typeval.clone(),
                                var: Some(print_newline),
                                lifetime: None,
                                is_unsafe: false,
                            },
                        )
                    )
                }
            ),
            external: None,
            is_alias: false,
            lifetime: self.lifetime_here(),
            is_unsafe: false,
        };
        self.add_global(print_var);

        let rt_namespace = CTimeVal::Namespace(rt_map);
        let rt_var = Variable {
            name: "Rt".to_string(),
            typeval: TypeValEnum::Unit.to_tval(),
            global_pos: None,
            stack_loc: None,
            const_val: Some(rt_namespace),
            external: None,
            is_alias: false,
            lifetime: self.lifetime_here(),
            is_unsafe: false,
        };

        self.add_global(rt_var);

        for stmt in ast {
            self.gen_stmt(&stmt);
        }

        if self.cprog.get_package_name().is_none() {
            for global in self.cprog.clone().globals_iter() {
                if global.is_exported && global.name != "main" {
                    self.emit_diagnostic(&SourceLocation::garbage(), "unable to export any symbols if a package name was never declared");
                }
            }
        }
        
        &mut self.cprog
    }

    fn emit_node(&mut self, node: IRNode) {
        self.cprog.app_node(node);
    }

    fn gen_stmt(&mut self, stmt: &'a Stmt) {
        match stmt.as_enum() {
            StmtEnum::Expr(expr, _) => {
                let prev_stack_sz = self.stack_sz;
                self.gen_expr(expr);

                let stack_diff = self.stack_sz - prev_stack_sz;
                if stack_diff > 0 {
                    self.emit_node(IRNode::StackDealloc(stack_diff));
                    self.stack_sz -= stack_diff;
                }

            },

            StmtEnum::PackageDecl(name) => {
                if let Some(name) = self.cprog.get_package_name() {
                    self.emit_diagnostic(stmt.get_loc(), format!("package was previously declared as `{name}`").as_str());
                } else {
                    if !is_pascal_case(name) {
                        self.emit_diagnostic(stmt.get_loc(), format!("|BadPackageName| ⚠️ a package name should be PascalCase: `package {};`", as_pascal_case(name)).as_str());
                    }
                    self.cprog.set_package_name(Some(name));
                }
            },

            StmtEnum::AliasDecl(alias_name, expr, is_call) => {
                let var = self.resolve_expr(expr).var;
                if let Some(var) = var {
                    match var.typeval.as_enum() {
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

            StmtEnum::ConstDecl(name, init, type_expr, is_exported) => self.gen_decl(name, init, type_expr, true, *is_exported, stmt.get_loc()),
            StmtEnum::VarDecl(name, init, type_expr, is_exported) => self.gen_decl(name, init, type_expr, false, *is_exported, stmt.get_loc()),
        }
    }

    fn gen_decl(&mut self, name: &'a str, init: &'a Option<Expr>, type_expr: &'a Option<Expr>, is_const: bool, is_exported: bool, loc: &SourceLocation) {

        if self.has_local_scope() && is_exported {
            self.emit_diagnostic(loc, "exports must be global variables");
        }

        let typeval = if let Some(type_expr) = type_expr {
            self.resolve_expr(type_expr).typeval
        } else if let Some(init) = init {
            self.resolve_expr(init).typeval
        } else {
            self.emit_diagnostic(loc, "type cannot be inferred here, defaults to `void`");
            TypeValEnum::Unit.to_tval()
        };

        let is_global_var = !self.has_local_scope();
        let global_pos = if is_global_var { Some(self.new_global_pos(&typeval)) } else { None };

        let mut var = Variable {
            name: name.to_string(),
            typeval: typeval.clone(),
            global_pos,
            stack_loc: None,
            const_val: None,
            external: None,
            is_alias: false,
            lifetime: self.lifetime_here(),
            is_unsafe: self.is_unsafe_allowed(),
        };

        if let Some(expr) = init {

            let symbol = self.resolve_expr(expr);
            if type_expr.is_none() {
                var.typeval = symbol.typeval;
            }

            if !var.is_unsafe && symbol.is_unsafe {
                self.emit_diagnostic(expr.get_loc(), "variable must be in `unsafe` block");
            } else if let Some(other_lifetime) = symbol.lifetime {
                if other_lifetime < var.lifetime {
                    // emit this error even in unsafe blocks
                    self.emit_diagnostic(expr.get_loc(), "lifetime may not live until initialization");
                }
            }

            if symbol.const_val.is_some() && is_const && is_global_var {
                var.const_val = symbol.const_val.clone();
            }
            
            if is_global_var {
                if let Some(const_val) = symbol.const_val {
                    if is_exported {
                        let global_info = GlobalInfo::new(global_pos.unwrap_or_default(), name, is_exported, const_val, is_const);
                        self.cprog.add_global(global_info);
                    }
                } else {
                    self.emit_diagnostic(expr.get_loc(), "global variable must be initialized with a compile-time constant");
                }
            } else if symbol.const_val.is_none() || !is_const || !is_global_var {
                self.gen_expr(expr);
                var.stack_loc = Some(self.stack_sz);
            }

        } else if is_exported {
            self.emit_diagnostic(loc, "exported symbol is not initialized");
        } else if !is_global_var {
            let result = self.push_zeroval(&typeval);
            if let Err(message) = result {
                self.emit_diagnostic(type_expr.as_ref().unwrap().get_loc(), message.as_str());
            }

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
            CTimeVal::Int(int) => {
                self.emit_node(IRNode::Push64(*int as u64));
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

            CTimeVal::DynamicFnDispatcher{ .. } => {
                self.emit_diagnostic(&SourceLocation::garbage(), "an overload set cannot indirectly coerce to a function pointer");
            },

            CTimeVal::Namespace(..) => {
                self.emit_diagnostic(&SourceLocation::garbage(), "cannot generate a namespace as an expression");
            },

            CTimeVal::Type(..) => {
                self.emit_diagnostic(&SourceLocation::garbage(), "cannot generate a type as a const value");
            },
        }
    }

    fn gen_expr(&mut self, expr: &'a Expr) {
        match expr.as_enum() {
            ExprEnum::TypeUInt64 => {
                self.emit_diagnostic(expr.get_loc(), "add compile errors (type `u64` is not allowed as a generatable expression)");
            },

            ExprEnum::TypeString => {
                self.emit_diagnostic(expr.get_loc(), "add compile errors (type `str` is not allowed as a generatable expression)");
            },

            ExprEnum::TypeUnit => {
                // nothing to generate
            },
            
            ExprEnum::IntLit(int) => {
                self.emit_node(IRNode::Push64(*int));
                self.stack_sz += SIZE_64;
            },

            ExprEnum::StringLit(string) => {
                let static_string_pointer = self.new_static_string(string);
                self.emit_node(IRNode::PushStaticStringPointer(static_string_pointer));
                self.emit_node(IRNode::Push64(string.len() as u64));
                self.stack_sz += 16;
            },

            ExprEnum::Dereference(subexpr) => {
                let symbol: CmplSymbol = self.resolve_expr(subexpr);
                if let Some(const_val) = symbol.const_val {
                    match const_val {
                        CTimeVal::Type(_) => {
                            self.emit_diagnostic(expr.get_loc(), "unable to dereference a type");
                        },
                        
                        _ => {
                            // cannot deref to a ctimeval
                            self.emit_diagnostic(expr.get_loc(), "cannot dereference to a compile time constant");
                        },
                    }
                } else if !symbol.typeval.is_ptr() {
                    self.emit_diagnostic(expr.get_loc(), "dereference on a non-pointer type");
                } else {
                    self.gen_expr(subexpr);
                    self.stack_sz -= symbol.typeval.size_of();
                    self.emit_node(IRNode::Deref64);
                    self.stack_sz += SIZE_64;
                }
            },
            
            ExprEnum::Reference(subexpr) => {
                let symbol = self.resolve_expr(subexpr);
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
                } else if let Some(const_val) = symbol.const_val {
                    match const_val {
                        _ => {
                            // cannot grab a reference to a ctimeval
                            self.emit_diagnostic(subexpr.get_loc(), "cannot grab a reference to a compile time constant");
                        },
                    }
                } else {
                    self.emit_diagnostic(expr.get_loc(), "expected a variable to grab a reference to");
                }
            },

            ExprEnum::Call(expr, args) => self.gen_call_expr(expr, args),

            ExprEnum::Variable(name) => {
                let var = self.lookup_var(name).cloned();
                if let Some(var) = var {
                    self.gen_var_read(var);
                } else {
                    self.emit_diagnostic(expr.get_loc(), "variable not found");
                }
            },

            ExprEnum::MemberAccess(expr, name) => {
                let symbol = self.resolve_expr(expr);
                if let Some(const_val) = symbol.const_val {
                    match const_val {
                        CTimeVal::Namespace(map) => {
                            let var = map.get(name);
                            if let Some(var) = var {
                                let typeval = var.typeval.clone();

                                if let Some(const_val) = var.const_val.clone() {
                                    self.gen_const_val(&const_val);
                                } else if let Some(external) = var.external.clone() {
                                    self.external_read_push(&typeval, external.clone());
                                } else {
                                    if let Some(pos) = var.global_pos {
                                        self.global_read_push(&typeval, pos);
                                    } else if let Some(stack_loc) = var.stack_loc {
                                        self.stack_read_push(&typeval, self.stack_sz - stack_loc);
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

            ExprEnum::If(if_kind, body, else_body) => {
                self.open_scope();

                match &(**if_kind) {
                    IfKind::Conditional(condition) => {
                        // let condition_symbol = self.resolve_expr(&condition);
                        self.gen_expr(condition);
                        self.emit_node(IRNode::JumpIfNot64FromOffset(0));
                        self.stack_sz -= SIZE_64;
                    },
                    
                    IfKind::ConstBinding { name, type_expr, init } => {
                        let init_symbol = self.resolve_expr(init);
                        match init_symbol.typeval.as_enum() {
                            TypeValEnum::TaggedUnion(typevals) => {
                                let typeval = self.resolve_expr(type_expr).typeval;
                                let var = Variable {
                                    name: name.clone(),
                                    typeval: typeval.clone(),
                                    global_pos: None,
                                    stack_loc: None,
                                    const_val: None,
                                    external: None,
                                    is_alias: false,
                                    lifetime: self.lifetime_here(),
                                    is_unsafe: false,
                                };

                                self.add_var(var);

                                let top_loc = self.stack_sz;
                                self.gen_expr(init);
                                self.emit_node(IRNode::Pop64ToStack((self.stack_sz - top_loc) - SIZE_64));
                                self.stack_sz = top_loc;

                                let index = typevals.iter().position(|x| *x == typeval);
                                if let Some(index) = index {
                                    self.emit_node(IRNode::JumpIfNotEqConst64FromOffset(index as u64, 0));
                                } else {
                                    self.emit_diagnostic(type_expr.get_loc(), format!("type `{typeval}` is not a variant in tagged union `{}`", init_symbol.typeval).as_str());
                                    self.emit_node(IRNode::JumpIfNotEqConst64FromOffset(0, 0));
                                }
                            },

                            _ => {
                                self.emit_diagnostic(init.get_loc(), format!("type `{}` does not support if bindings", init_symbol.typeval).as_str());
                                self.close_scope();
                                return
                            },
                        }
                    },
                };

                // TODO: make 8 bytes not hardcoded
                let jump_ifn_label = self.cprog.ir_pos();

                self.gen_block(body, false, expr.get_loc());
                let skipover_else_label = self.cprog.count_ir();
                if else_body.is_some() {
                    self.emit_node(IRNode::JumpFromOffset(0));
                }

                let else_label = self.cprog.count_ir() as i64;
                match self.cprog.node_mut_at(jump_ifn_label) {
                    IRNode::JumpIfNot64FromOffset(offset) => {
                        *offset = else_label - (jump_ifn_label as i64);
                    },
                    IRNode::JumpIfNotEqConst64FromOffset(_,offset) => {
                        *offset = else_label - (jump_ifn_label as i64);
                    },
                    _ => unreachable!(),
                }

                if let Some(else_body) = else_body {
                    self.gen_block(else_body, false, expr.get_loc());

                    let after_else_label = self.cprog.ir_pos() as i64;
                    match self.cprog.node_mut_at(skipover_else_label) {
                        IRNode::JumpFromOffset(offset) => {
                            *offset = after_else_label - (skipover_else_label as i64);
                        },
                        _ => unreachable!(),
                    }
                }

                self.close_scope();
            },

            ExprEnum::Block(block, is_unsafe_block) => self.gen_block(block, *is_unsafe_block, expr.get_loc()),

            ExprEnum::Function(..) => {
                self.resolve_expr(expr);
            },

            ExprEnum::BinaryOp { operands, op } => {
                let symbol = self.resolve_expr(expr);

                if let Some(const_val) = symbol.const_val {
                    self.gen_const_val(&const_val);
                } else {

                    match op {
                        Operator::Assign => {
                            let lhs_symbol = self.resolve_expr(&operands.0);
                            let rhs_symbol = self.resolve_expr(&operands.1);
                            if let Some(var) = lhs_symbol.var {
                                if let Some(stack_loc) = var.stack_loc {
                                    if !var.is_unsafe && rhs_symbol.is_unsafe {
                                        self.emit_diagnostic(operands.1.get_loc(), "assignment of an unsafe value to a safe variable");
                                    } else if let Some(other_lifetime) = rhs_symbol.lifetime {
                                        if other_lifetime < var.lifetime {
                                            if !var.is_unsafe && self.is_unsafe_allowed() {
                                                self.emit_diagnostic(operands.0.get_loc(), "variable must be declared in `unsafe` block");
                                            } else if !var.is_unsafe {
                                                self.emit_diagnostic(operands.1.get_loc(), "lifetime may not live long enough");
                                            }
                                        }
                                    }

                                    self.gen_expr(&operands.1);
                                    self.pop_to_stack(&rhs_symbol.typeval, self.stack_sz - stack_loc);
                                }
                            }
                        },
                        
                        Operator::Add => {
                            self.gen_expr(&operands.0);
                            self.gen_expr(&operands.1);
                            self.emit_node(IRNode::Add64);
                            self.stack_sz -= SIZE_64;
                        },

                        Operator::Sub => {
                            self.gen_expr(&operands.0);
                            self.gen_expr(&operands.1);
                            self.emit_node(IRNode::Sub64);
                            self.stack_sz -= SIZE_64;
                        },

                        Operator::BitOr => {
                            unimplemented!()
                        }
                    }

                }
            },
        }
    }

    fn gen_var_read(&mut self, var: Variable) {
        let typeval = var.typeval;

        if let Some(const_val) = var.const_val {
            self.gen_const_val(&const_val);
        } else if let Some(external) = var.external {
            self.external_read_push(&typeval, external.clone());
        } else {
            if let Some(pos) = var.global_pos {
                self.global_read_push(&typeval, pos);
            } else if let Some(stack_loc) = var.stack_loc {
                self.stack_read_push(&typeval, self.stack_sz - stack_loc);
            } else {
                let size = typeval.size_of();
                self.emit_node(IRNode::StackAlloc(size));
                self.stack_sz += size;
                self.emit_diagnostic(&SourceLocation::garbage(), "variable is invalid and cannot be read");
            }
        }
    }

    fn gen_call_expr(&mut self, expr: &'a Expr, args: &'a Vec<Expr>) {

        // non-recursive solution
        let init_symbol = self.resolve_expr(expr);
        let mut queue: VecDeque<(CmplSymbol, bool, bool)> = vec![(init_symbol, false, true)].into();
        // (CmplSymbol, should_drop_result, should_use_args)

        loop {
            if let Some(item) = queue.pop_front() {
                let (symbol, should_drop_result, should_use_args) = item;

                match symbol.const_val {
                    Some(CTimeVal::DynamicFnDispatcher { map, meta_funcs }) => {
                        let first_typeval = self.resolve_expr(args.first().unwrap()).typeval;
                        let selected = map.get(&first_typeval);

                        let selected_symbol = if let Some(selected) = selected {
                            selected.clone()
                        } else {
                            self.emit_diagnostic(expr.get_loc(), format!("no function overload for type `{first_typeval}`", ).as_str());
                            CmplSymbol {
                                const_val: None,
                                typeval: TypeValEnum::Unit.to_tval(),
                                var: None,
                                lifetime: None,
                                is_unsafe: false,
                            }
                        };
                        
                        queue.push_back((selected_symbol, false, true));

                        // run this AFTER
                        queue.push_back((meta_funcs.2, true, false));
                    }

                    _ => match symbol.typeval.as_enum() {
                        TypeValEnum::FunctionPointer(_param_typevals, return_typeval) => {
                            // alloc return value
                            self.emit_node(IRNode::StackAlloc(return_typeval.size_of()));
                            self.stack_sz += return_typeval.size_of();

                            // push args
                            let prev_stack_sz = self.stack_sz;
                            if should_use_args {
                                for arg in args {
                                    self.gen_expr(arg);
                                }
                            }
                            
                            // actually perform the function call
                            if let Some(var) = &symbol.var {
                                self.gen_var_read(var.clone());
                            } else {
                                self.gen_expr(expr);
                            }
                            
                            self.emit_node(IRNode::Call);
                            self.stack_sz = if should_drop_result {
                                self.emit_node(IRNode::StackDealloc(return_typeval.size_of()));
                                prev_stack_sz - return_typeval.size_of()
                            } else {
                                prev_stack_sz + return_typeval.size_of()
                            };
                        },

                        TypeValEnum::TaggedUnion(typevals) => {
                            assert!(should_use_args);
                            assert!(args.len() == 1);
                            let typeval = symbol.typeval.clone();
                            let arg = args.first().unwrap();
                            let arg_symbol = self.resolve_expr(arg);

                            let mut matched_index = 0;
                            let mut matched_typeval = None;
                            for (i, typeval) in typevals.iter().enumerate() {
                                if arg_symbol.typeval == *typeval {
                                    matched_typeval = Some(typeval);
                                    matched_index = i;
                                    break;
                                }
                            }

                            let size = typeval.size_of();
                            if should_drop_result {
                                let prev_stack_sz = self.stack_sz;
                                self.gen_expr(arg);
                                self.emit_node(IRNode::StackDealloc(self.stack_sz - prev_stack_sz));
                                self.stack_sz -= self.stack_sz - prev_stack_sz;
                            } else if let Some(matched_typeval) = matched_typeval {
                                let extra_union_space = (size - SIZE_64 /*tag*/) - matched_typeval.size_of();
                                self.emit_node(IRNode::StackAlloc(extra_union_space)); // extra unused union space for other variants
                                self.stack_sz += extra_union_space;
                                self.gen_expr(arg);
                                self.emit_node(IRNode::Push64(matched_index as u64));
                                self.stack_sz += SIZE_64;
                            } else {
                                self.emit_node(IRNode::StackAlloc(size));
                                self.stack_sz += size;
                                self.emit_diagnostic(expr.get_loc(), format!("no variant `{}` in tagged union `{typeval}`", arg_symbol.typeval).as_str());
                            }
                        },

                        TypeValEnum::Unit => {
                            // push args
                            let prev_stack_sz = self.stack_sz;
                            if should_use_args {
                                for arg in args {
                                    self.gen_expr(arg);
                                }
                            }

                            self.emit_node(IRNode::StackDealloc(self.stack_sz - prev_stack_sz));
                            self.stack_sz = prev_stack_sz;
                        },

                        _ => {
                            self.emit_diagnostic(expr.get_loc(), "cannot perform call on this type");
                        },
                    }
                }
            } else {
                break
            }
        }
    }

    fn gen_block(&mut self, block: &'a AstBlock, is_unsafe_block: bool, loc: &SourceLocation) {

        self.emit_node(IRNode::Nop); // ensures that nothing points to the wrong place when the optimizer changes addresses

        if is_unsafe_block && self.is_unsafe_allowed() {
            self.emit_diagnostic(loc, "unnecessary `unsafe` block");
        }
        
        // skip block generation if its just a return expr
        if block.body.is_empty() {
            if let Some(return_expr) = &block.return_expr {
                self.unsafe_depth += is_unsafe_block as usize;
                self.gen_expr(return_expr);
                self.unsafe_depth -= is_unsafe_block as usize;
                return
            }
        }

        self.open_scope();
        self.unsafe_depth += is_unsafe_block as usize;

        let prev_stack_sz = self.stack_sz;

        // alloc for return (final expr)
        let stack_alloc_label = self.cprog.count_ir();
        self.emit_node(IRNode::StackAlloc(0));

        for stmt in &block.body {
            self.gen_stmt(stmt);
        }

        if let Some(return_expr) = &block.return_expr {
            self.gen_expr(return_expr);
        }

        if let Some(return_expr) = &block.return_expr {
            let typeval = self.resolve_expr(return_expr).typeval;
            let return_size = typeval.size_of();
            let return_typeval = Some(typeval);

            // alloc for return (final expr)
            match self.cprog.node_mut_at(stack_alloc_label) {
                IRNode::StackAlloc(size) => {
                    *size = return_size;
                },
                _ => unreachable!(),
            }

            self.cprog.realign_stack_offsets(stack_alloc_label, self.stack_sz - prev_stack_sz, return_size);
            self.stack_sz += return_size;

            if let Some(typeval) = return_typeval {
                match self.scope_locals_stackp() {
                    Some(locals_begin_stackp) => self.pop_to_stack(&typeval, (self.stack_sz - locals_begin_stackp) + return_size),
                    None => self.pop_to_stack(&typeval, return_size),
                }
            }
        }

        self.close_scope();
        self.unsafe_depth -= is_unsafe_block as usize;
    }

    fn resolve_expr(&mut self, expr: &'a Expr) -> CmplSymbol {
        match expr.as_enum() {
            ExprEnum::Function(..) => self.resolve_expr_cached(expr),
            ExprEnum::Block(..) => self.resolve_expr_cached(expr),

            // these may emit compile errors
            // so cache to avoid duplicate errors
            ExprEnum::Dereference(..) => self.resolve_expr_cached(expr),
            ExprEnum::Reference(..) => self.resolve_expr_cached(expr),
            ExprEnum::Variable(..) => self.resolve_expr_cached(expr),

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
        match expr.as_enum() {
            ExprEnum::TypeUInt64 => CmplSymbol {
                const_val: Some(CTimeVal::Type(TypeValEnum::UInt64.to_tval())),
                typeval: TypeValEnum::UInt64.to_tval(),
                var: None,
                lifetime: None,
                is_unsafe: false,
            },

            ExprEnum::TypeString => CmplSymbol {
                const_val: Some(CTimeVal::Type(TypeValEnum::StringSlice.to_tval())),
                typeval: TypeValEnum::StringSlice.to_tval(),
                var: None,
                lifetime: None,
                is_unsafe: false,
            },

            ExprEnum::TypeUnit => CmplSymbol::void(),
            
            ExprEnum::IntLit(int) => CmplSymbol {
                const_val: Some(CTimeVal::Int(*int as i128)),
                typeval: TypeValEnum::UInt64.to_tval(),
                var: None,
                lifetime: None,
                is_unsafe: false,
            },

            ExprEnum::StringLit(string) => CmplSymbol {
                const_val: Some(CTimeVal::StringSlice(self.new_static_string(string), string.len())),
                typeval: TypeValEnum::StringSlice.to_tval(),
                var: None,
                lifetime: None,
                is_unsafe: false,
            },

            ExprEnum::Dereference(subexpr) => {
                let symbol = self.resolve_expr(subexpr);
                if let Some(const_val) = symbol.const_val {
                    match const_val {
                        // to ref type
                        CTimeVal::Type(typeval) => {
                            
                            CmplSymbol {
                                const_val: Some(CTimeVal::Type(typeval.clone().to_ptr())),
                                typeval: typeval.to_ptr(),
                                var: None,
                                lifetime: None,
                                is_unsafe: false,
                            }
                        },

                        // cannot dereference to a ctimeval
                        _ => CmplSymbol {
                            const_val: None,
                            typeval: TypeValEnum::Unit.to_tval(),
                            var: None,
                            lifetime: None,
                            is_unsafe: false,
                        },
                    }
                } else {
                    CmplSymbol {
                        const_val: None,
                        typeval: symbol.typeval.to_lessptr(),
                        var: None,
                        lifetime: None,
                        is_unsafe: false,
                    }
                }
            },
            
            ExprEnum::Reference(subexpr) => {
                let symbol = self.resolve_expr(subexpr);
                if let Some(const_val) = symbol.const_val {
                    match const_val {
                        CTimeVal::Type(..) => {
                            self.emit_diagnostic(expr.get_loc(), "unable to grab a reference to a type (did you mean: `*u64`)");
                            CmplSymbol {
                                const_val: Some(CTimeVal::Type(symbol.typeval.clone().to_ptr())),
                                typeval: symbol.typeval.to_ptr(),
                                var: None,
                                lifetime: symbol.lifetime,
                                is_unsafe: false,
                            }
                        },
                        
                        // cannot grab a reference to a ctimeval
                        _ => CmplSymbol {
                            const_val: None,
                            typeval: symbol.typeval.to_ptr(),
                            var: None,
                            lifetime: symbol.lifetime,
                            is_unsafe: false,
                        },
                    }
                } else {
                    // ref to
                    CmplSymbol {
                        const_val: None,
                        typeval: symbol.typeval.to_ptr(),
                        var: None,
                        lifetime: if let Some(var) = symbol.var {
                            Some(var.lifetime)
                        } else { None },
                        is_unsafe: symbol.is_unsafe,
                    }
                }
            },

            ExprEnum::Call(function, args) => {
                let symbol = self.resolve_expr(function);
                self.resolve_call(&symbol, args)
            },

            ExprEnum::Variable(name) => {
                let var = self.lookup_var(name);
                if let Some(var) = var {
                    CmplSymbol {
                        const_val: var.const_val.clone(),
                        typeval: var.typeval.clone(),
                        var: Some(var.clone()),
                        lifetime: if var.typeval.is_ptr() { Some(var.lifetime.clone()) } else { None },
                        is_unsafe: var.is_unsafe,
                    }
                } else {
                    self.emit_diagnostic(expr.get_loc(), "⚠️ unable to resolve this variable (type is void)");
                    CmplSymbol {
                        const_val: None,
                        typeval: TypeValEnum::Unit.to_tval(),
                        var: None,
                        lifetime: None,
                        is_unsafe: false,
                    }
                }
            },

            ExprEnum::MemberAccess(namespace_expr, name) => {
                let namespace_symbol = self.resolve_expr(namespace_expr);
                if let Some(const_val) = namespace_symbol.const_val {
                    match const_val {
                        CTimeVal::Namespace(map) => {
                            let var = map.get(name);
                            if let Some(var) = var {
                                CmplSymbol {
                                    const_val: var.const_val.clone(),
                                    typeval: var.typeval.clone(),
                                    var: Some(var.clone()),
                                    lifetime: None,
                                    is_unsafe: false,
                                }
                            } else {
                                CmplSymbol {
                                    const_val: None,
                                    typeval: TypeValEnum::Unit.to_tval(),
                                    var: None,
                                    lifetime: None,
                                    is_unsafe: false,
                                }
                            }
                        },
                        _ => unimplemented!(),
                    }
                } else {
                    CmplSymbol {
                        const_val: None,
                        typeval: TypeValEnum::Unit.to_tval(),
                        var: None,
                        lifetime: None,
                        is_unsafe: false,
                    }
                }
            },

            ExprEnum::If(_condition, body, _else_body) => {
                let symbol = self.resolve_block(body);
                CmplSymbol {
                    const_val: None,
                    typeval: symbol.typeval,
                    var: symbol.var,
                    lifetime: symbol.lifetime,
                    is_unsafe: false,
                }
            },

            ExprEnum::Block(block, _is_unsafe_block) => self.resolve_block(block),

            ExprEnum::Function(body, return_type, params) => {                
                let symbol = if let Some(return_type) = return_type {
                    self.resolve_expr(return_type)
                } else {
                    self.resolve_block(body)
                };

                let is_global_scope = self.has_local_scope();
                let jump_over = self.cprog.count_ir();
                if is_global_scope {
                    self.emit_node(IRNode::JumpFromOffset(0));
                }

                self.open_scope();

                let mut param_types = Vec::new();
                for param in params {
                    match param.as_enum() {
                        StmtEnum::ConstDecl(name, init, type_expr, _) => {
                            let typeval = if let Some(type_expr) = type_expr {
                                self.resolve_expr(type_expr).typeval
                            } else if let Some(init) = init {
                                self.resolve_expr(init).typeval
                            } else {
                                TypeValEnum::Unit.to_tval()
                            };

                            param_types.push(typeval.clone());
                            self.stack_sz += typeval.size_of();
                            let param_var = Variable {
                                name: name.clone(),
                                typeval,
                                global_pos: None,
                                stack_loc: Some(self.stack_sz),
                                const_val: None,
                                external: None,
                                is_alias: false,
                                lifetime: self.make_lifetime(1),
                                is_unsafe: false,
                            };

                            self.add_var(param_var);
                        },
                        _ => unreachable!(),
                    }
                }

                // return address
                self.stack_sz += SIZE_64;

                let address = self.cprog.count_ir();
                self.gen_block(body, false, expr.get_loc());
                
                let params_size = self.close_scope_noclean();
                self.stack_sz -= params_size + (SIZE_64 /* return address */);

                self.pop_to_stack(&symbol.typeval, symbol.typeval.size_of() + (params_size + SIZE_64 /* return address */));
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
                        return_typeval: symbol.typeval.clone(),
                    }),
                    typeval: TypeValEnum::FunctionPointer(param_types, Box::new(symbol.typeval)).to_tval(),
                    var: None,
                    lifetime: None,
                    is_unsafe: false,
                };

                function_symbol
            },

            ExprEnum::BinaryOp { operands, op } => {
                let operands = &(**operands);
                match op {
                    Operator::Assign => {
                        CmplSymbol {
                            const_val: None,
                            typeval: TypeValEnum::Unit.to_tval(),
                            var: None,
                            lifetime: None,
                            is_unsafe: false,
                        }
                    },

                    Operator::Add | Operator::Sub => {
                        let lhs_symbol = self.resolve_expr(&operands.0);
                        let rhs_symbol = self.resolve_expr(&operands.1);

                        // calculate at ctime if both operands are ctimevals
                        let calculate = |x: i128, y: i128| match op {
                            Operator::Add => x + y,
                            Operator::Sub => x - y,
                            _ => unreachable!(),
                        };

                        let const_val = unsafe {
                            if lhs_symbol.const_val.is_some() && rhs_symbol.const_val.is_some() {
                                let const_operands = (
                                    lhs_symbol.const_val.unwrap_unchecked(),
                                    rhs_symbol.const_val.unwrap_unchecked()
                                );

                                match const_operands {
                                    (CTimeVal::Int(x), CTimeVal::Int(y)) => {
                                        let result = calculate(x, y);
                                        Some(CTimeVal::Int(result))
                                    },
                                    _ => None,
                                }
                            } else {
                                None
                            }
                        };

                        CmplSymbol {
                            const_val,
                            typeval: TypeValEnum::UInt64.to_tval(),
                            var: None,
                            lifetime: None,
                            is_unsafe: false,
                        }
                    },

                    Operator::BitOr => {
                        let lhs_symbol = self.resolve_expr(&operands.0);
                        let rhs_symbol = self.resolve_expr(&operands.1);

                        let new_typeval = Self::mix_tagged_unions(&lhs_symbol.typeval, &rhs_symbol.typeval);
                        CmplSymbol {
                            const_val: Some(CTimeVal::Type(new_typeval.clone())),
                            typeval: new_typeval,
                            var: None,
                            lifetime: None,
                            is_unsafe: false,
                        }
                    }
                }
            },
        }
    }

    fn resolve_call(&mut self, symbol: &CmplSymbol, args: &'a Vec<Expr>) -> CmplSymbol {
        match &symbol.const_val {
            Some(CTimeVal::DynamicFnDispatcher { map, meta_funcs: _ }) => {
                let selected = map.get(&self.resolve_expr(args.first().unwrap()).typeval);
                // synthesize a call
                if let Some(selected) = selected {
                    self.resolve_call(selected, args)
                } else {
                    CmplSymbol {
                        const_val: None,
                        typeval: TypeValEnum::Unit.to_tval(),
                        var: None,
                        lifetime: None,
                        is_unsafe: false,
                    }
                }
            },
            
            _ => match symbol.typeval.as_enum() {
                TypeValEnum::FunctionPointer(_param_typevals, return_typeval) => CmplSymbol {
                    const_val: None,
                    typeval: *return_typeval.clone(),
                    var: None,
                    lifetime: None,
                    is_unsafe: false,
                },

                TypeValEnum::TaggedUnion(..) => {
                    let typeval = symbol.typeval.clone();
                    CmplSymbol {
                        const_val: Some(CTimeVal::Type(typeval.clone())),
                        typeval: typeval,
                        var: None,
                        lifetime: None,
                        is_unsafe: false,
                    }
                },

                _ => CmplSymbol {
                    const_val: None,
                    typeval: TypeValEnum::Unit.to_tval(),
                    var: None,
                    lifetime: None,
                    is_unsafe: false,
                },
            },
        }
    }

    fn resolve_block(&mut self, block: &'a AstBlock) -> CmplSymbol {
        if let Some(return_expr) = &block.return_expr {
            // skip block resolving if its just a return expr
            if block.body.is_empty() {
                let symbol = self.resolve_expr(return_expr);
                return symbol
            }

            // return value might be a variable
            // that is only declared inside this block
            // so we must rollback changes after
            let prev_ir_count = self.cprog.count_ir();

            self.diagnostics_lock += 1;
            self.open_scope();

            for stmt in &block.body {
                match stmt.as_enum() {
                    StmtEnum::ConstDecl(..) => self.gen_stmt(stmt),
                    StmtEnum::VarDecl(..) => self.gen_stmt(stmt),
                    _ => (),
                }
            }

            let symbol = self.resolve_expr(return_expr);

            self.close_scope();
            self.diagnostics_lock -= 1;

            self.cprog.shift_nodes(prev_ir_count..=(self.cprog.ir_pos()));

            symbol
        } else {
            CmplSymbol {
                const_val: None,
                typeval: TypeValEnum::Unit.to_tval(),
                var: None,
                lifetime: None,
                is_unsafe: false,
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
    fn push_zeroval(&mut self, typeval: &TypeVal) -> Result<(), String> {
        self.stack_sz += typeval.size_of();
        match typeval.as_enum() {
            TypeValEnum::Pointer(..) => {
                self.emit_node(IRNode::Push64(0));
                Err(format!("zeroval for a pointer type `{typeval}` implies null pointers"))
            },

            TypeValEnum::TaggedUnion(..) => {
                self.emit_node(IRNode::Push64(0));
                Err(format!("no default for variant type `{typeval}`"))
            },

            TypeValEnum::MethodPointer
            | TypeValEnum::StringSlice
            | TypeValEnum::FunctionPointer(..) => {
                self.emit_node(IRNode::Push64(0));
                self.emit_node(IRNode::Push64(0));
                Err(format!("zeroval for `{typeval}` implies null pointers"))
            },

            TypeValEnum::UInt64 => {
                self.emit_node(IRNode::Push64(0));
                Ok(())
            },

            TypeValEnum::Unit => Ok(()), // nothing to push
        }
    }

    fn pop_to_stack(&mut self, typeval: &TypeVal, offset: usize) {
        match typeval.as_enum() {
            TypeValEnum::Pointer(..) => self.emit_node(IRNode::Pop64ToStack(offset)),
            TypeValEnum::UInt64 => self.emit_node(IRNode::Pop64ToStack(offset)),
            TypeValEnum::FunctionPointer(..) => self.emit_node(IRNode::Pop64ToStack(offset)),
            TypeValEnum::MethodPointer | TypeValEnum::StringSlice => {
                self.emit_node(IRNode::Pop64ToStack(offset));
                self.emit_node(IRNode::Pop64ToStack(offset));
            },

            TypeValEnum::TaggedUnion(_typevals) => todo!(),
            TypeValEnum::Unit => {},
        }

        self.stack_sz -= typeval.size_of();
    }

    fn global_read_push(&mut self, typeval: &TypeVal, global_pos: usize) {
        match typeval.as_enum() {
            TypeValEnum::UInt64
            | TypeValEnum::FunctionPointer(..)
            | TypeValEnum::Pointer(..) => {
                self.emit_node(IRNode::GlobalReadPush64(global_pos));
            },

            TypeValEnum::MethodPointer | TypeValEnum::StringSlice => {
                self.emit_node(IRNode::GlobalReadPush64(global_pos));
                self.emit_node(IRNode::GlobalReadPush64(global_pos + 8));
            },

            TypeValEnum::TaggedUnion(_typevals) => todo!(),
            TypeValEnum::Unit => {},
        }

        self.stack_sz += typeval.size_of();
    }

    fn external_read_push(&mut self, typeval: &TypeVal, external: ExternalInfo) {
        match typeval.as_enum() {
            TypeValEnum::UInt64
            | TypeValEnum::FunctionPointer(..)
            | TypeValEnum::Pointer(..) => {
                self.emit_node(IRNode::ExternalReadPush64(external));
            },

            TypeValEnum::MethodPointer | TypeValEnum::StringSlice => {
                self.emit_node(IRNode::ExternalReadPush64(external));
            },

            TypeValEnum::TaggedUnion(_typevals) => todo!(),
            TypeValEnum::Unit => {},
        }

        self.stack_sz += typeval.size_of();
    }

    fn stack_read_push(&mut self, typeval: &TypeVal, offset: usize) {
        match typeval.as_enum() {
            TypeValEnum::UInt64
            | TypeValEnum::FunctionPointer(..)
            | TypeValEnum::Pointer(..) => {
                self.emit_node(IRNode::StackReadPush64(offset));
            },

            TypeValEnum::MethodPointer | TypeValEnum::StringSlice => {
                self.emit_node(IRNode::StackReadPush64(offset + 8));
                self.emit_node(IRNode::StackReadPush64(offset + 8));
            },

            TypeValEnum::TaggedUnion(typevals) => {
                // for tagged unions we also need to push the tag
                let greatest = TypeVal::greatest(typevals);
                self.stack_read_push(&greatest, offset + /*tag*/ SIZE_64);
                self.emit_node(IRNode::StackReadPush64(offset + 16)); // tag
            },
            
            TypeValEnum::Unit => {},
        }

        self.stack_sz += typeval.size_of();
    }
}

impl IRGen<'_> {
    fn make_lifetime(&mut self, scope_depth_offset: isize) -> Lifetime {
        let lifetime = Lifetime::new(self.lifetime_id_counter, self.scopes.len() as isize + scope_depth_offset);
        self.lifetime_id_counter += 1;
        lifetime
    }

    fn lifetime_here(&mut self) -> Lifetime {
        self.make_lifetime(0)
    }

    fn mix_tagged_unions(lhs: &TypeVal, rhs: &TypeVal) -> TypeVal {
        let mut types: Vec<TypeVal> = Vec::new();

        let mut push_unique = |t: TypeVal| {
            if !types.contains(&t) {
                types.push(t);
            }
        };

        match lhs.as_enum() {
            TypeValEnum::TaggedUnion(inner) => {
                for t in inner {
                    push_unique(t.clone());
                }
            }
            _ => push_unique(lhs.clone()),
        }

        match rhs.as_enum() {
            TypeValEnum::TaggedUnion(inner) => {
                for t in inner {
                    push_unique(t.clone());
                }
            }
            _ => push_unique(rhs.clone()),
        }

        TypeValEnum::TaggedUnion(types).to_tval()
    }
}

