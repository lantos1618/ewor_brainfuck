use anyhow::Result;
use std::collections::HashMap;
use std::fmt;
use std::io::Write;
use syscalls::{syscall, SyscallArgs, Sysno};
use thiserror::Error;

// errors
#[derive(Error, Debug)]
pub enum BfGenError {
    #[error("Invalid syscall number")]
    InvalidSyscallNumber,
    #[error("Invalid variable name")]
    InvalidVarName,
    #[error("Invalid memory access")]
    InvalidMemoryAccess,
    #[error("Invalid assign value")]
    InvalidAssignValue,
}

#[derive(Error, Debug)]
pub enum BfVmError {
    #[error("Syscall failed")]
    SyscallFailed,
    #[error("Invalid memory access")]
    InvalidMemoryAccess,
    #[error("Invalid syscall number")]
    InvalidSyscallNumber,
}

#[derive(Debug, Clone)]
pub enum BfNode {
    Bytes(Vec<u8>),
    Var(String),
    Assign(String, Box<BfNode>),
    Syscall(Box<BfNode>, Vec<BfNode>),
    Block(Vec<BfNode>),
}

#[derive(Clone)]
pub struct MemoryCell {
    pub value: usize,
    pub ptr: usize,
    pub size: usize,
}

pub struct CGOutput {
    pub bf: String,
    pub debug: String,
    pub indent: usize,
}

impl fmt::Display for CGOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.bf)
    }
}

impl CGOutput {
    // Helper to get debug output with BF code and debug info interleaved
    pub fn debug_output(&self) -> String {
        format!(
            "{:indent$}{} // {}",
            "",
            self.debug,
            self.bf,
            indent = self.indent * 4
        )
    }
}

pub struct BfCodeGenerator {
    pub output: Vec<CGOutput>,
    pub debug: bool,
    pub indent_level: usize,
    pub memory: Vec<u8>,
    pub cells: HashMap<String, MemoryCell>,
    pub next_alloc: usize, // Track next allocation position
}

impl BfCodeGenerator {
    pub fn new(debug: bool) -> Self {
        Self {
            output: Vec::new(),
            debug,
            indent_level: 0,
            memory: Vec::new(),
            cells: HashMap::new(),
            next_alloc: 0,
        }
    }

    pub fn alloc_cells(&mut self, size: usize) -> Result<(usize, usize)> {
        // Allocate from heap area
        let start = self.next_alloc;
        self.next_alloc += size;
        self.memory.extend_from_slice(&vec![0; size]);
        Ok((start, size))
    }

    fn push_output(&mut self, bf: String, debug: String) {
        self.output.push(CGOutput {
            bf,
            debug,
            indent: self.indent_level,
        });
    }

    fn generate_ptr_move(&self, from: usize, to: usize) -> CGOutput {
        if to > from {
            CGOutput {
                bf: ">".repeat(to - from),
                debug: format!("moving ptr {} to {}", from, to),
                indent: self.indent_level,
            }
        } else {
            CGOutput {
                bf: "<".repeat(from - to),
                debug: format!("moving ptr {} to {}", from, to),
                indent: self.indent_level,
            }
        }
    }

    pub fn generate_number(&self, value: u8) -> CGOutput {
        CGOutput {
            bf: String::from("+").repeat(value as usize),
            debug: format!("setting cell to {}", value),
            indent: self.indent_level,
        }
    }

    pub fn generate_clear_cell(&self) -> CGOutput {
        CGOutput {
            bf: String::from("[-]"),
            debug: format!("clearing cell"),
            indent: self.indent_level,
        }
    }

    fn generate_byte(&self, pos: usize, value: u8) -> Vec<CGOutput> {
        let mut result = Vec::new();
        result.push(self.generate_ptr_move(pos, pos));
        result.push(self.generate_clear_cell());
        result.push(self.generate_number(value));
        result
    }

    // generate the code to set the bytes
    pub fn generate_bytes(&mut self, bytes: &[u8]) -> Vec<CGOutput> {
        let mut result = Vec::new();

        // request memory for the bytes
        let (start, size) = self.alloc_cells(bytes.len()).unwrap();
        self.memory[start..start + size].copy_from_slice(&bytes);

        // generate the code to set the bytes
        for i in 0..bytes.len() {
            result.extend(self.generate_byte(start + i, bytes[i]));
        }

        result
    }

    // Helper functions for common BF operations
    fn copy_byte(&mut self, from: usize, to: usize) -> Result<()> {
        self.indent_level += 1;
        todo!();
        Ok(())
    }

    fn copy_bytes(&mut self, from: usize, to: usize, size: usize) -> Result<()> {
        for i in 0..size {
            self.copy_byte(from + i, to + i)?;
        }
        Ok(())
    }

    // Now we can simplify the generate method using these helpers
    pub fn generate(&mut self, node: &BfNode) -> Result<()> {
        match node {
            BfNode::Bytes(v) => {
                todo!();
                Ok(())
            }
            BfNode::Var(name) => {
                todo!();
                Ok(())
            }
            BfNode::Assign(name, value) => {
                todo!();
                Ok(())
            }
            BfNode::Syscall(num, args) => {
                todo!();
                Ok(())
            }
            BfNode::Block(nodes) => {
                self.push_output("".to_string(), "begin block".to_string());
                self.indent_level += 1;

                for node in nodes {
                    self.generate(node)?;
                }

                self.indent_level -= 1;
                self.push_output("".to_string(), "end block".to_string());
                Ok(())
            }
        }
    }

    // Helper to get output in appropriate format based on debug mode
    pub fn get_formatted_output(&self) -> String {
        if self.debug {
            self.output
                .iter()
                .map(|o| o.debug_output())
                .collect::<Vec<_>>()
                .join("\n")
        } else {
            self.output
                .iter()
                .map(|o| o.bf.as_str())
                .collect::<Vec<_>>()
                .join("")
        }
    }
}

// layout
// | do_syscall == 255 | syscall_num | args[6] | heap |
pub struct BrainfuckVM {
    pub memory: Vec<u8>,
    pub ptr: usize,
    pub debug: bool,
    pub heap_start: usize,
}

// Memory layout constants
impl BrainfuckVM {
    pub fn new(debug: bool, memory_size: usize) -> Self {
        Self {
            memory: vec![0; memory_size],
            ptr: 0,
            debug,
            heap_start: 0,
        }
    }

    fn move_ptr(&mut self, offset: isize) -> Result<()> {
        let new_ptr = if offset >= 0 {
            self.ptr.checked_add(offset as usize)
        } else {
            self.ptr.checked_sub(offset.unsigned_abs())
        };

        match new_ptr {
            Some(ptr) if ptr < self.memory.len() => {
                self.ptr = ptr;
                Ok(())
            }
            _ => Err(BfVmError::InvalidMemoryAccess.into()),
        }
    }

    pub fn run(&mut self, code: &str) -> Result<()> {
        let mut pos = 0;
        while pos < code.len() {
            let ins = code.chars().nth(pos).unwrap();
            match ins {
                '>' => self.move_ptr(1)?,
                '<' => self.move_ptr(-1)?,
                '+' => {
                    self.memory[self.ptr] = self.memory[self.ptr].wrapping_add(1);
                }
                '-' => {
                    self.memory[self.ptr] = self.memory[self.ptr].wrapping_sub(1);
                }
                '.' => {
                    if self.memory[self.ptr] == 255 {
                        self.handle_syscall()?;
                    } else {
                        print!("{}", self.memory[self.ptr] as char);
                    }
                }
                ',' => {
                    let mut input = String::new();
                    std::io::stdin().read_line(&mut input)?;
                    if let Some(c) = input.chars().next() {
                        self.memory[self.ptr] = c as u8;
                    }
                }
                '[' => {
                    todo!();
                }
                ']' => {
                    todo!();
                }
                _ => {}
            }
            pos += 1;
        }
        Ok(())
    }

    fn handle_syscall(&mut self) -> Result<()> {
        let syscall_num = self.memory[self.ptr];
        let args: Vec<usize> = self.memory[self.ptr + 1..self.ptr + 7]
            .iter()
            .map(|&x| x as usize)
            .collect();

        let nr = Sysno::from(syscall_num as i32);
        let args = SyscallArgs::new(args[0], args[1], args[2], args[3], args[4], args[5]);

        let result = unsafe {
            match syscall(nr, &args) {
                Ok(result) => result,
                Err(e) => {
                    eprintln!("Syscall failed: {}", e);
                    return Err(BfVmError::SyscallFailed.into());
                }
            }
        };

        self.memory[self.ptr] = result as u8;
        Ok(())
    }
}

fn main() {
    // Test program that outputs "Hello World!"
    let message = "Hello World!".as_bytes().to_vec();
    let program = BfNode::Block(vec![
        // Store "Hello" in memory
        BfNode::Assign(
            "message".to_string(),
            Box::new(BfNode::Bytes(message.clone())),
        ),
        BfNode::Syscall(
            Box::new(BfNode::Bytes(vec![1 as u8])),
            vec![
                BfNode::Var("message".to_string()),
                BfNode::Bytes(vec![message.len() as u8]),
            ],
        ),
    ]);

    let mut gen = BfCodeGenerator::new(true);
    gen.generate(&program).unwrap();

    println!("Generated code:\n{}", gen.get_formatted_output());
    println!("\nProgram output:");

    let mut vm = BrainfuckVM::new(true, 1024 * 1024);
    vm.run(&gen.get_formatted_output()).unwrap();
}
