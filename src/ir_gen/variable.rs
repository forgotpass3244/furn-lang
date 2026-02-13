use crate::ir_gen::ctimeval::CTimeVal;

#[derive(Clone)]
pub struct Variable<'a> {
    pub name: &'a str,
    pub global_pos: Option<usize>,
    pub stack_loc: Option<usize>,
    pub const_val: Option<CTimeVal>,
}
