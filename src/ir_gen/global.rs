use crate::ir_gen::ctimeval::CTimeVal;

#[derive(Clone)]
pub struct GlobalInfo<'a> {
    pub pos: usize, // offset from the global base
    pub name: &'a str,
    pub is_exported: bool,
    pub init: CTimeVal,
    pub is_const: bool,
}

impl<'a> GlobalInfo<'a> {
    pub fn new(pos: usize, name: &'a str, is_exported: bool, init: CTimeVal, is_const: bool) -> Self {
        Self {
            pos,
            name,
            is_exported,
            init,
            is_const,
        }
    }
}
