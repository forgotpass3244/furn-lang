


#[derive(Clone)]
pub enum TypeVal {
    Unit,
    UInt64,
    StringSlice,
    FunctionPointer(Box<TypeVal>),
    MethodPointer, // also contains a reference to 'self'
}

impl TypeVal {
    pub fn size_of(&self) -> usize {
        match self {
            TypeVal::Unit => 0,
            TypeVal::UInt64 => 8,
            TypeVal::StringSlice => 16,
            TypeVal::FunctionPointer(..) => 8,
            TypeVal::MethodPointer => 16,
        }
    }
}
