use crate::ir_gen::{ctimeval::CTimeVal, external::ExternalInfo, typeval::TypeVal};

#[derive(Clone)]
pub struct Variable<'a> {
    pub name: &'a str,
    pub type_val: TypeVal,
    pub global_pos: Option<usize>,
    pub stack_loc: Option<usize>,
    pub const_val: Option<CTimeVal>,
    pub external: Option<ExternalInfo<'a>>,
}
