
#[derive(Debug)]
pub enum IRNode {
    CallFromOffset(i16),
    Return,
    Push64(u64),
    Pop64ToStack(usize),
    PushAddressFromOffset(i16),
    JumpFromOffset(i16),
    StackDealloc(usize),
}

