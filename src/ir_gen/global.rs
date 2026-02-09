use crate::ir_gen::ctimeval::CTimeVal;


pub struct GlobalInfo<'a> {
    pub name: &'a str,
    pub is_exported: bool,
    pub init: CTimeVal,
}

impl<'a> GlobalInfo<'a> {
    pub fn new(name: &'a str, is_exported: bool, init: CTimeVal) -> Self {
        Self {
            name,
            is_exported,
            init,
        }
    }
}
