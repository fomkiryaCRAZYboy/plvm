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

const BC_HEADER: &[u8] = b"PLIBCbeta"; /* pli bytecode signature */
const CONST_POOL_LABEL: &[u8] = b"poolstartlabel"; /* constant pool start label */
const SYMTAB_LABEL: &[u8] = b"symtabstartlabel"; /* symtab dtart label */

use std::fs::File;
use std::io::Write;

use std::{fmt};
use std::collections::HashMap;

use crate::ast:: { LiteralValue, Program, Stmt, Stmt::VarDecl, Expr, Expr::Binary };

/* opcode of bytecode */
#[derive(Debug, Clone, PartialEq)]
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
        self.ops.push(op);
        
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

        self.bytecode
    }
}