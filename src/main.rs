use std::{env::args, fs::{DirBuilder, File}, io::{BufWriter, Write}, process::{Command, Stdio}, time::Instant};

use crate::{flags::{CompilationTarget, Flags}, ir_gen::{cmpld_program::CompiledProgram, ir_gen::IRGen, ir_optimizer::IROptimizer}, lexer::{lexer::Lexer, tokens::Tokens}, outputs::asm_x86_64::gen_asm_x86_64_from_ir, parser::parser::Parser, tok::token_other::TokenOther};
pub mod flags;
pub mod lexer;
pub mod tok;
pub mod parser;
pub mod ir_gen;
pub mod outputs;

fn clear_line() {
    print!("\x1B[1A"); // move cursor up 1 line
    print!("\x1B[2K"); // clear entire line
}

#[allow(dead_code)]
fn clear_lines(n: usize) {
    for _ in 0..n {
        clear_line();
    }
}

fn output_program(cprog: &CompiledProgram, target: CompilationTarget) {
    let out_dir = DirBuilder::new();
    _ = out_dir.create("furnbuild");
    
    let out_file = File::create("furnbuild/out.asm").unwrap();
    let mut buffer = BufWriter::new(out_file);
    
    match target {
        CompilationTarget::LinuxX86_64 => gen_asm_x86_64_from_ir(&mut buffer, cprog).unwrap(),
        _ => unreachable!(),
    }

    buffer.flush().unwrap();

    let mut assembler = Command::new("nasm");
    assembler.stdout(Stdio::inherit()).stderr(Stdio::inherit());
    assembler.arg("-felf64").arg("furnbuild/out.asm").arg("-ofurnbuild/out.o");

    match assembler.output().expect("Assembler command failed to start (make sure you have NASM installed)").status.code() {
        Some(0) => {
            clear_line();
            println!(":: Linking object files...")
        },
        _ => {
            println!(":: Assembler errors ^^^^^^^^^^^^^^^^^^^^");
            return
        },
    }

    let mut linker = Command::new("ld");
    linker.stdout(Stdio::inherit()).stderr(Stdio::inherit());
    linker.arg("rt.o").arg("furnbuild/out.o").arg("-ofurnbuild/out");

    match linker.output().expect("Command ld failed to start").status.code() {
        Some(0) => {
            clear_line();
            println!(":: Build complete.")
        },
        _ => println!(":: Linker errors (make sure the Furn Runtime is compiled) ^^^^^^^^^^^^^^^^^^^^"),
    }

    clear_line();
}

fn main() {
    let flags = Flags::parse_args(args());
    let start = Instant::now();

    println!(":: Lexing...");

    let token_map = TokenOther::make_token_map();
    let mut lexer = Lexer::from_file(flags.file_name.unwrap().as_str());
    let tokens: Tokens<TokenOther> = lexer.tokenize(token_map);

    clear_line();
    println!(":: Parsing...");

    let mut parser = Parser::new(&tokens);
    let ast = parser.parse();
    
    if parser.has_errors() {
        println!(":: Parse errors, aborting.");
        return
    }

    clear_line();
    println!(":: Generating IR...");

    let mut ir_gen = IRGen::new();
    let mut cprog = ir_gen.generate(&ast);

    clear_line();
    println!(":: Optimizing IR...");

    let mut ir_optimizer = IROptimizer::new(&mut cprog);
    let cprog = ir_optimizer.optimize(flags.optimization_level);

    if flags.print_ir {
        clear_line();
        for node in cprog.ir_iter() {
            println!("{node:?} ");
        }

        // balance for clear_line
        println!();
    }

    let duration = start.elapsed();
    clear_line();

    if let Some(target) = flags.target {
        match target {
            CompilationTarget::None => {
                println!(":: IR compilation took {}ms to complete.", duration.as_millis());
            },
            _ => {
                println!(":: IR compilation took {}ms to complete. Assembling...", duration.as_millis());
                output_program(cprog, target)
            },
        }
    } else {
        println!(":: IR compilation took {}ms to complete. Building...", duration.as_millis());
        output_program(cprog, CompilationTarget::LinuxX86_64);
    }
}
