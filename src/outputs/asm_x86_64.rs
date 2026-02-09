use std::fs::File;
use std::io::Write;

use crate::ir_gen::ctimeval::CTimeVal;
use crate::ir_gen::global::GlobalInfo;
use crate::ir_gen::ir::IRNode;


pub fn gen_asm_x86_64_from_ir(out: &mut File, ir: &Vec<IRNode>, globals: &Vec<GlobalInfo>) -> Result<(), std::io::Error> {
    writeln!(out, "; nasm -felf64 rt.asm -o out/rt.o")?;
    writeln!(out)?;
    writeln!(out, "; nasm -felf64 out/out.asm -o out/out.o")?;
    writeln!(out, "; ld out/rt.o out/out.o -o out/out")?;
    writeln!(out)?;

    writeln!(out, "section .data")?;
    
    for global in globals {
        if global.is_exported {
            writeln!(out, "global {}", global.name)?;
        }

        write!(out, "{}: ", global.name)?;
        match global.init {
            CTimeVal::UInt(int) => writeln!(out, "dq {int}")?,
            CTimeVal::Function { address } => writeln!(out, "dq OP_{address}")?,
            _ => todo!(),
        }
    }

    writeln!(out, "section .text")?;

    for (i, node) in ir.iter().enumerate() {
        match node {
            IRNode::Return => writeln!(out, "    OP_{i}: ret")?,

            IRNode::Pop64ToStack(offset) => {
                writeln!(out, "OP_{i}:")?;
                writeln!(out, "    pop rax")?;
                writeln!(out, "    mov [rsp+{}], rax", offset - 8)?;
            },

            IRNode::CallFromOffset(offset) => writeln!(out, "    OP_{i}: call OP_{}", (i as i16) + offset)?,
            IRNode::JumpFromOffset(offset) => writeln!(out, "    OP_{i}: jump OP_{}", (i as i16) + offset)?,
            IRNode::PushAddressFromOffset(offset) => writeln!(out, "    OP_{i}: push qword OP_{}", (i as i16) + offset)?,
            IRNode::Push64(int) => writeln!(out, "    OP_{i}: push qword {int}")?,
            IRNode::StackDealloc(size) => writeln!(out, "    OP_{i}: add rsp, {size}")?,
        }
    }

    Ok(())
}

