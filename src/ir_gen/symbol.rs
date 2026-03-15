
use crate::ir_gen::{ctimeval::CTimeVal, lifetime::Lifetime, typeval::{TypeVal, TypeValEnum}, variable::Variable};

#[derive(Clone, Default, Debug)]
pub struct CmplSymbol {
    pub const_val: Option<CTimeVal>,
    pub typeval: TypeVal,
    pub var: Option<Variable>,
    pub lifetime: Option<Lifetime>,
    pub is_unsafe: bool,
}

impl CmplSymbol {
    pub fn void() -> Self {
        Self {
            const_val: Some(CTimeVal::Type(TypeValEnum::Unit.to_tval())),
            typeval: TypeValEnum::Unit.to_tval(),
            var: None,
            lifetime: None,
            is_unsafe: false,
        }
    }
}

