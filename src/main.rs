use std::{env, process::exit};

use vm::VM;

pub mod jit;
pub mod tokenizer;
pub mod vm;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        println!("usage bfjit <file.bf>");
        exit(1);
    }

    let filepath = &args[1];
    VM::new_from_file(filepath).expect("build vm failed").run();
}
