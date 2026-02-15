use std::fs::File;
use std::io::Write;

use crate::ir_gen::cmpld_program::CompiledProgram;
use crate::ir_gen::ctimeval::CTimeVal;
use crate::ir_gen::ir::IRNode;


pub fn gen_asm_x86_64_from_ir(out: &mut File, cprog: &CompiledProgram) -> Result<(), std::io::Error> {
    writeln!(out, "section .data")?;

    let package_name = cprog.get_package_name().unwrap_or("main");

    for external in cprog.externals_iter() {
        if external.is_const {
            writeln!(out, "extern _PKG_{}_{}", external.package_name, external.name)?;
        } else {
            writeln!(out, "extern _PKGv_{}_{}", external.package_name, external.name)?;
        }
    }

    for (string, pos) in cprog.static_strings_iter() {
        write!(out, "_STR_{}: db ", pos)?;

        for (i, ch) in string.chars().enumerate() {
            if string.len() > (i + 1) {
                write!(out, "{},", ch as u8)?;
            } else {
                write!(out, "{}", ch as u8)?;
            }
        }

        writeln!(out)?;
    }
    
    for global in cprog.globals_iter() {
        let pkg_prefix = if global.is_const { "PKG" } else { "PKGv" };

        if global.is_exported {
            writeln!(out, "global _{pkg_prefix}_{package_name}_{}", global.name)?;
            writeln!(out, "_{pkg_prefix}_{package_name}_{} equ _GLOB_{}", global.name, global.pos)?;
        }

        if global.is_const {

            match global.init {
                CTimeVal::UInt(int) => writeln!(out, "_GLOB_{} equ {int}", global.pos)?,
                CTimeVal::Function { address, .. } => writeln!(out, "_GLOB_{} equ OP_{address}", global.pos)?,
                CTimeVal::StringSlice(pointer, len) => {
                    writeln!(out, "_GLOB_{} equ _STR_{pointer}", global.pos)?;
                    writeln!(out, "_GLOB_{} equ {len}", global.pos + 8)?;
                },
                _ => todo!(),
            }

        } else {

            match global.init {
                CTimeVal::UInt(int) => writeln!(out, "_GLOB_{}: dq {int}", global.pos)?,
                CTimeVal::Function { address, .. } => writeln!(out, "_GLOB_{}: dq OP_{address}", global.pos)?,
                CTimeVal::StringSlice(pointer, len) => {
                    write!(out, "_GLOB_{}: ", global.pos)?;
                    writeln!(out, "dq _STR_{pointer}")?;
                    write!(out, "_GLOB_{}: ", global.pos + 8)?;
                    writeln!(out, "dq {len}")?;
                },
                _ => todo!(),
            }

        }
    }

    writeln!(out, "section .text")?;

    for (i, node) in cprog.ir_iter().enumerate() {
        match node {
            IRNode::Return { params_size: 0 } => writeln!(out, "    OP_{i}: ret")?,
            IRNode::Return { params_size } => writeln!(out, "    OP_{i}: ret {params_size}")?,
            IRNode::CallFromOffset(offset) => writeln!(out, "    OP_{i}: call OP_{}", (i as i16) + offset)?,
            IRNode::JumpFromOffset(offset) => writeln!(out, "    OP_{i}: jump OP_{}", (i as i16) + offset)?,
            IRNode::PushAddressFromOffset(offset) => writeln!(out, "    OP_{i}: push qword OP_{}", (i as i16) + offset)?,
            IRNode::Push64(int) => writeln!(out, "    OP_{i}: push qword {int}")?,
            IRNode::StackAlloc(size) => writeln!(out, "    OP_{i}: sub rsp, {size}")?,
            IRNode::StackDealloc(size) => writeln!(out, "    OP_{i}: add rsp, {size}")?,
            IRNode::Load64ToStack(int, offset) => writeln!(out, "    OP_{i}: mov qword [rsp+{offset}], {int}")?,
            IRNode::GlobalReadPush64(offset) => writeln!(out, "    OP_{i}: push qword [_GLOB_{offset}]")?,
            IRNode::StackReadPush64(offset) => writeln!(out, "    OP_{i}: push qword [rsp+{offset}]")?,
            IRNode::PushStaticStringPointer(pos) => writeln!(out, "    OP_{i}: push qword _STR_{pos}")?,
            
            IRNode::ExternalReadPush64(external) => {
                if external.is_const {
                    writeln!(out, "    OP_{i}: push _PKG_{}_{}", external.package_name, external.name)?;
                } else {
                    writeln!(out, "    OP_{i}: push qword [_PKGv_{}_{}]", external.package_name, external.name)?;
                }
            },

            IRNode::Pop64ToStack(offset) => {
                writeln!(out, "OP_{i}:")?;
                writeln!(out, "    pop rax")?;
                writeln!(out, "    mov [rsp+{}], rax", offset - 8)?;
            },

            IRNode::GlobalReadLoad64ToStack(global_offset, offset) => {
                writeln!(out, "OP_{i}:")?;
                writeln!(out, "    mov rax, [_GLOB_{global_offset}]")?;
                writeln!(out, "    mov [rsp+{offset}], rax")?;
            },

            IRNode::StackReadLoad64ToStack(src_offset, dst_offset) => {
                writeln!(out, "OP_{i}:")?;
                writeln!(out, "    mov rax, [rsp+{src_offset}]")?;
                writeln!(out, "    mov [rsp+{dst_offset}], rax")?;
            },

            IRNode::Call => {
                writeln!(out, "OP_{i}:")?;
                writeln!(out, "    pop rax")?;
                writeln!(out, "    call rax")?;
            },
        }
    }

    Ok(())
}

