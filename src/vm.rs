use crate::tokenizer::{self, optimize, Token};

use std::{
    fs::File,
    io::{Read, Write},
    mem::size_of,
    str::FromStr,
};

const MEMORY_SIZE: usize = 4 * 1024 * 1024;

#[derive(Debug, thiserror::Error)]
pub enum VmError {
    #[error("Instruction Is Null")]
    InstructionIsNull,

    #[error("Read File Error")]
    IO(#[from] std::io::Error),

    #[error("Token Error")]
    Token(#[from] crate::tokenizer::TokenizerError),

    #[error("Pointer OverFlow Error")]
    PointerOverFlow,
}

pub struct VM {
    inst_len: usize,  // instruction length
    inst: Vec<Token>, // instruction to run
    mem_len: usize,   // memory length
    mem: Box<[u8]>,   // memory buffer
}

impl VM {
    pub fn new(inst: Vec<Token>) -> Result<Self, VmError> {
        if inst.len() == 0 {
            return Err(VmError::InstructionIsNull);
        }

        let mem = vec![0 as u8; MEMORY_SIZE].into_boxed_slice();
        Ok(VM {
            mem_len: mem.len(),
            mem,
            inst_len: inst.len(),
            inst,
        })
    }

    pub fn new_from_file(path: &String) -> Result<Self, VmError> {
        let mut file = File::open(path).expect("file not found");
        let mut src = String::new();
        file.read_to_string(&mut src).expect("failed to read file");
        let mut tokens = tokenizer::tokenizer(&src)?;
        optimize(&mut tokens);
        Self::new(tokens)
    }

    pub fn run(&mut self) -> Result<(), VmError> {
        let mut pc = 0;
        let mut point = 0;

        use crate::tokenizer::Token::*;
        while pc < self.inst_len {
            match self.inst[pc] {
                IncrementData(x) => {
                    self.mem[point] += x;
                }
                DecrementData(x) => {
                    self.mem[point] -= x;
                }
                IncrementPointer(x) => {
                    if point + x >= self.mem_len {
                        return Err(VmError::PointerOverFlow);
                    }
                    point += x;
                }
                DecrementPointer(x) => {
                    if ((point + x) >> (size_of::<usize>() - 1)) == 0xf {
                        return Err(VmError::PointerOverFlow);
                    }
                    point -= x;
                }
                Output => {
                    let mut buf = [0_u8];
                    buf[0] = self.mem[point];
                    match std::io::stdout().write_all(&buf) {
                        Ok(()) => {}
                        Err(e) => return Err(VmError::IO(e)),
                    }
                }
                Input => {
                    let mut buf = [0_u8];
                    match std::io::stdin().read(&mut buf) {
                        Ok(0) => {}
                        Ok(1) => {
                            self.mem[point] = buf[0];
                        }
                        Err(e) => return Err(VmError::IO(e)),
                        _ => unreachable!(),
                    }
                }
                LoopStart(x) => {
                    if self.mem[point] == 0 {
                        if x as usize <= self.inst_len {
                            pc = x as usize;
                        }
                    }
                }
                LoopEnd(x) => {
                    if self.mem[point] != 0 {
                        if x as usize <= self.inst_len {
                            pc = x as usize;
                        }
                    }
                }
            }
            pc += 1;
        }
        Ok(())
    }
}

#[test]
fn test_vm_run() {
    let file = String::from_str("bfcode\\hellow.bf").unwrap();
    let vm = VM::new_from_file(&file);
    vm.unwrap().run();
}
