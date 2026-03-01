use crate::ir_gen::{ctimeval::CTimeVal, typeval::TypeVal, variable::Variable};

#[derive(Clone)]
pub struct CmplSymbol {
    pub const_val: Option<CTimeVal>,
    pub type_val: TypeVal,
    pub var: Option<Variable>,
}

