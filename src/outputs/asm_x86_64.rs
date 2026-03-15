use std::fs::File;
use std::io::{BufWriter, Write};

use crate::ir_gen::cmpld_program::CompiledProgram;
use crate::ir_gen::ctimeval::CTimeVal;
use crate::ir_gen::ir::IRNode;


pub fn gen_asm_x86_64_from_ir(out: &mut BufWriter<File>, cprog: &CompiledProgram) -> Result<(), std::io::Error> {
    let package_name = cprog.get_package_name().unwrap_or("Main");

    for external in cprog.externals_iter() {
        if external.is_const {
            writeln!(out, "extern {}?{}", external.package_name, external.name)?;
        } else {
            writeln!(out, "extern {}${}", external.package_name, external.name)?;
        }
    }

    writeln!(out, "section .rodata")?;
    for (string, pos) in cprog.static_strings_iter() {
        write!(out, "STR_{}: db ", pos)?;

        for (i, ch) in string.chars().enumerate() {
            if string.len() > (i + 1) {
                write!(out, "{},", ch as u8)?;
            } else {
                write!(out, "{}", ch as u8)?;
            }
        }

        writeln!(out)?;
    }
    
    writeln!(out, "section .data")?;
    for global in cprog.globals_iter() {
        let pkg_sep = if global.is_const { '?' } else { '$' };

        if global.is_exported {
            writeln!(out, "global {package_name}{pkg_sep}{}", global.name)?;
            writeln!(out, "{package_name}{pkg_sep}{} equ GLOB_{}", global.name, global.pos)?;
        }

        if global.is_const {

            match global.init {
                CTimeVal::Int(int) => writeln!(out, "GLOB_{} equ {int}", global.pos)?,
                CTimeVal::Function { address, .. } => writeln!(out, "GLOB_{} equ OP_{address}", global.pos)?,
                CTimeVal::StringSlice(pointer, len) => {
                    writeln!(out, "GLOB_{} equ STR_{pointer}", global.pos)?;
                    writeln!(out, "GLOB_{} equ {len}", global.pos + 8)?;
                },
                _ => todo!(),
            }

        } else {

            match global.init {
                CTimeVal::Int(int) => writeln!(out, "GLOB_{}: dq {int}", global.pos)?,
                CTimeVal::Function { address, .. } => writeln!(out, "GLOB_{}: dq OP_{address}", global.pos)?,
                CTimeVal::StringSlice(pointer, len) => {
                    write!(out, "GLOB_{}: ", global.pos)?;
                    writeln!(out, "dq STR_{pointer}")?;
                    write!(out, "GLOB_{}: ", global.pos + 8)?;
                    writeln!(out, "dq {len}")?;
                },
                _ => todo!(),
            }

        }
    }

    writeln!(out, "section .text")?;

    for (i, node) in cprog.ir_iter().enumerate() {
        match node {
            IRNode::Nop => unreachable!(), // not an actual nop instruction, just a denotion for the optimizer
            IRNode::Return { params_size: 0 } => writeln!(out, "    OP_{i}: ret")?,
            IRNode::Return { params_size } => writeln!(out, "    OP_{i}: ret {params_size}")?,
            IRNode::CallFromOffset(offset) => writeln!(out, "    OP_{i}: call OP_{}", (i as i64) + offset)?,
            IRNode::JumpFromOffset(offset) => writeln!(out, "    OP_{i}: jmp OP_{}", (i as i64) + offset)?,
            IRNode::PushAddressFromOffset(offset) => writeln!(out, "    OP_{i}: push qword OP_{}", (i as i64) + offset)?,
            IRNode::Push64(int) => writeln!(out, "    OP_{i}: push qword {int}")?,
            IRNode::StackAlloc(size) => writeln!(out, "    OP_{i}: sub rsp, {size}")?,
            IRNode::StackDealloc(size) => writeln!(out, "    OP_{i}: add rsp, {size}")?,
            IRNode::Load64ToStack(int, offset) => writeln!(out, "    OP_{i}: mov qword [rsp+{offset}], {int}")?,
            IRNode::GlobalReadPush64(offset) => writeln!(out, "    OP_{i}: push qword [GLOB_{offset}]")?,
            IRNode::StackReadPush64(offset) => writeln!(out, "    OP_{i}: push qword [rsp+{offset}]")?,
            IRNode::PushStaticStringPointer(pos) => writeln!(out, "    OP_{i}: push qword STR_{pos}")?,
            
            IRNode::ExternalReadPush64(external) => {
                if external.is_const {
                    writeln!(out, "    OP_{i}: push {}?{}", external.package_name, external.name)?;
                } else {
                    writeln!(out, "    OP_{i}: push qword [{}${}]", external.package_name, external.name)?;
                }
            },

            IRNode::ExternalReadCall(external) => {
                if external.is_const {
                    writeln!(out, "    OP_{i}: call {}?{}", external.package_name, external.name)?;
                } else {
                    writeln!(out, "    OP_{i}: call qword [{}${}]", external.package_name, external.name)?;
                }
            },

            IRNode::Pop64ToStack(offset) => {
                writeln!(out, "OP_{i}:")?;
                writeln!(out, "    pop rax")?;
                writeln!(out, "    mov [rsp+{}], rax", offset - 8)?;
            },

            IRNode::GlobalReadLoad64ToStack(global_offset, offset) => {
                writeln!(out, "OP_{i}:")?;
                writeln!(out, "    mov rax, [GLOB_{global_offset}]")?;
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

            IRNode::JumpIfNot64FromOffset(offset) => {
                writeln!(out, "OP_{i}:")?;
                writeln!(out, "    pop rax")?;
                writeln!(out, "    test rax, rax")?;
                writeln!(out, "    jz OP_{}", (i as i64) + offset)?;
            },
            
            IRNode::PushStackPointer(offset) => {
                writeln!(out, "OP_{i}:")?;
                writeln!(out, "    lea rax, [rsp+{offset}]")?;
                writeln!(out, "    push rax")?;
            },

            IRNode::Deref64 => {
                writeln!(out, "OP_{i}:")?;
                writeln!(out, "    pop rax")?;
                writeln!(out, "    push qword [rax]")?;
            },

            IRNode::StackDeref64(offset) => {
                writeln!(out, "OP_{i}:")?;
                writeln!(out, "    mov rax, [rsp+{offset}]")?;
                writeln!(out, "    push qword [rax]")?;
            },

            IRNode::Add64 => {
                writeln!(out, "OP_{i}:")?;
                writeln!(out, "    pop rbx")?;
                writeln!(out, "    pop rax")?;
                writeln!(out, "    add rax, rbx")?;
                writeln!(out, "    push rax")?;
            },

            IRNode::Sub64 => {
                writeln!(out, "OP_{i}:")?;
                writeln!(out, "    pop rbx")?;
                writeln!(out, "    pop rax")?;
                writeln!(out, "    sub rax, rbx")?;
                writeln!(out, "    push rax")?;
            },
        }
    }

    Ok(())
}

