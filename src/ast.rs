pub const MAX_STR_SIZE: usize = 64;
pub const MAX_VAR_SIZE: usize = 64;

/* converted types from pli/include/parser.h */

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BinaryOp {
    And,
    Or,
    Equal,
    NotEqual,
    Less,
    Greater,
    LessEqual,
    GreaterEqual,
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UnaryOp {
    Negate,
    Not,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LiteralValue {
    Number(f64),
    String(String),
    Boolean(bool),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Binary {
        op: BinaryOp,
        left: Box<Expr>,
        right: Box<Expr>,
        line: i32,
    },
    Unary {
        op: UnaryOp,
        operand: Box<Expr>,
        line: i32,
    },
    Literal {
        value: LiteralValue,
        line: i32,
    },
    Variable {
        name: String,
        line: i32,
    },
    Grouping {
        expression: Box<Expr>,
        line: i32,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    VarDecl {
        var_name: String,
        initializer: Expr,
        line: i32,
    },
    Assignment {
        var_name: String,
        value: Expr,
        line: i32,
    },
    If {
        condition: Expr,
        then_branch: Box<Stmt>,
        else_branch: Option<Box<Stmt>>,
        line: i32,
    },
    While {
        condition: Expr,
        body: Box<Stmt>,
        line: i32,
    },
    Print {
        expressions: Vec<Expr>,
        line: i32,
    },
    Read {
        var_name: String,
        line: i32,
    },
    Block {
        statements: Vec<Stmt>,
        line: i32,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub statements: Vec<Stmt>,
}
