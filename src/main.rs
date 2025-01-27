use anyhow::Result;
use std::collections::HashMap;
use std::fmt;
use syscalls::{syscall, SyscallArgs, Sysno};
use thiserror::Error;

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
    Byte(u8),
    Bytes(Vec<u8>),
    Var(String),
    Assign(String, Box<BfNode>),
    Copy(String, String, usize),
    Ptr(String, isize),
    Syscall(Box<BfNode>, Vec<BfNode>),
    If(Box<BfNode>, Box<BfNode>),
    While(Box<BfNode>, Box<BfNode>),
    Eq(Box<BfNode>, Box<BfNode>),
    Lt(Box<BfNode>, Box<BfNode>),
    Block(Vec<BfNode>),
}

#[derive(Clone)]
pub struct MemoryCell {
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
    pub fn debug_output(&self) -> String {
        let indent_str = " ".repeat(self.indent * 4);
        format!("{}{} // {}", indent_str, self.bf, self.debug)
    }
}

pub struct BfCodeGenerator {
    pub output: Vec<CGOutput>,
    pub debug: bool,
    pub indent_level: usize,
    pub memory: Vec<u8>,
    pub cells: HashMap<String, MemoryCell>,
    pub current_ptr: usize,
    pub next_free_cell: usize,
}

impl BfCodeGenerator {
    pub fn new(debug: bool) -> Self {
        Self {
            output: Vec::new(),
            debug,
            indent_level: 0,
            memory: vec![],
            cells: HashMap::new(),
            current_ptr: 0,
            next_free_cell: 8,
        }
    }

    pub fn alloc_cells(&mut self, size: usize) -> Result<(usize, usize)> {
        let start = self.next_free_cell;
        self.next_free_cell += size;

        if self.memory.len() < self.next_free_cell {
            self.memory.resize(self.next_free_cell, 0);
        }
        Ok((start, size))
    }

    fn push_output(&mut self, bf: String, debug: String) {
        self.output.push(CGOutput {
            bf,
            debug,
            indent: self.indent_level,
        });
    }

    fn generate_ptr_move(&mut self, target: usize) -> CGOutput {
        let from = self.current_ptr;
        let bf = if target > from {
            ">".repeat(target - from)
        } else {
            "<".repeat(from - target)
        };
        self.current_ptr = target;

        CGOutput {
            bf,
            debug: format!("move ptr from {} to {}", from, target),
            indent: self.indent_level,
        }
    }

    fn generate_clear_cell(&self) -> CGOutput {
        CGOutput {
            bf: "[-]".to_string(),
            debug: "clear cell to 0".to_string(),
            indent: self.indent_level,
        }
    }

    fn generate_number(&self, value: u8) -> CGOutput {
        CGOutput {
            bf: "+".repeat(value as usize),
            debug: format!("set cell to {}", value),
            indent: self.indent_level,
        }
    }

    fn set_byte(&mut self, cell_pos: usize, value: u8) -> Vec<CGOutput> {
        vec![
            self.generate_ptr_move(cell_pos),
            self.generate_clear_cell(),
            self.generate_number(value),
        ]
    }

    fn set_bytes(&mut self, bytes: &[u8]) -> Vec<CGOutput> {
        let mut ret = vec![];
        let (start_pos, size) = self
            .alloc_cells(bytes.len())
            .expect("Failed to alloc for set_bytes");

        for (i, &byte) in bytes.iter().enumerate() {
            ret.extend(self.set_byte(start_pos + i, byte));
        }
        ret
    }

    fn copy_bytes_impl(&mut self, src_pos: usize, dst_pos: usize, size: usize) -> Result<()> {
        for offset in 0..size {
            let from = src_pos + offset;
            let to = dst_pos + offset;
            let temp = to + 1;

            self.indent_level += 1;

            let move_result = self.generate_ptr_move(from);
            self.output.push(move_result);

            {
                let mut bf = String::new();
                bf.push_str("[-");
                if to >= from {
                    bf.push_str(&">".repeat(to - from));
                } else {
                    bf.push_str(&"<".repeat(from - to));
                }
                bf.push('+');

                bf.push('>');
                bf.push('+');

                if to >= from {
                    bf.push_str(&"<".repeat(to - from + 1));
                } else {
                    bf.push_str(&">".repeat(from - to - 1));
                }

                bf.push(']');

                self.push_output(
                    bf,
                    format!("copy (first pass) from cell {} to cell {}", from, to),
                );
            }

            let move_to_from = self.generate_ptr_move(from);
            self.output.push(move_to_from);
            let move_to_temp = self.generate_ptr_move(temp);
            self.output.push(move_to_temp);

            {
                let mut bf = String::new();
                bf.push_str("[-");
                if from >= temp {
                    bf.push_str(&">".repeat(from - temp));
                } else {
                    bf.push_str(&"<".repeat(temp - from));
                }
                bf.push('+');

                if from >= temp {
                    bf.push_str(&"<".repeat(from - temp));
                } else {
                    bf.push_str(&">".repeat(temp - from));
                }
                bf.push(']');

                self.push_output(bf, format!("restore source at cell {}", from));
            }

            let move_to_temp_final = self.generate_ptr_move(temp);
            self.output.push(move_to_temp_final);
            let move_to_to = self.generate_ptr_move(to);
            self.output.push(move_to_to);
            self.indent_level -= 1;
        }

        Ok(())
    }

    fn set_i64(&mut self, cell_pos: usize, value: i64) -> Vec<CGOutput> {
        let bytes = value.to_le_bytes();
        let mut result = vec![];
        for (i, &byte) in bytes.iter().enumerate() {
            result.extend(self.set_byte(cell_pos + i, byte));
        }
        result
    }

    pub fn generate(&mut self, node: &BfNode) -> Result<()> {
        match node {
            BfNode::Byte(value) => {
                let (pos, _) = self.alloc_cells(1)?;
                let code = self.set_byte(pos, *value);
                self.output.extend(code);
            }

            BfNode::Bytes(bytes) => {
                let code = self.set_bytes(bytes);
                self.output.extend(code);
            }

            BfNode::Var(name) => {
                let cell = self
                    .cells
                    .get(name)
                    .ok_or_else(|| BfGenError::InvalidVarName)?
                    .clone();

                let (new_pos, _) = self.alloc_cells(cell.size)?;
                self.copy_bytes_impl(cell.ptr, new_pos, cell.size)?;
            }

            BfNode::Assign(var_name, expr) => {
                let before_alloc = self.next_free_cell;
                self.generate(expr)?;
                let after_alloc = self.next_free_cell;

                let size = after_alloc - before_alloc;
                let cell_ptr = before_alloc;

                self.cells.insert(
                    var_name.clone(),
                    MemoryCell {
                        ptr: cell_ptr,
                        size,
                    },
                );
            }

            BfNode::Copy(src_var, dst_var, size) => {
                let src_cell = self
                    .cells
                    .get(src_var)
                    .ok_or_else(|| BfGenError::InvalidVarName)?
                    .clone();
                let dst_cell = self
                    .cells
                    .get(dst_var)
                    .ok_or_else(|| BfGenError::InvalidVarName)?
                    .clone();

                if src_cell.size < *size || dst_cell.size < *size {
                    return Err(BfGenError::InvalidMemoryAccess.into());
                }

                self.copy_bytes_impl(src_cell.ptr, dst_cell.ptr, *size)?;
            }

            BfNode::Ptr(var_name, offset) => {
                let base_cell = self
                    .cells
                    .get(var_name)
                    .ok_or_else(|| BfGenError::InvalidVarName)?
                    .clone();

                let final_ptr = if offset.is_negative() {
                    base_cell.ptr.checked_sub(offset.unsigned_abs())
                } else {
                    base_cell.ptr.checked_add(*offset as usize)
                }
                .ok_or_else(|| BfGenError::InvalidMemoryAccess)?;

                // For now, we need to keep pointers within byte range since the VM treats everything as bytes
                if final_ptr > 255 {
                    return Err(BfGenError::InvalidMemoryAccess.into());
                }

                let (pos, _) = self.alloc_cells(1)?;
                let move_result = self.generate_ptr_move(pos);
                self.output.push(move_result);
                self.output.push(self.generate_clear_cell());

                self.output.push(CGOutput {
                    bf: "+".repeat(final_ptr),
                    debug: format!("set pointer address = {}", final_ptr),
                    indent: self.indent_level,
                });
            }

            BfNode::Syscall(num_expr, args_exprs) => {
                // First allocate space for the result at current position
                let result_pos = self.next_free_cell;
                let (_, _) = self.alloc_cells(8)?; // i64 size

                // Generate syscall number
                let before_num = self.next_free_cell;
                self.generate(num_expr)?;
                let syscall_num_cell = before_num;

                // Generate arguments
                let mut arg_cells = vec![];
                for arg_expr in args_exprs {
                    let before_arg = self.next_free_cell;
                    self.generate(arg_expr)?;
                    let after_arg = self.next_free_cell;
                    arg_cells.push(before_arg..after_arg);
                }

                // Trigger syscall
                {
                    // Set trigger
                    let move_to_zero = self.generate_ptr_move(0);
                    self.output.push(move_to_zero);
                    self.output.push(self.generate_clear_cell());
                    self.output.push(self.generate_number(255));

                    // Copy syscall number to position 1 (as i64)
                    let num_value = self.memory[syscall_num_cell];
                    let code = self.set_i64(1, num_value as i64);
                    self.output.extend(code);

                    // Copy arguments (each as i64)
                    let mut dest_pos = 9; // After trigger(1) and syscall_num(8)
                    for rng in arg_cells {
                        let value = self.memory[rng.start];
                        let code = self.set_i64(dest_pos, value as i64);
                        self.output.extend(code);
                        dest_pos += 8;
                    }

                    self.push_output(".".to_string(), "trigger syscall".to_string());

                    // Move back to result position
                    let move_to_result = self.generate_ptr_move(result_pos);
                    self.output.push(move_to_result);
                }
            }

            BfNode::If(cond_expr, body_expr) => {
                let before_cond = self.next_free_cell;
                self.generate(cond_expr)?;
                let cond_cell = before_cond;

                let move_result = self.generate_ptr_move(cond_cell);
                self.output.push(move_result);
                self.push_output("[".to_string(), "if: start loop".to_string());

                self.indent_level += 1;
                self.generate(body_expr)?;

                let move_back = self.generate_ptr_move(cond_cell);
                self.output.push(move_back);
                self.output.push(self.generate_clear_cell());

                self.indent_level -= 1;
                self.push_output("]".to_string(), "if: end loop".to_string());
            }

            BfNode::While(cond_expr, body_expr) => {
                let before_cond = self.next_free_cell;
                self.generate(cond_expr)?;
                let cond_cell = before_cond;

                let move_result = self.generate_ptr_move(cond_cell);
                self.output.push(move_result);
                self.push_output("[".to_string(), "while: start loop".to_string());

                self.indent_level += 1;
                self.generate(body_expr)?;
                self.generate(cond_expr)?;
                self.indent_level -= 1;

                self.push_output("]".to_string(), "while: end loop".to_string());
            }

            BfNode::Eq(a_expr, b_expr) => {
                let a_start = self.next_free_cell;
                self.generate(a_expr)?;
                let a_end = self.next_free_cell;
                let a_size = a_end - a_start;

                let b_start = self.next_free_cell;
                self.generate(b_expr)?;
                let b_end = self.next_free_cell;
                let b_size = b_end - b_start;

                if a_size == 1 && b_size == 1 {
                    let a_cell = a_start;
                    let b_cell = b_start;

                    let (result_cell, _) = self.alloc_cells(1)?;
                    self.copy_bytes_impl(a_cell, result_cell, 1)?;

                    let move_result = self.generate_ptr_move(b_cell);
                    self.output.push(move_result);

                    let mut bf = String::new();
                    bf.push_str("[-");
                    if result_cell >= b_cell {
                        bf.push_str(&">".repeat(result_cell - b_cell));
                    } else {
                        bf.push_str(&"<".repeat(b_cell - result_cell));
                    }
                    bf.push('-');
                    if result_cell >= b_cell {
                        bf.push_str(&"<".repeat(result_cell - b_cell));
                    } else {
                        bf.push_str(&">".repeat(b_cell - result_cell));
                    }
                    bf.push(']');

                    self.push_output(bf, "subtract b from result".to_string());
                } else {
                    self.push_output(
                        "".to_string(),
                        "eq on multi-byte: not implemented".to_string(),
                    );
                }
            }

            BfNode::Lt(a_expr, b_expr) => {
                let a_start = self.next_free_cell;
                self.generate(a_expr)?;
                let a_end = self.next_free_cell;

                let b_start = self.next_free_cell;
                self.generate(b_expr)?;
                let b_end = self.next_free_cell;

                if (a_end - a_start) == 1 && (b_end - b_start) == 1 {
                    self.push_output("".to_string(), "lt(a,b) single-byte: stub".to_string());
                } else {
                    self.push_output(
                        "".to_string(),
                        "lt on multi-byte: not implemented".to_string(),
                    );
                }
            }

            BfNode::Block(nodes) => {
                let block_start = CGOutput {
                    bf: "".to_string(),
                    debug: "begin block".to_string(),
                    indent: self.indent_level,
                };
                self.output.push(block_start);

                self.indent_level += 1;
                for n in nodes {
                    self.generate(n)?;
                }
                self.indent_level -= 1;

                let block_end = CGOutput {
                    bf: "".to_string(),
                    debug: "end block".to_string(),
                    indent: self.indent_level,
                };
                self.output.push(block_end);
            }
        }
        Ok(())
    }

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

pub struct BrainfuckVM {
    pub memory: Vec<u8>,
    pub ptr: usize,
    pub debug: bool,
    pub heap_start: usize,
}

impl BrainfuckVM {
    pub fn new(debug: bool, memory_size: usize) -> Self {
        Self {
            memory: vec![0; memory_size],
            ptr: 0,
            debug,
            // Reserve space for syscall trigger only, everything else is dynamic
            heap_start: 1,
        }
    }

    fn handle_syscall(&mut self) -> Result<()> {
        // byte trigger
        // u32 syscall_number (1..5)
        // usize syscall_arg_0 (5..13)
        // usize syscall_arg_1 (13..21)
        // usize syscall_arg_2 (21..29)
        // usize syscall_arg_3 (29..37)
        // usize syscall_arg_4 (37..45)
        // usize syscall_arg_5 (45..53)

        // usize syscall_result we put this where current pointer is
        // usize on macos aarch64 is 64-bit

        let syscall_num_start = 1;
        let syscall_num_end = syscall_num_start + size_of::<u32>();
        let syscall_num: u32 = u32::from_le_bytes(
            self.memory[syscall_num_start..syscall_num_end]
                .try_into()
                .unwrap(),
        );

        let syscall_arg_start = syscall_num_end + 1;
        let mut syscall_args = [0usize; 6];
        for (i, arg) in syscall_args.iter_mut().enumerate() {
            let start = syscall_arg_start + i * size_of::<usize>();
            let end = start + size_of::<usize>();
            *arg = usize::from_le_bytes(self.memory[start..end].try_into()?);
        }

        // size = 4, align = 0x4
        let nr = Sysno::from(syscall_num);
        let sa = SyscallArgs::new(
            syscall_args[0],
            syscall_args[1],
            syscall_args[2],
            syscall_args[3],
            syscall_args[4],
            syscall_args[5],
        );

        let result = unsafe {
            match syscall(nr, &sa) {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("Syscall failed: {}", e);
                    return Err(BfVmError::SyscallFailed.into());
                }
            }
        };

        // Write result to current pointer position
        let result_bytes = result.to_le_bytes();
        self.memory[self.ptr..self.ptr + size_of::<usize>()].copy_from_slice(&result_bytes);
        Ok(())
    }

    fn move_ptr(&mut self, offset: isize) -> Result<()> {
        let new_ptr = if offset >= 0 {
            self.ptr.checked_add(offset as usize)
        } else {
            self.ptr.checked_sub((-offset) as usize)
        };
        match new_ptr {
            Some(np) if np < self.memory.len() => {
                self.ptr = np;
                Ok(())
            }
            _ => Err(BfVmError::InvalidMemoryAccess.into()),
        }
    }

    pub fn run(&mut self, code: &str) -> Result<()> {
        let mut pc = 0;
        let instructions: Vec<char> = code.chars().collect();
        let mut bracket_stack = Vec::new();

        while pc < instructions.len() {
            match instructions[pc] {
                '>' => self.move_ptr(1)?,
                '<' => self.move_ptr(-1)?,
                '+' => {
                    self.memory[self.ptr] = self.memory[self.ptr].wrapping_add(1);
                }
                '-' => {
                    self.memory[self.ptr] = self.memory[self.ptr].wrapping_sub(1);
                }
                '.' => {
                    if self.ptr == 0 && self.memory[0] == 255 {
                        self.handle_syscall()?;
                    } else {
                        print!("{}", self.memory[self.ptr] as char);
                    }
                }
                ',' => {
                    let mut input = String::new();
                    std::io::stdin().read_line(&mut input)?;
                    if let Some(ch) = input.chars().next() {
                        self.memory[self.ptr] = ch as u8;
                    }
                }
                '[' => {
                    if self.memory[self.ptr] == 0 {
                        let mut depth = 1;
                        while depth > 0 && pc < instructions.len() {
                            pc += 1;
                            match instructions[pc] {
                                '[' => depth += 1,
                                ']' => depth -= 1,
                                _ => {}
                            }
                        }
                    } else {
                        bracket_stack.push(pc);
                    }
                }
                ']' => {
                    if self.memory[self.ptr] != 0 {
                        if let Some(start) = bracket_stack.last() {
                            pc = *start;
                            continue;
                        }
                    } else {
                        bracket_stack.pop();
                    }
                }
                _ => {}
            }
            pc += 1;
        }
        Ok(())
    }
}

fn main() -> Result<()> {
    let message = "Hello World!".as_bytes().to_vec();

    let program = BfNode::Block(vec![
        BfNode::Assign("trigger".to_string(), Box::new(BfNode::Byte(0))),
        BfNode::Assign(
            "syscall_num".to_string(),
            Box::new(BfNode::Bytes(0u32.to_le_bytes().to_vec())),
        ),
        BfNode::Assign(
            "syscall_arg_0".to_string(),
            Box::new(BfNode::Bytes(0usize.to_le_bytes().to_vec())),
        ),
        BfNode::Assign(
            "syscall_arg_1".to_string(),
            Box::new(BfNode::Bytes(0usize.to_le_bytes().to_vec())),
        ),
        // syscall_arg_2
        BfNode::Assign(
            "syscall_arg_2".to_string(),
            Box::new(BfNode::Bytes(0usize.to_le_bytes().to_vec())),
        ),
        // syscall_arg_3
        BfNode::Assign(
            "syscall_arg_3".to_string(),
            Box::new(BfNode::Bytes(0usize.to_le_bytes().to_vec())),
        ),
        // syscall_arg_4
        BfNode::Assign(
            "syscall_arg_4".to_string(),
            Box::new(BfNode::Bytes(0usize.to_le_bytes().to_vec())),
        ),
        // syscall_arg_5
        BfNode::Assign(
            "syscall_arg_5".to_string(),
            Box::new(BfNode::Bytes(0usize.to_le_bytes().to_vec())),
        ),
        // syscall_result
        BfNode::Assign(
            "syscall_result".to_string(),
            Box::new(BfNode::Bytes(0usize.to_le_bytes().to_vec())),
        ),
        // our program
        BfNode::Assign(
            "message".to_string(),
            Box::new(BfNode::Bytes(message.clone())),
        ),
        BfNode::Assign(
            "syscall_result".to_string(),
            Box::new(BfNode::Syscall(
                Box::new(BfNode::Byte(1)),
                vec![
                    BfNode::Byte(1),
                    BfNode::Var("message".to_string()),
                    BfNode::Byte(message.len() as u8),
                ],
            )),
        ),
    ]);

    let mut gen = BfCodeGenerator::new(true);
    gen.generate(&program)?;

    println!("=== Generated Brainfuck Code (Debug Mode) ===");
    println!("{}", gen.get_formatted_output());
    println!("\n=== Execution Result ===");

    let mut vm = BrainfuckVM::new(true, 1024 * 1024);
    vm.run(&gen.get_formatted_output())?;

    println!("{:?}", vm.memory[0..32].to_vec());

    Ok(())
}
