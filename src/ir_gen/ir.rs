use crate::ir_gen::external::ExternalInfo;


#[derive(Debug, Clone)]
pub enum IRNode {
    Call,
    ExternalReadPush64(ExternalInfo),
    ExternalReadCall(ExternalInfo),
    CallFromOffset(i64),
    JumpIfNot64FromOffset(i64),
    Return { params_size: usize },
    Push64(u64),
    Pop64ToStack(usize),
    Load64ToStack(u64, usize),
    PushAddressFromOffset(i64),
    JumpFromOffset(i64),
    StackAlloc(usize),
    StackDealloc(usize),
    GlobalReadPush64(usize),
    GlobalReadLoad64ToStack(usize, usize),
    StackReadPush64(usize),
    StackReadLoad64ToStack(usize, usize),
    PushStaticStringPointer(usize),
    PushStackPointer(usize),
    Deref64,
}

