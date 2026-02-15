use crate::ir_gen::{ctimeval::CTimeVal, typeval::TypeVal};


pub struct CmplSymbol {
    pub const_val: Option<CTimeVal>,
    pub type_val: TypeVal,
}

