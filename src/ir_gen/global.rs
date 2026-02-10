use crate::ir_gen::ctimeval::CTimeVal;


pub struct GlobalInfo<'a> {
    pub name: &'a str,
    pub is_exported: bool,
    pub init: CTimeVal,
    pub is_const: bool,
}

impl<'a> GlobalInfo<'a> {
    pub fn new(name: &'a str, is_exported: bool, init: CTimeVal, is_const: bool) -> Self {
        Self {
            name,
            is_exported,
            init,
            is_const,
        }
    }
}
