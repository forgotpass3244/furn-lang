
#[derive(Debug)]
pub enum Expr {
    IntLit(u64),
    StringLit(String),
    Block(Vec<Stmt>, Option<Box<Expr>>),
    Call(Box<Expr>, Vec<Expr>),
    Function(Box<Expr>),
    Variable(String),
    // NamespaceAccess(Box<Expr>, Vec<Expr>),
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
}

impl Expr {
    pub fn is_block(&self) -> bool {
        match self {
            Expr::Block(..) => true,
            Expr::Function(expr) => expr.is_block(),
            _ => false,
        }
    }
}

