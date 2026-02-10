
#[derive(Debug, Clone)]
pub enum IRNode {
    CallFromOffset(i16),
    Return,
    Push64(u64),
    Load64(u64),
    Pop64ToStack(usize),
    Load64ToStack(u64, usize),
    PushAddressFromOffset(i16),
    JumpFromOffset(i16),
    StackDealloc(usize),
    GlobalReadPush64(usize),
    GlobalReadLoad64ToStack(usize, usize),
}

