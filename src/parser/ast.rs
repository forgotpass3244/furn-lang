
#[derive(Debug)]
pub enum Expr {
    IntLit(u64),
    Block(Vec<Stmt>, Option<Box<Expr>>),
    Function(Box<Expr>),
    Variable(String),
}

#[derive(Debug)]
pub enum Stmt {
    Expr(Expr),
    ConstDecl(String, Option<Expr>, bool),
    VarDecl(String, Option<Expr>, bool),
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

