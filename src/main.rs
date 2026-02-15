use std::{env::args, fs::{DirBuilder, File}, process::{Command, Stdio}, time::Instant};

use crate::{flags::Flags, ir_gen::{ir_gen::IRGen, ir_optimizer::IROptimizer}, lexer::{lexer::Lexer, tokens::Tokens}, outputs::asm_x86_64::gen_asm_x86_64_from_ir, parser::parser::Parser, tok::token_other::TokenOther};
pub mod flags;
pub mod lexer;
pub mod tok;
pub mod parser;
pub mod ir_gen;
pub mod outputs;

fn main() {
    let flags = Flags::parse_args(args());
    let start = Instant::now();

    println!(":: Lexing...");

    let token_map = TokenOther::make_token_map();
    let mut lexer = Lexer::from_file(flags.file_name.unwrap().as_str());
    let tokens: Tokens<TokenOther> = lexer.tokenize(token_map);

    println!(":: Parsing...");

    let mut parser = Parser::new(&tokens);
    let ast = parser.parse().unwrap();

    println!(":: Generating IR...");

    let mut ir_gen = IRGen::new();
    let mut cprog = ir_gen.generate(&ast);

    println!(":: Optimizing IR...");

    let mut ir_optimizer = IROptimizer::new(&mut cprog);
    let cprog = ir_optimizer.optimize();

    for node in cprog.ir_iter() {
        println!("{node:?} ");
    }

    let duration = start.elapsed();
    println!(":: IR compilation took {}mcs to complete.", duration.as_micros());

    let out_dir = DirBuilder::new();
    _ = out_dir.create("furnbuild");
    
    let mut out_file = File::create("furnbuild/out.asm").unwrap();
    gen_asm_x86_64_from_ir(&mut out_file, cprog).unwrap();

    let mut assembler = Command::new("nasm");
    assembler.stdout(Stdio::inherit()).stderr(Stdio::inherit());
    assembler.arg("-felf64").arg("furnbuild/out.asm").arg("-ofurnbuild/out.o");

    match assembler.output().expect("Assembler command failed to start (make sure you have NASM installed)").status.code() {
        Some(0) => println!(":: Linking object files..."),
        _ => {
            println!(":: Assembler errors ^^^^^^^^^^^^^^^^^^^^");
            return
        },
    }

    let mut linker = Command::new("ld");
    linker.stdout(Stdio::inherit()).stderr(Stdio::inherit());
    linker.arg("rt.o").arg("furnbuild/out.o").arg("-ofurnbuild/out");

    match linker.output().expect("Command ld failed to start").status.code() {
        Some(0) => println!(":: Build complete."),
        _ => println!(":: Linker errors (make sure the Furn Runtime is compiled) ^^^^^^^^^^^^^^^^^^^^"),
    }
}
