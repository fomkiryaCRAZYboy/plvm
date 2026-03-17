use crate::bytecode_gen:: { ByteCode, Op };
use crate::ast::LiteralValue;

struct Stack {
    /* variable meaning/value from const pool/number of operations to skip(jump) */
    slot: [LiteralValue; 96]
}

impl Stack {
    /* fn pop
    fn push
     */
}

pub fn exec(bc: ByteCode) {
    for op in bc.ops{
        match op {
            Op::PushConst(idx) => {

            }

            _ => {}
        }
    }
}