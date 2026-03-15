use crate::lexer::tokens::SourceLocation;


#[derive(Debug, Clone, Copy)]
pub enum Operator {
    Add,
    Sub,
    Assign,
    BitOr,
}

impl Operator {
    pub fn precedence(&self) -> u8 {
        match self {
            // Operator::Mul | Operator::Div => 2,
            Operator::Add | Operator::Sub | Operator::BitOr => 1,
            Operator::Assign => 0,
        }
    }
}

// NEVER add cloning to expression enum
#[derive(Debug, Clone)]
pub enum ExprEnum {
    IntLit(u64),
    StringLit(String),
    Block(AstBlock, bool),// (body, return_expr, is_unsafe_block)
    If(Box<Expr>, AstBlock, Option<AstBlock>), // (condition, body, else_body)
    Call(Box<Expr>, Vec<Expr>),
    Function(AstBlock, Option<Box<Expr>>, Vec<Stmt>), // (body, return_type, params)
    Variable(String),
    MemberAccess(Box<Expr>, String),
    Reference(Box<Expr>),
    Dereference(Box<Expr>),
    BinaryOp { operands: Box<(Expr, Expr)>, op: Operator },
    TypeUnit,
    TypeUInt64,
    TypeString,
}

#[derive(Debug, Clone)]
pub struct Expr {
    e_enum: ExprEnum,
    loc: SourceLocation,
}

impl Expr {
    pub fn as_enum(&self) -> &ExprEnum {
        &self.e_enum
    }

    pub fn get_loc(&self) -> &SourceLocation {
        &self.loc
    }
}

#[derive(Debug, Clone)]
pub enum StmtEnum {
    Expr(Expr, bool), // the bool is whether or not it is the final expr in a block

    // (name, init, type_expr, is_exported)
    ConstDecl(String, Option<Expr>, Option<Expr>, bool),
    VarDecl(String, Option<Expr>, Option<Expr>, bool),

    PackageDecl(String),
    AliasDecl(Option<String>, Expr, bool),
}

#[derive(Debug, Clone)]
pub struct Stmt {
    s_enum: StmtEnum,
    loc: SourceLocation,
}

impl Stmt {
    pub fn as_enum(&self) -> &StmtEnum {
        &self.s_enum
    }

    pub fn get_loc(&self) -> &SourceLocation {
        &self.loc
    }
}

impl Expr {
    pub fn is_block(&self) -> bool {
        match self.as_enum() {
            ExprEnum::Block(..) => true,
            ExprEnum::Function(..) => true,
            ExprEnum::If(..) => true,
            _ => false,
        }
    }
}

impl ExprEnum {
    pub fn to_expr(self, loc: SourceLocation) -> Expr {
        Expr {
            e_enum: self,
            loc,
        }
    }
}

impl StmtEnum {
    pub fn to_stmt(self, loc: SourceLocation) -> Stmt {
        Stmt {
            s_enum: self,
            loc,
        }
    }
}

#[derive(Debug, Clone)]
pub struct AstBlock {
    pub body: Vec<Stmt>,
    pub return_expr: Option<Box<Expr>>,
}

