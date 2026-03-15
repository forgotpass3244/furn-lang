use std::collections::HashMap;

use crate::ir_gen::{symbol::CmplSymbol, typeval::TypeVal, variable::Variable};


#[derive(Clone, Debug)]
pub enum CTimeVal {
    Int(i128),
    StringSlice(usize, usize), // pointer, len
    Function { address: usize, return_typeval: TypeVal },
    DynamicFnDispatcher { map: HashMap<TypeVal, CmplSymbol>, meta_funcs: /* pre, mid, post */ Box<(CmplSymbol, CmplSymbol, CmplSymbol)> },
    Namespace(HashMap<String, Variable>),
    Type(TypeVal),
}

