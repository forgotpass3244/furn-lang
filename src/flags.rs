
use std::{collections::VecDeque, env::Args};
use crate::maybe_inf::MaybeInf;

#[derive(Clone)]
pub enum CompilationTarget {
    None,
    LinuxX86_64,
}

#[derive(Clone)]
pub struct Flags {
    args: VecDeque<String>,
    pub program_name: String,
    pub file_name: Option<String>,
    pub optimization_level: Option<MaybeInf<u32>>,
    pub target: Option<CompilationTarget>,
    pub print_ir: bool,
    pub output_file_name: Option<String>,
    pub compile_iters: Option<u64>,
}

impl Flags {
    pub fn with_args(args: Args) -> Self {
        let mut args: VecDeque<String> = args.collect();
        let program_name = args.pop_front().unwrap(); // ignore program name

        Self {
            args,
            program_name,
            file_name: None,
            optimization_level: None,
            target: None,
            print_ir: false,
            output_file_name: None,
            compile_iters: None,
        }
    }

    fn split_args(&mut self) {

        let mut new_args = VecDeque::new();

        while let Some(mut arg) = self.args.pop_front() {
            if !arg.starts_with("--") && arg.starts_with("-") && arg.len() > 2 {
                let ch = arg.chars().nth(1).unwrap();
                if ch.is_uppercase() {
                    let sub_arg = arg.split_off(2);
                    new_args.push_back(arg);
                    new_args.push_back(sub_arg);
                } else {
                    for ch in arg.get(1..).unwrap().chars() {
                        let mut synthetic_arg = String::from("-");
                        synthetic_arg.push(ch);
                        new_args.push_back(synthetic_arg);
                    }
                }
            } else {
                new_args.push_back(arg); 
            }
        }

        self.args = new_args;

    }

    pub fn parse_args(args: Args) -> Self {
        let mut flags = Self::with_args(args);
        flags.parse();
        flags
    }

    pub fn parse(&mut self) {

        self.split_args();

        while !self.args.is_empty() {
            let arg = self.args.pop_front().unwrap_or_default();
            match arg.as_str() {

                "-O" => {
                    let string = self.args.pop_front();
                    if let Some(string) = string {
                        let optimization_level: u32 = string.parse().unwrap_or_default();
                        self.optimization_level = Some(MaybeInf::NonInf(optimization_level));
                    }
                }

                "-o" => {
                    let string = self.args.pop_front();
                    if let Some(string) = string {
                        self.output_file_name = Some(string);
                    }
                }

                "-T" => {
                    let string = self.args.pop_front();
                    if let Some(string) = string {
                        self.target = match string.as_str() {
                            "none" => Some(CompilationTarget::None),
                            "linux_x86_64" => Some(CompilationTarget::LinuxX86_64),
                            _ => None,
                        };
                    }
                }
                
                "--print-ir" => {
                    self.print_ir = true;
                }
                
                "--optimized" => {
                    self.optimization_level = Some(MaybeInf::Inf);
                }
                
                "--iterations" => {
                    let string = self.args.pop_front();
                    if let Some(string) = string {
                        let compile_iters: u64 = string.parse().unwrap_or_default();
                        self.compile_iters = Some(compile_iters);
                    }
                }

                _ => self.file_name = Some(arg),
            }
        }
    }
}

