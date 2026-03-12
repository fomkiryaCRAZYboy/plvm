macro_rules! missing_decl {
    ($line:expr, $var:expr) => {
        panic!(
            "missing declaration: line: {}. Variable '{}' does not exist!",
            $line, $var
        )
    };
}

macro_rules! redecl {
    ($line:expr, $var:expr) => {
        panic!{
            "redeclaration detected: line: {}. Variable '{}' already exists!",
            $line, $var
        }
    };
}

use crate::ast:: { LiteralValue, Program, Stmt, Stmt::VarDecl, Expr, Expr::Binary, BinaryOp ,BinaryOp::*, UnaryOp, UnaryOp::* };

macro_rules! simple_exp {
    ($e: expr) => {
        matches!($e, Expr::Literal { .. } | Expr::Variable { .. })
    };
}

macro_rules! heavy_exp {
    ($e: expr) => {
        matches!($e, Expr::Grouping { .. } | Expr::Unary { .. } | Expr::Binary { .. })
    };
}

macro_rules! bin_op_to_opcode {
    ($op: expr) => {
        match $op {
            BinaryOp::And => Op::And,
            crate::ast::BinaryOp::Or => Op::Or,
            crate::ast::BinaryOp::Equal => Op::Equal,
            crate::ast::BinaryOp::NotEqual => Op::NEqual,
            crate::ast::BinaryOp::Less => Op::Less,
            crate::ast::BinaryOp::Greater => Op::Greater,
            crate::ast::BinaryOp::LessEqual => Op::LEqual,
            crate::ast::BinaryOp::GreaterEqual => Op::GEqual,
            crate::ast::BinaryOp::Add => Op::Add,
            crate::ast::BinaryOp::Subtract => Op::Sub,
            crate::ast::BinaryOp::Multiply => Op::Mul,
            crate::ast::BinaryOp::Divide => Op::Div,
            crate::ast::BinaryOp::Modulo => Op::Mod,
        }
    };
}

const BC_HEADER: &[u8] = b"PLIBCbeta"; /* pli bytecode signature */
const CONST_POOL_LABEL: &[u8] = b"poolstartlabel"; /* constant pool start label */
const SYMTAB_LABEL: &[u8] = b"symtabstartlabel"; /* symtab dtart label */

/* code map */
const PUSH_CONST: u8 = 0x01;
const LOAD: u8       = 0x02;
const STORE: u8      = 0x03;

const ADD: u8        = 0x10;
const SUB: u8        = 0x11;
const MUL: u8        = 0x12;
const DIV: u8        = 0x13;
const MOD: u8        = 0x14;

const EQUAL: u8      = 0x20;
const N_EQUAL: u8    = 0x21;
const LESS: u8       = 0x22;
const GREATER: u8    = 0x23;
const L_EQUAL: u8    = 0x24;
const G_EQUAL: u8    = 0x25;

const AND: u8        = 0x30;
const OR: u8         = 0x31;
const NOT: u8        = 0x32;
const NEGATE: u8     = 0x33;

const JUMP: u8           = 0x40;
const JUMP_IF_FALSE: u8  = 0x41;
const JUMP_IF_TRUE: u8   = 0x42;

const PRINT_N: u8    = 0x50;
const READ: u8       = 0x51;

const POP: u8        = 0x60;
const DUP: u8        = 0x61;
const NOP: u8        = 0x62;
const HALT: u8       = 0x63;

use std::any::Any;
use std::fs::File;
use std::io::Write;

use std::{fmt};
use std::collections::HashMap;

/* opcode of bytecode */
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Op {
    // === Constants (index in constant pool) ===
    PushConst(u16),

    // === Variables (index in symbol table) ===
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
    NEqual,
    Less,
    Greater,
    LEqual,
    GEqual,

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
            Op::NEqual => write!(f, "NotEqual"),
            Op::Less => write!(f, "Less"),
            Op::Greater => write!(f, "Greater"),
            Op::LEqual => write!(f, "LessEqual"),
            Op::GEqual => write!(f, "GreaterEqual"),
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
#[derive(Debug)]
pub struct ByteCode {
    pub ops: Vec<Op>,
    pub symtab: HashMap<String, u16>, /* varname: key */
    pub const_pool: Vec<LiteralValue>,
    pub plibc_file: File
}

impl ByteCode {
    pub fn new() -> Self {
        let mut plibc_file = File::create("plibc.plbc").expect("failed to create plibc.bc file");
        let result = plibc_file.write_all(BC_HEADER);

        match result {
            Ok(_) => {}
            Err(e) => panic!("failed to write BC_HEADER to plibc.plbc file: {}", e),
        }

        Self { ops: Vec::new(), symtab: HashMap::new(), const_pool: Vec::new(), plibc_file }
    }

    pub fn push_op(&mut self, op: Op) -> usize {
        let pos = self.ops.len();

        /* add operation to ops */
        self.ops.push(op.clone());

        /* write operation in file */
        let result = self._write_op(op);
        match result {
            Ok(_) => {}
            Err(e) => panic!("failed to write operation to plibc.plbc file: {}", e),
        }

        pos
    }

    fn _add_sym(&mut self, sym: String) -> u16 {
        let idx: u16 = self.symtab.len() as u16;
        self.symtab.insert(sym, idx);
        
        idx
    }

    pub fn has_sym(&self, sym: &str) -> bool {
        let has = self.symtab.get(sym);
        match has {
            Some(_) => true,
            None => false
        }
    }

    pub fn get_or_add_sym(&mut self, sym: String) -> u16 {
        if let Some(idx) = self.symtab.get(&sym) {
            return *idx;
        }

        self._add_sym(sym)
    }

    fn _add_const(&mut self, lit: LiteralValue) -> u16 {
        let idx = self.const_pool.len(); 
        self.const_pool.insert(idx, lit);

        idx as u16
    }

    pub fn get_or_add_const(&mut self, lit: &LiteralValue) -> u16 {
        if let Some(idx) = self.const_pool.iter().position(|v| v == lit) {
            return idx as u16;
        }

        self._add_const(lit.clone())
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

    pub fn _write_op(&mut self, op: Op) -> std::io::Result<()> {
        match op {
            Op::PushConst(idx) => {
                self.plibc_file.write_all(&[PUSH_CONST])?;
                self.plibc_file.write_all(&idx.to_le_bytes())?;
            }
            Op::Load(idx) => {
                self.plibc_file.write_all(&[LOAD])?;
                self.plibc_file.write_all(&idx.to_le_bytes())?;
            }
            Op::Store(idx) => {
                self.plibc_file.write_all(&[STORE])?;
                self.plibc_file.write_all(&idx.to_le_bytes())?;
            }
            Op::Add => self.plibc_file.write_all(&[ADD])?,
            Op::Sub => self.plibc_file.write_all(&[SUB])?,
            Op::Mul => self.plibc_file.write_all(&[MUL])?,
            Op::Div => self.plibc_file.write_all(&[DIV])?,
            Op::Mod => self.plibc_file.write_all(&[MOD])?,
            Op::Equal => self.plibc_file.write_all(&[EQUAL])?,
            Op::NEqual => self.plibc_file.write_all(&[N_EQUAL])?,
            Op::Less => self.plibc_file.write_all(&[LESS])?,
            Op::Greater => self.plibc_file.write_all(&[GREATER])?,
            Op::LEqual => self.plibc_file.write_all(&[L_EQUAL])?,
            Op::GEqual => self.plibc_file.write_all(&[G_EQUAL])?,
            Op::And => self.plibc_file.write_all(&[AND])?,
            Op::Or => self.plibc_file.write_all(&[OR])?,
            Op::Not => self.plibc_file.write_all(&[NOT])?,
            Op::Negate => self.plibc_file.write_all(&[NEGATE])?,
            Op::Jump(off) => {
                self.plibc_file.write_all(&[JUMP])?;
                self.plibc_file.write_all(&off.to_le_bytes())?;
            }
            Op::JumpIfFalse(off) => {
                self.plibc_file.write_all(&[JUMP_IF_FALSE])?;
                self.plibc_file.write_all(&off.to_le_bytes())?;
            }
            Op::JumpIfTrue(off) => {
                self.plibc_file.write_all(&[JUMP_IF_TRUE])?;
                self.plibc_file.write_all(&off.to_le_bytes())?;
            }
            Op::PrintN(n) => {
                self.plibc_file.write_all(&[PRINT_N])?;
                self.plibc_file.write_all(&[n])?;
            }
            Op::Read(idx) => {
                self.plibc_file.write_all(&[READ])?;
                self.plibc_file.write_all(&idx.to_le_bytes())?;
            }
            Op::Pop => self.plibc_file.write_all(&[POP])?,
            Op::Dup => self.plibc_file.write_all(&[DUP])?,
            Op::Nop => self.plibc_file.write_all(&[NOP])?,
            Op::Halt => self.plibc_file.write_all(&[HALT])?,
        }

        Ok(())
    }

    pub fn write_const_pool(&mut self) -> std::io::Result<()> {
        self.plibc_file.write_all(CONST_POOL_LABEL)?;

        let count = self.const_pool.len() as u32;
        self.plibc_file.write_all(&count.to_le_bytes())?;

        for c in &self.const_pool {
            match c {
                LiteralValue::Boolean(v) => {
                    self.plibc_file.write_all(&[0x01])?;
                    self.plibc_file.write_all(&[*v as u8])?;
                }
                LiteralValue::Number(v) => {
                    self.plibc_file.write_all(&[0x02])?;
                    self.plibc_file.write_all(&v.to_le_bytes())?;
                }
                LiteralValue::String(s) => {
                    self.plibc_file.write_all(&[0x03])?;
                    let bytes = s.as_bytes();
                    let len = bytes.len() as u32;
                    self.plibc_file.write_all(&len.to_le_bytes())?;
                    self.plibc_file.write_all(bytes)?;
                }
            }
        }

        Ok(())
    }

    pub fn write_symtab(&mut self) -> std::io::Result<()> {
        self.plibc_file.write_all(SYMTAB_LABEL)?;

        let count = self.symtab.len() as u32;
        self.plibc_file.write_all(&count.to_le_bytes())?;

        // HashMap не гарантирует порядок: пишем стабильно по индексу слота.
        let mut entries: Vec<(&String, &u16)> = self.symtab.iter().collect();
        entries.sort_by_key(|(_, idx)| *idx);

        for (name, idx) in entries {
            let name_bytes = name.as_bytes();
            let name_len = name_bytes.len() as u32;

            self.plibc_file.write_all(&name_len.to_le_bytes())?;
            self.plibc_file.write_all(name_bytes)?;
            self.plibc_file.write_all(&idx.to_le_bytes())?;
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct Generator{
    pub bytecode: ByteCode
}

impl Generator {
    pub fn new() -> Self {
        Self { bytecode: ByteCode::new() }  
   }

    /* generate bytecode from ast */
    pub fn generate_bytecode(&mut self, ast: Program) {
        for stmt in ast.statements{
            match stmt{
                Stmt::VarDecl{var_name, initializer, line} => {
                    self.process_var_decl(&var_name, &initializer, line);
                }

                Stmt::Assignment { var_name, value, line } => {
                    self.process_assignment(&var_name, &value, line);
                }

                _ => panic!("undefined statement type!")
            }
        }
    }

    pub fn process_var_decl(&mut self, var_name: &str, initializer: &Expr, line: i32) {
        if self.bytecode.has_sym(&var_name){
            redecl!(line, var_name) /* panic! */
        }
    
        self._process_expr(&initializer);

        let s_idx = self.bytecode.get_or_add_sym(var_name.to_string());
        self.bytecode.push_op(Op::Store(s_idx));
    }

    pub fn process_assignment(&mut self, var_name: &str, value: &Expr, line: i32) {
        if !self.bytecode.has_sym(&var_name){
            missing_decl!(line, var_name) /* panic! */
        }

        self._process_expr(&value);

        let s_idx = self.bytecode.get_or_add_sym(var_name.to_string());
        self.bytecode.push_op(Op::Store(s_idx));
    }


    /* add a set of operations representing an expression to bytecode */
    fn _process_expr(&mut self, expr: &Expr) {
        match expr{
            Expr::Literal { value, line } => {
                self.__process_literal(&value);
            }

            Expr::Variable { name, line } => {
                if !self.bytecode.has_sym(&name) {
                    missing_decl!(line, name) /* panic! */
                }

                self.__process_var(&name);
            }

            Expr::Unary { op, operand, line } => {
                self._process_expr(operand);

                match op {
                    UnaryOp::Negate => {
                        self.bytecode.push_op(Op::Negate);
                    }

                    UnaryOp::Not => {
                        self.bytecode.push_op(Op::Not);
                    }

                    _ => {
                        panic!("undefined unary operation! line: {line}");
                    }      
                }
            }

            Expr::Binary { op, left, right, line } => {
                /* logic for optimizing the order of evaluation */
                /* first of all, heavyweight expressions are evaluated, their result is placed on the stack */
                if heavy_exp!(right.as_ref()) && simple_exp!(left.as_ref()) {
                    self._process_expr(right.as_ref());
                    self._process_expr(left.as_ref());    

                    self.bytecode.push_op(bin_op_to_opcode!(op));
                } else {
                    self._process_expr(left.as_ref());
                    self._process_expr(right.as_ref());    

                    self.bytecode.push_op(bin_op_to_opcode!(op));
                }
            }

            Expr::Grouping { expression, line } => {
                self._process_expr(expression);
            }

            _ => panic!("undefined expression type!")
        }
    }

    fn __process_literal(&mut self, lit: &LiteralValue) {
        /* add constant to pool if it isn`t already exists */
        let c_idx = self.bytecode.get_or_add_const(&lit);

        /* push literal on top of stack */
        self.bytecode.push_op(Op::PushConst(c_idx));
    }

    fn __process_var(&mut self, var_name: &str) {
        /* get the var idx */
        let s_idx = self.bytecode.get_or_add_sym(var_name.to_string());

        /* 'push' variable on top of stack */
        self.bytecode.push_op(Op::Load(s_idx));
    }

    pub fn finish(mut self) -> ByteCode {
        let res = self.bytecode.write_const_pool();
        match res {
            Ok(_) => {}
            Err(e) => panic!("failed to write const pool to plibc.plbc file: {}", e),
        }

        let res = self.bytecode.write_symtab();
        match res {
            Ok(_) => {}
            Err(e) => panic!("failed to write symbol table to plibc.plbc file: {}", e),
        }

        self.bytecode
    }
}