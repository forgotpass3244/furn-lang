use std::{env::args, fs::{DirBuilder, File}};

use crate::{flags::Flags, ir_gen::{ir_gen::IRGen, ir_optimizer::IROptimizer}, lexer::{lexer::Lexer, tokens::Tokens}, outputs::asm_x86_64::gen_asm_x86_64_from_ir, parser::parser::Parser, tok::token_other::TokenOther};
pub mod flags;
pub mod lexer;
pub mod tok;
pub mod parser;
pub mod ir_gen;
pub mod outputs;

fn main() {
    let flags = Flags::parse_args(args());

    let token_map = TokenOther::make_token_map();
    let mut lexer = Lexer::from_file(flags.file_name.unwrap().as_str());
    let tokens: Tokens<TokenOther> = lexer.tokenize(token_map);

    let mut parser = Parser::new(&tokens);
    let ast = parser.parse().unwrap();

    let mut ir_gen = IRGen::new();
    let mut cprog = ir_gen.generate(&ast);

    let mut ir_optimizer = IROptimizer::new(&mut cprog);
    let cprog = ir_optimizer.optimize();

    let out_dir = DirBuilder::new();
    _ = out_dir.create("out");
    
    let mut out_file = File::create("out/out.asm").unwrap();
    gen_asm_x86_64_from_ir(&mut out_file, cprog).unwrap();
}
