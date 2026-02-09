
use std::{collections::VecDeque, env::Args};

pub struct Flags {
    args: VecDeque<String>,
    pub program_name: String,
    pub file_name: Option<String>,
}

impl Flags {
    pub fn with_args(args: Args) -> Self {
        let mut argv: VecDeque<String> = args.collect();
        let program_name = argv.pop_front().unwrap(); // ignore program name

        Self {
            args: argv,
            program_name,
            file_name: None,
        }
    }

    pub fn parse_args(args: Args) -> Self {
        let mut flags = Self::with_args(args);
        flags.parse();
        flags
    }

    pub fn parse(&mut self) {
        while !self.args.is_empty() {
            self.file_name = self.args.pop_front();
        }
    }
}

