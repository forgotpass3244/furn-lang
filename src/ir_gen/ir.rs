use crate::ir_gen::external::ExternalInfo;


#[derive(Debug, Clone)]
pub enum IRNode<'a> {
    Call,
    ExternalReadPush64(ExternalInfo<'a>),
    CallFromOffset(i16),
    Return { params_size: usize },
    Push64(u64),
    Pop64ToStack(usize),
    Load64ToStack(u64, usize),
    PushAddressFromOffset(i16),
    JumpFromOffset(i16),
    StackAlloc(usize),
    StackDealloc(usize),
    GlobalReadPush64(usize),
    GlobalReadLoad64ToStack(usize, usize),
    StackReadPush64(usize),
    StackReadLoad64ToStack(usize, usize),
    PushStaticStringPointer(usize),
}

