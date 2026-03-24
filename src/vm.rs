use crate::ast::LiteralValue;
use crate::bytecode_gen::{ByteCode, Op};
use std::collections::HashMap;

const ERR_MSG: &str = "invalid bytecode or a compiler bug!";

fn literal_truthy(v: &LiteralValue) -> bool {
    match v {
        LiteralValue::Boolean(b) => *b,
        LiteralValue::Number(n) => *n != 0.0,
        LiteralValue::String(s) => !s.is_empty(),
    }
}

fn literals_equal(a: &LiteralValue, b: &LiteralValue) -> bool {
    match (a, b) {
        (LiteralValue::Number(x), LiteralValue::Number(y)) => x == y,
        (LiteralValue::String(s1), LiteralValue::String(s2)) => s1 == s2,
        (LiteralValue::Boolean(b1), LiteralValue::Boolean(b2)) => b1 == b2,
        _ => false,
    }
}

#[derive(Debug)]
struct Variable {
    varname: String,
    value: Option<LiteralValue>,
}

impl Default for Variable {
    fn default() -> Self {
        Self {
            varname: String::new(),
            value: None,
        }
    }
}

#[derive(Debug)]
struct RuntimeContext {
    stack: Vec<LiteralValue>,
    vars: Vec<Variable>,
}

impl RuntimeContext {
    fn new(vars: Vec<Variable>) -> Self {
        Self {
            stack: Vec::new(),
            vars,
        }
    }

    fn push(&mut self, lit: LiteralValue) {
        self.stack.push(lit);
    }

    fn pop(&mut self) -> Option<LiteralValue> {
        self.stack.pop()
    }

    /** Pops `b` (right), then `a` (left), matching bytecode binary ops: `a op b`. */
    fn pop_two(&mut self, op_name: &'static str) -> (LiteralValue, LiteralValue) {
        let b = self.pop().unwrap_or_else(|| {
            panic!("{} {}: stack underflow (need 2 values)", ERR_MSG, op_name)
        });
        let a = self.pop().unwrap_or_else(|| {
            panic!("{} {}: stack underflow (need 2 values, got 1)", ERR_MSG, op_name)
        });
        (a, b)
    }

    fn pop_one(&mut self, op_name: &'static str) -> LiteralValue {
        self.pop().unwrap_or_else(|| {
            panic!("{} {}: stack underflow (need 1 value)", ERR_MSG, op_name)
        })
    }
}

fn vars_from_symtab(symtab: HashMap<String, u16>) -> Vec<Variable> {
    let n = symtab.len();
    let mut vars: Vec<Variable> = (0..n).map(|_| Variable::default()).collect();
    for (name, idx) in symtab {
        vars[idx as usize] = Variable {
            varname: name.clone(),
            value: None,
        };
    }

    vars
}

pub fn exec(bc: ByteCode) {
    let vars = vars_from_symtab(bc.symtab);
    let mut context = RuntimeContext::new(vars);

    for op in bc.ops {
        match op {
            Op::PushConst(idx) => {
                context.push(bc.const_pool[idx as usize].clone());
            }

            Op::Load(idx) => {
                let v = context.vars[idx as usize]
                    .value
                    .clone()
                    .unwrap_or(LiteralValue::Number(0.0));

                context.push(v);
            }

            Op::Store(idx) => {
                match context.pop() {
                    Some(v) => context.vars[idx as usize].value = Some(v),
                    None => panic!("{} Store on empty stack", ERR_MSG)
                }
            }

            Op::Add => {
                /* Bytecode: pop b, pop a, push a + b */
                let (a, b) = context.pop_two("Add");
                let out = match (a, b) {
                    (LiteralValue::Number(x), LiteralValue::Number(y)) => {
                        LiteralValue::Number(x + y)
                    }

                    (LiteralValue::String(s1), LiteralValue::String(s2)) => {
                        LiteralValue::String(format!("{s1}{s2}"))
                    }

                    (LiteralValue::String(s), LiteralValue::Number(n))
                    | (LiteralValue::Number(n), LiteralValue::String(s)) => {
                        panic!("{} Add: Can not add a string to a number", ERR_MSG)
                    }

                    _ => panic!("{} Add: unsupported operand types", ERR_MSG),
                };

                context.push(out);
            }

            Op::Sub => {
                /* Bytecode: pop b, pop a, push a - b */
                let (a, b) = context.pop_two("Sub");
                let out = match (a, b) {
                    (LiteralValue::Number(x), LiteralValue::Number(y)) => {
                        LiteralValue::Number(x - y)
                    }

                    _ => panic!("{} Sub: unsupported operand types", ERR_MSG),
                };

                context.push(out);
            }

            Op::Mul => {
                /* Bytecode: pop b, pop a, push a * b */
                let (a, b) = context.pop_two("Mul");
                let out = match (a, b) {
                    (LiteralValue::Number(x), LiteralValue::Number(y)) => {
                        LiteralValue::Number(x * y)
                    }
                    _ => panic!("{} Mul: unsupported operand types", ERR_MSG),
                };
                context.push(out);
            }

            Op::Div => {
                /* Bytecode: pop b, pop a, push a / b */
                let (a, b) = context.pop_two("Div");
                let out = match (a, b) {
                    (LiteralValue::Number(x), LiteralValue::Number(y)) => {
                        let q = if y == 0.0 { 0.0 } else { x / y };
                        LiteralValue::Number(q)
                    }
                    _ => panic!("{} Div: unsupported operand types", ERR_MSG),
                };
                context.push(out);
            }

            Op::Equal => {
                let (a, b) = context.pop_two("Equal");
                let out = LiteralValue::Boolean(literals_equal(&a, &b));
                context.push(out);
            }

            Op::NEqual => {
                let (a, b) = context.pop_two("NEqual");
                let out = LiteralValue::Boolean(!literals_equal(&a, &b));
                context.push(out);
            }

            Op::Less => {
                let (a, b) = context.pop_two("Less");
                let out = match (a, b) {
                    (LiteralValue::Number(x), LiteralValue::Number(y)) => LiteralValue::Boolean(x < y),
                    (LiteralValue::String(s1), LiteralValue::String(s2)) => {
                        LiteralValue::Boolean(s1 < s2)
                    }
                    _ => panic!("{} Less: unsupported operand types", ERR_MSG),
                };
                context.push(out);
            }

            Op::Greater => {
                let (a, b) = context.pop_two("Greater");
                let out = match (a, b) {
                    (LiteralValue::Number(x), LiteralValue::Number(y)) => LiteralValue::Boolean(x > y),
                    (LiteralValue::String(s1), LiteralValue::String(s2)) => {
                        LiteralValue::Boolean(s1 > s2)
                    }
                    _ => panic!("{} Greater: unsupported operand types", ERR_MSG),
                };
                context.push(out);
            }

            Op::LEqual => {
                let (a, b) = context.pop_two("LEqual");
                let out = match (a, b) {
                    (LiteralValue::Number(x), LiteralValue::Number(y)) => LiteralValue::Boolean(x <= y),
                    (LiteralValue::String(s1), LiteralValue::String(s2)) => {
                        LiteralValue::Boolean(s1 <= s2)
                    }
                    _ => panic!("{} LEqual: unsupported operand types", ERR_MSG),
                };
                context.push(out);
            }

            Op::GEqual => {
                let (a, b) = context.pop_two("GEqual");
                let out = match (a, b) {
                    (LiteralValue::Number(x), LiteralValue::Number(y)) => LiteralValue::Boolean(x >= y),
                    (LiteralValue::String(s1), LiteralValue::String(s2)) => {
                        LiteralValue::Boolean(s1 >= s2)
                    }
                    _ => panic!("{} GEqual: unsupported operand types", ERR_MSG),
                };
                context.push(out);
            }

            Op::And => {
                let (a, b) = context.pop_two("And");
                let out = LiteralValue::Boolean(literal_truthy(&a) && literal_truthy(&b));
                context.push(out);
            }

            Op::Or => {
                let (a, b) = context.pop_two("Or");
                let out = LiteralValue::Boolean(literal_truthy(&a) || literal_truthy(&b));
                context.push(out);
            }

            Op::Not => {
                let a = context.pop_one("Not");
                let out = LiteralValue::Boolean(!literal_truthy(&a));
                context.push(out);
            }

            Op::Negate => {
                let a = context.pop_one("Negate");
                let out = match a {
                    LiteralValue::Number(n) => LiteralValue::Number(-n),
                    _ => panic!("{} Negate: expected number", ERR_MSG),
                };
                context.push(out);
            }

            /*
            Jump(i16),        /* unconditional jump */
    JumpIfFalse(i16), /* pop; if false — jump (for if, and) */
    JumpIfTrue(i16),
             */

            Op::PrintN(count) => {
                let mut first = true;
                for _ in 0..count {
                    let lit = match context.pop() {
                        Some(v) => v,
                        None => panic!("{} Print: stack is empty", ERR_MSG),
                    };
                    if !first {
                        print!(" ");
                    }

                    first = false;
                    match lit {
                        LiteralValue::Number(n) => print!("{n}"),
                        LiteralValue::String(ref s) => print!("{s}"),
                        LiteralValue::Boolean(b) => print!("{b}"),
                    }
                }
                println!();
            }

            _ => {}
        }
    }
}
