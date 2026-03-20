
use std::{env::{self, args}, fs::{self, File}, io::{BufWriter, Write}, process::{Command, Stdio}, time::Instant, hint::black_box};

use crate::{flags::{CompilationTarget, Flags}, ir_gen::{cmpld_program::CompiledProgram, ir_gen::IRGen, ir_optimizer::IROptimizer}, lexer::{lexer::Lexer, tokens::Tokens}, maybe_inf::MaybeInf, outputs::asm_x86_64::gen_asm_x86_64_from_ir, parser::parser::Parser, tok::token_other::TokenOther};
pub mod flags;
pub mod maybe_inf;
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

fn output_program(cprog: &CompiledProgram, target: CompilationTarget, flags: &Flags) {
    let mut out_dir = env::temp_dir();
    out_dir.push("furn-build-artifacts");
    _ = fs::create_dir(&out_dir);
    
    let out_file = File::create(out_dir.join("out.asm")).unwrap();
    let mut buffer = BufWriter::new(out_file);
    
    match target {
        CompilationTarget::LinuxX86_64 => gen_asm_x86_64_from_ir(&mut buffer, cprog).unwrap(),
        CompilationTarget::Windows => gen_asm_x86_64_from_ir(&mut buffer, cprog).unwrap(),
        _ => unreachable!(),
    }

    buffer.flush().unwrap();

    let obj_file = match target {
        CompilationTarget::LinuxX86_64 => "out.o",
        CompilationTarget::Windows => "out.obj",
        _ => unreachable!(),
    };

    let mut assembler = Command::new("nasm");
    assembler.stdout(Stdio::inherit()).stderr(Stdio::inherit());
    assembler.arg(match target {
        CompilationTarget::LinuxX86_64 => "-felf64",
        CompilationTarget::Windows => "-fwin64",
        _ => unreachable!(),
    });
    assembler.arg(out_dir.join("out.asm")).arg("-o".to_string() + out_dir.join(obj_file).to_str().unwrap());

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

    let mut linker = Command::new(match target {
        CompilationTarget::LinuxX86_64 => "ld",
        CompilationTarget::Windows => "gcc",
        _ => unreachable!(),
    });
    linker.stdout(Stdio::inherit()).stderr(Stdio::inherit());
    
    let output_path = flags.output_file_name.clone().unwrap_or_else(|| {
        flags.file_name.clone()
        .unwrap_or("unnamed".to_string())
        .rsplitn(2, ".").last().unwrap_or(
            flags.file_name.clone()
            .unwrap_or_default().as_str()
        ).to_string()
    }) + match target {
        CompilationTarget::LinuxX86_64 => None,
        CompilationTarget::Windows => Some(".exe"),
        _ => unreachable!(),
    }.unwrap_or_default();

    let runtime_file = match target {
        CompilationTarget::LinuxX86_64 => "runtimes/furn_rt_linuxx86_64.o",
        CompilationTarget::Windows => "runtimes\\furn_rt_win64.obj",
        _ => unreachable!(),
    };
    
    linker.arg(runtime_file).arg(out_dir.join(obj_file)).arg("-o").arg(output_path);

    match linker.output().expect("Linker command failed to start").status.code() {
        Some(0) => {
            clear_line();
            println!(":: Build complete.")
        },
        _ => println!(":: Linker errors (make sure the Furn Runtime is compiled) ^^^^^^^^^^^^^^^^^^^^"),
    }

    clear_line();
}


fn compile(flags: &Flags) {
    let start = Instant::now();

    println!(":: Lexing...");

    let token_map = TokenOther::make_token_map();
    let mut lexer = Lexer::from_file(flags.file_name.as_ref().unwrap().as_str());
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
    let mut cprog = ir_gen.generate(&ast).clone();

    if ir_gen.has_errors() {
        return
    }

    clear_line();
    println!(":: Optimizing IR...");

    let mut ir_optimizer = IROptimizer::new(&mut cprog);
    let cprog = match &flags.target {
        Some(CompilationTarget::None) => if flags.optimization_level.is_some() {
            ir_optimizer.optimize(flags.optimization_level)
        } else {
            ir_optimizer.optimize(Some(MaybeInf::NonInf(0)))
        },
        _ => ir_optimizer.optimize(flags.optimization_level),
    };

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

    if let Some(target) = &flags.target {
        match target {
            CompilationTarget::None => {
                println!(":: IR compilation took {}ms to complete.", duration.as_millis());
            },
            _ => {
                println!(":: IR compilation took {}ms to complete. Assembling...", duration.as_millis());
                output_program(cprog, target.clone(), &flags)
            },
        }
    } else {
        println!(":: IR compilation took {}ms to complete. Building...", duration.as_millis());
        output_program(cprog, CompilationTarget::LinuxX86_64, &flags);
    }
}

fn main() {
    let mut args = args();
    if args.len() > 1 {
        let flags = Flags::parse_args(args);
        for _ in 0..(flags.compile_iters.unwrap_or(1)) {
            black_box(compile(&flags));
        }
    } else {
        let program_name = args.nth(0);
        if let Some(program_name) = program_name {
            println!("usage:\n{program_name} <file> [ flags... ]")
        }
    }
}






