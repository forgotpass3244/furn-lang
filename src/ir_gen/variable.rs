use crate::ir_gen::{ctimeval::CTimeVal, external::ExternalInfo, lifetime::Lifetime, scope::Named, typeval::TypeVal};

#[derive(Clone, Debug)]
pub struct Variable {
    pub name: String,
    pub typeval: TypeVal,
    pub global_pos: Option<usize>,
    pub stack_loc: Option<usize>,
    pub const_val: Option<CTimeVal>,
    pub external: Option<ExternalInfo>,
    pub is_alias: bool,
    pub lifetime: Lifetime,
    pub is_unsafe: bool,
}

impl Named for Variable {
    fn get_name(&self) -> &String {
        &self.name
    }
}
