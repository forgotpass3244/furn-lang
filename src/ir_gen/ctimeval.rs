use crate::ir_gen::typeval::TypeVal;


#[derive(Clone)]
pub enum CTimeVal {
    UInt(u64),
    Int(i128),
    StringSlice(usize, usize), // pointer, len
    Function { address: usize, return_type_val: TypeVal },
    Type(TypeVal),
}

