use std::ffi::{c_char, c_int, c_void};
use std::os::raw::c_char as raw_c_char;

use crate::ast::{BinaryOp, Expr, LiteralValue, Program, Stmt, UnaryOp};

mod c_ast {
    include!(concat!(env!("OUT_DIR"), "/c_ast_binds.rs"));
}

use c_ast:: {expr_node, stmt_node, program_t};

/* declaring c-funcs */
unsafe extern "C" {
    pub fn atexit_registration() -> c_int;
    pub fn emergency_cleanup();
    pub fn get_ast(program: *mut c_char) -> *mut c_void;
}

fn c_str_to_string(arr: &[raw_c_char; 64]) -> String {
    let ptr = arr.as_ptr() as *const std::ffi::c_char;
    unsafe { std::ffi::CStr::from_ptr(ptr) }
        .to_string_lossy()
        .into_owned()
}

fn convert_binary_op(op: u32) -> BinaryOp {
    match op {
        0 => BinaryOp::And,
        1 => BinaryOp::Or,
        2 => BinaryOp::Equal,
        3 => BinaryOp::NotEqual,
        4 => BinaryOp::Less,
        5 => BinaryOp::Greater,
        6 => BinaryOp::LessEqual,
        7 => BinaryOp::GreaterEqual,
        8 => BinaryOp::Add,
        9 => BinaryOp::Subtract,
        10 => BinaryOp::Multiply,
        11 => BinaryOp::Divide,
        12 => BinaryOp::Modulo,
        _ => BinaryOp::Add,
    }
}

fn convert_unary_op(op: u32) -> UnaryOp {
    match op {
        0 => UnaryOp::Negate,
        1 => UnaryOp::Not,
        _ => UnaryOp::Negate,
    }
}

unsafe fn convert_expr(ptr: *mut expr_node) -> Expr {
    if ptr.is_null() {
        panic!("null expr_node");
    }
    let exp_node = unsafe { &*ptr };
    let line = exp_node.line;

    unsafe {
        match exp_node.type_ {
            0 => {
                /* EXPR_BINARY */
                let b = &*exp_node.expr.binary;
                Expr::Binary {
                    op: convert_binary_op(b.op),
                    left: Box::new(convert_expr(b.left)),
                    right: Box::new(convert_expr(b.right)),
                    line,
                }
            }
            1 => {
                /* EXPR_UNARY */
                let u = &*exp_node.expr.unary;
                Expr::Unary {
                    op: convert_unary_op(u.op),
                    operand: Box::new(convert_expr(u.operand)),
                    line,
                }
            }
            2 => {
                /* EXPR_LITERAL */
                let lit = &*exp_node.expr.literal;
                let value = match lit.type_ {
                    0 => LiteralValue::Number(lit.value.number),
                    1 => LiteralValue::String(c_str_to_string(&lit.value.string)),
                    2 => LiteralValue::Boolean(lit.value.boolean),
                    _ => LiteralValue::Number(0.0),
                };
                Expr::Literal { value, line }
            }
            3 => {
                /* EXPR_VARIABLE */
                let v = &*exp_node.expr.variable;
                Expr::Variable {
                    name: c_str_to_string(&v.name),
                    line,
                }
            }
            4 => {
                /* EXPR_GROUPING */
                let g = &*exp_node.expr.grouping;
                Expr::Grouping {
                    expression: Box::new(convert_expr(g.expression)),
                    line,
                }
            }

            _ => panic!("unknown expr type"),
        }
    }
}

unsafe fn convert_stmt(ptr: *mut stmt_node) -> Stmt {
    if ptr.is_null() {
        panic!("null stmt_node");
    }
    let node = unsafe { &*ptr };
    let line = node.line;

    unsafe {
    match node.type_ {
            0 => {
                /* STMT_VAR_DECL */
                let s = &*node.as_.var_decl;
                Stmt::VarDecl {
                    var_name: c_str_to_string(&s.var_name),
                    initializer: convert_expr(s.initializer),
                    line,
                }
            }
            1 => {
                /* STMT_ASSIGNMENT */
                let s = &*node.as_.assignment;
                Stmt::Assignment {
                    var_name: c_str_to_string(&s.var_name),
                    value: convert_expr(s.value),
                    line,
                }
            }
            2 => {
                /* STMT_IF */
                let s = &*node.as_.if_stmt;
                Stmt::If {
                    condition: convert_expr(s.condition),
                    then_branch: Box::new(convert_stmt(s.then_branch)),
                    else_branch: if s.else_branch.is_null() {
                        None
                    } else {
                        Some(Box::new(convert_stmt(s.else_branch)))
                    },
                    line,
                }
            }
            3 => {
                /* STMT_WHILE */
                let s = &*node.as_.while_stmt;
                Stmt::While {
                    condition: convert_expr(s.condition),
                    body: Box::new(convert_stmt(s.body)),
                    line,
                }
            }
            4 => {
                /* STMT_PRINT */
                let s = &*node.as_.print_stmt;
                let mut expressions = Vec::new();
                for i in 0..s.expr_count {
                    let expr_ptr = *s.expressions.add(i as usize);
                    expressions.push(convert_expr(expr_ptr));
                }
                Stmt::Print { expressions, line }
            }
            5 => {
                /* STMT_READ */
                let s = &*node.as_.read_stmt;
                Stmt::Read {
                    var_name: c_str_to_string(&s.var_name),
                    line,
                }
            }
            6 => {
                /* STMT_BLOCK */
                let s = &*node.as_.block;
                let mut statements = Vec::new();
                let mut stmt_ptr = s.statements;
                for _ in 0..s.stmt_count {
                    if !stmt_ptr.is_null() {
                        statements.push(convert_stmt(stmt_ptr));
                        stmt_ptr = (*stmt_ptr).next;
                    }
                }
                Stmt::Block { statements, line }
            }

            _ => panic!("unknown stmt type"),
        }
    }
}

/* convert c-AST (*mut c_void → program_t) into Rust Program type */
pub unsafe fn convert_program(c_prog: *mut c_void) -> Program {
    if c_prog.is_null() {
        return Program {
            statements: Vec::new(),
        };
    }
    let prog = unsafe { &*(c_prog as *const program_t) };
    let mut statements = Vec::new();
    let mut stmt_ptr = prog.statements;
    unsafe{
        for _ in 0..prog.stmt_count {
            if !stmt_ptr.is_null() {
                statements.push(convert_stmt(stmt_ptr));
                stmt_ptr = (*stmt_ptr).next;
            }
        }
    }

    /* return rust Program */
    Program { statements }
}
