use std::fs::File;
use std::io::Write;

use crate::ir_gen::cmpld_program::CompiledProgram;
use crate::ir_gen::ctimeval::CTimeVal;
use crate::ir_gen::ir::IRNode;


pub fn gen_asm_x86_64_from_ir(out: &mut File, cprog: &CompiledProgram) -> Result<(), std::io::Error> {
    writeln!(out, "; nasm -felf64 rt.asm -o out/rt.o")?;
    writeln!(out)?;
    writeln!(out, "; nasm -felf64 out/out.asm -o out/out.o")?;
    writeln!(out, "; ld out/rt.o out/out.o -o out/out")?;
    writeln!(out)?;

    writeln!(out, "section .data")?;

    let package_name = cprog.get_package_name().unwrap_or("main");
    
    for global in cprog.globals_iter() {
        let pkg_prefix = if global.is_const { "PKG" } else { "PKGv" };

        if global.is_exported {
            writeln!(out, "global _{pkg_prefix}_{package_name}_{}", global.name)?;
        }

        if global.is_const {

            write!(out, "_{pkg_prefix}_{package_name}_{} equ ", global.name)?;
            match global.init {
                CTimeVal::UInt(int) => writeln!(out, "{int}")?,
                CTimeVal::Function { address } => writeln!(out, "OP_{address}")?,
                _ => todo!(),
            }

        } else {

            write!(out, "_{pkg_prefix}_{package_name}_{}: ", global.name)?;
            match global.init {
                CTimeVal::UInt(int) => writeln!(out, "dq {int}")?,
                CTimeVal::Function { address } => writeln!(out, "dq OP_{address}")?,
                _ => todo!(),
            }

        }
    }

    writeln!(out, "section .text")?;

    for (i, node) in cprog.ir_iter().enumerate() {
        match node {
            IRNode::Return => writeln!(out, "    OP_{i}: ret")?,
            IRNode::CallFromOffset(offset) => writeln!(out, "    OP_{i}: call OP_{}", (i as i16) + offset)?,
            IRNode::JumpFromOffset(offset) => writeln!(out, "    OP_{i}: jump OP_{}", (i as i16) + offset)?,
            IRNode::PushAddressFromOffset(offset) => writeln!(out, "    OP_{i}: push qword OP_{}", (i as i16) + offset)?,
            IRNode::Load64(int) => writeln!(out, "    OP_{i}: mov rax, {int}")?,
            IRNode::Push64(int) => writeln!(out, "    OP_{i}: push qword {int}")?,
            IRNode::StackDealloc(size) => writeln!(out, "    OP_{i}: add rsp, {size}")?,
            IRNode::Load64ToStack(int, offset) => writeln!(out, "    OP_{i}: mov qword [rsp+{}], {int}", offset)?,
            
            IRNode::Pop64ToStack(offset) => {
                writeln!(out, "OP_{i}:")?;
                writeln!(out, "    pop rax")?;
                writeln!(out, "    mov [rsp+{}], rax", offset - 8)?;
            },

        }
    }

    Ok(())
}

