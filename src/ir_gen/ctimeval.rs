use std::collections::HashMap;

use crate::ir_gen::{typeval::TypeVal, variable::Variable};


#[derive(Clone)]
pub enum CTimeVal {
    UInt(u64),
    Int(i128),
    StringSlice(usize, usize), // pointer, len
    Function { address: usize, return_type_val: TypeVal },
    Namespace(HashMap<String, Variable>),
    Type(TypeVal),
}

