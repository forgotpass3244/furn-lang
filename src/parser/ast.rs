
// NEVER add cloning to expression enum
#[derive(Debug)]
pub enum Expr {
    IntLit(u64),
    StringLit(String),
    Block(Vec<Stmt>, Option<Box<Expr>>),
    If(Box<Expr>, Box<Expr>, Option<Box<Expr>>), // (condition, body, else_body)
    Call(Box<Expr>, Vec<Expr>),
    Function(Box<Expr>, Option<Box<Expr>>, Vec<Stmt>), // (body, return_type, params)
    Variable(String),
    NamespaceAccess(Box<Expr>, String),
    Reference(Box<Expr>),
    Dereference(Box<Expr>),
    TypeUnit,
    TypeUInt64,
    TypeString,
}

#[derive(Debug)]
pub enum Stmt {
    Expr(Expr, bool), // the bool is whether or not it is the final expr in a block

    // (name, init, type_expr, is_exported)
    ConstDecl(String, Option<Expr>, Option<Expr>, bool),
    VarDecl(String, Option<Expr>, Option<Expr>, bool),

    PackageDecl(String),
    AliasDecl(Option<String>, Expr, bool),
}

impl Expr {
    pub fn is_block(&self) -> bool {
        match self {
            Expr::Block(..) => true,
            Expr::Function(expr, ..) => expr.is_block(),
            
            Expr::If(_condition, body, else_body) => {
                if let Some(else_body) = else_body {
                    else_body.is_block()
                } else {
                    body.is_block()
                }
            },

            _ => false,
        }
    }
}

