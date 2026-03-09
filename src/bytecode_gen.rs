//! Байткод виртуальной машины для PLI.
//!
//! Соответствует синтаксису: declaration, assignment, if/else, while,
//! print, read; выражения с +-*/%, ==, !=, <, >, and, or, not, -.

use std::{env::var, fmt};
use std::collections::HashMap;

use crate::ast:: { LiteralValue, Program, Stmt, Stmt::VarDecl, Expr, Expr::Binary };

/// Опкод байткода. Стек: [...] — верх справа.
#[derive(Debug, Clone, PartialEq)]
pub enum Op {
    // === Константы (индекс в constant pool) ===
    PushConst(u16),

    // === Переменные (индекс в symbol table) ===
    Load(u16),  // push vars[idx]
    Store(u16), // pop -> vars[idx]

    // === Арифметика: pop b, pop a, push result ===
    Add,
    Sub,
    Mul,
    Div,
    Mod,

    // === Сравнение: pop b, pop a, push bool ===
    Equal,
    NotEqual,
    Less,
    Greater,
    LessEqual,
    GreaterEqual,

    // === Логика ===
    And, // short-circuit: JumpIfFalse для реализации
    Or,  // short-circuit: JumpIfTrue для реализации
    Not, // pop a, push !a

    // === Унарные ===
    Negate, // pop a, push -a

    // === Управление потоком (смещение в байтах/инструкциях) ===
    Jump(i16),        // безусловный переход
    JumpIfFalse(i16), // pop; если false — jump (для if, and)
    JumpIfTrue(i16),  // pop; если true — jump (для or)

    // === Ввод-вывод ===
    PrintN(u8), // pop N значений, напечатать (print(expr, expr, ...))
    Read(u16),  // прочитать, store в vars[idx] (read(x))

    // === Служебные ===
    Pop,  // снять вершину стека
    Dup,  // дублировать вершину
    Nop,
    Halt,
}

impl fmt::Display for Op {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Op::PushConst(i) => write!(f, "PushConst({})", i),
            Op::Load(i) => write!(f, "Load({})", i),
            Op::Store(i) => write!(f, "Store({})", i),
            Op::Add => write!(f, "Add"),
            Op::Sub => write!(f, "Sub"),
            Op::Mul => write!(f, "Mul"),
            Op::Div => write!(f, "Div"),
            Op::Mod => write!(f, "Mod"),
            Op::Equal => write!(f, "Equal"),
            Op::NotEqual => write!(f, "NotEqual"),
            Op::Less => write!(f, "Less"),
            Op::Greater => write!(f, "Greater"),
            Op::LessEqual => write!(f, "LessEqual"),
            Op::GreaterEqual => write!(f, "GreaterEqual"),
            Op::And => write!(f, "And"),
            Op::Or => write!(f, "Or"),
            Op::Not => write!(f, "Not"),
            Op::Negate => write!(f, "Negate"),
            Op::Jump(off) => write!(f, "Jump({})", off),
            Op::JumpIfFalse(off) => write!(f, "JumpIfFalse({})", off),
            Op::JumpIfTrue(off) => write!(f, "JumpIfTrue({})", off),
            Op::PrintN(n) => write!(f, "PrintN({})", n),
            Op::Read(i) => write!(f, "Read({})", i),
            Op::Pop => write!(f, "Pop"),
            Op::Dup => write!(f, "Dup"),
            Op::Nop => write!(f, "Nop"),
            Op::Halt => write!(f, "Halt"),
        }
    }
}

/* bytecode - operations list and map of variables */
#[derive(Debug, Default)]
pub struct ByteCode {
    pub ops: Vec<Op>,
    pub sytab: Vec<String>,

    /* БАЙТКОД НЕ ДОЛЖЕН ХРАНИТЬ МАПУ ПЕРЕМЕННЫХ, ЭТО НЕ РАНТАЙМ. НУЖНО ЛИШЬ РЕЗОЛВИТЬ ИМЕНА: symtab */
    pub vars: HashMap<String, LiteralValue>,    /* 
                                                   varname: String - key, 
                                                   value: LiteralValue
                                                */
}

impl ByteCode {
    pub fn push_op(&mut self, op: Op) -> usize {
        let pos = self.ops.len();
        self.ops.push(op);
        pos
    }

    pub fn add_var(&mut self, varname: String, value: LiteralValue) {
        self.vars.insert(varname, value);
    }

    pub fn has_var(&self, varname: &str) -> bool{
        let has = self.vars.get(varname);
        match has{
            Some(v) => { false },
            None => true
        }
    }

    pub fn rewrite_jump(&mut self, pos: usize, target: usize) {
        let offset = target as i16 - pos as i16;
        match &mut self.ops[pos] {
            Op::Jump(off)        |
            Op::JumpIfFalse(off) |
            Op::JumpIfTrue(off)  => *off = offset,
            
            _ => {}
        }
    }
}

fn process_var_decl(vd: Stmt, bcode: &mut ByteCode){

}

/* generate bytecode from ast */
pub fn generate_bytecode(ast: Program) -> ByteCode {
    let bcode = ByteCode::default();

    for stmt in ast.statements{
        match stmt{
            Stmt::VarDecl{var_name, initializer, line} => {
                if bcode.has_var(&var_name) == true{
                    panic!("variable already exists! line: {line}")
                }

                /* operations that calculate the value of a variable */
                match initializer{
                    Expr::Literal { value, line } => {

                    }

                    Expr::Unary { op, operand, line } => {

                    }

                    _ => panic!("undefined initializer type! line {line}")
                }
                //bcode.add_var(var_name, init);
            }
            _ => panic!("")
        }
    }

    bcode
}