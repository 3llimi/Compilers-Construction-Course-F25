use crate::token::Token;

#[derive(Debug, Clone, PartialEq)]
pub enum Program {
    Stmts(Vec<Stmt>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    VarDecl { name: String, init: Expr },
    Assign { target: Expr, value: Expr },
    Print { args: Vec<Expr> },
    If { cond: Expr, then_branch: Vec<Stmt>, else_branch: Option<Vec<Stmt>> },
    While { cond: Expr, body: Vec<Stmt> },
    For { var: String, iterable: Expr, body: Vec<Stmt> },
    Return(Option<Expr>),
    Exit,
    Expr(Expr),
}

#[derive(Debug, Clone, PartialEq)]
pub enum TypeIndicator {
    Int,
    Real,
    Bool,
    String,
    None,
    Array,   // []
    Tuple,   // {}
    Func,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Integer(i64),
    Real(f64),
    Bool(bool),
    None,
    String(String),
    Ident(String),
    Range(Box<Expr>, Box<Expr>),
    Binary { left: Box<Expr>, op: BinOp, right: Box<Expr> },
    Unary { op: UnOp, expr: Box<Expr> },
    Call { callee: Box<Expr>, args: Vec<Expr> },
    Index { target: Box<Expr>, index: Box<Expr> },
    Member { target: Box<Expr>, field: String },
    Array(Vec<Expr>),
    Tuple(Vec<TupleElement>),
    IsType { expr: Box<Expr>, type_ind: TypeIndicator },
    Func { params: Vec<String>, body: FuncBody },
}

#[derive(Debug, Clone, PartialEq)]
pub enum FuncBody {
    Expr(Box<Expr>),         // func(x)=> expr
    Block(Vec<Stmt>),        // func(x) is ... end
}

#[derive(Debug, Clone, PartialEq)]
pub enum BinOp {
    Add, Sub, Mul, Div,
    Eq, Ne, Lt, Le, Gt, Ge,
    And, Or, Xor,
    Is,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnOp {
    Neg,
    Not,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TupleElement {
    pub name: Option<String>,  
    pub value: Expr,
}

// Simple helper for pretty printing tokens in errors
pub fn token_to_string(tok: &Token) -> String {
    format!("{:?}", tok)
}
