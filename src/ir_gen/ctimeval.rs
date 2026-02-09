
#[derive(Clone)]
pub enum CTimeVal {
    UInt(u64),
    Int(i128),
    Function { address: usize },
}

