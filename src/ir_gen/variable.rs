use crate::ir_gen::ctimeval::CTimeVal;

#[derive(Clone)]
pub struct Variable<'a> {
    pub name: &'a str,
    pub const_val: Option<CTimeVal>,
}

