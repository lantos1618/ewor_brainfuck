use crate::{
    errors::{Result, VMError, VMResult},
    SyscallNum,
};
use anyhow::Context;
use std::{io::Read, os::fd::OwnedFd};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BFMode {
    Normal,
    Syscall,
}

pub struct BFVM {
    mode: BFMode,
    memory: Vec<u32>,
    ptr: u32,
    output: String,
    pc: usize,
    execution_steps: u64,
    max_steps: u64,
    fds: Vec<OwnedFd>,
}

// | syscall_result | syscall_number | syscall_args | heap...|

impl BFVM {
    pub fn new(memory_size: usize) -> Self {
        Self {
            mode: BFMode::Normal,
            memory: vec![0; memory_size],
            ptr: 0,
            output: String::new(),
            pc: 0,
            execution_steps: 0,
            max_steps: 1_000_000,
            fds: Vec::new(),
        }
    }

    pub fn run(&mut self, code: &str) -> Result<()> {
        while self.pc < code.len() && self.execution_steps < self.max_steps {
            self.execution_steps += 1;
            match self.mode {
                BFMode::Normal => self
                    .execute_normal(code)
                    .context("Failed to execute normal mode")?,
                BFMode::Syscall => self
                    .execute_syscall(code)
                    .context("Failed to execute syscall mode")?,
            }
            self.pc += 1;
        }
        Ok(())
    }

    fn execute_normal(&mut self, code: &str) -> VMResult<()> {
        match code.chars().nth(self.pc).unwrap() {
            '>' => self.increment_ptr()?,
            '<' => self.decrement_ptr()?,
            '+' => self.increment_value()?,
            '-' => self.decrement_value()?,
            '.' => self.output_char()?,
            ',' => self.input_char()?,
            _ => {}
        }
        Ok(())
    }

    fn execute_syscall(&mut self, code: &str) -> VMResult<()> {
        match code.chars().nth(self.pc).unwrap() {
            '.' => {
                // get syscall result
                // get syscall number
                // get syscall args
                // execute syscall

                let syscall_number = self.memory[1];
                let syscall_number = SyscallNum::try_from(syscall_number)?;

                match syscall_number {
                    SyscallNum::Read => {
                        // read
                        let fd = self.memory[2];
                        let buf = self.memory[3..];
                        let result = self.memory[1] = todo!();
                    }
                    SyscallNum::Write => {
                        // write
                        let fd = self.memory[2];
                        let buf = self.memory[3..];
                        let result = todo!();
                        self.memory[1] = result.unwrap_or(0) as u32;
                    }
                    SyscallNum::Socket => {
                        // socket
                        let domain = self.memory[2];
                        let ty = self.memory[3];
                        let protocol = self.memory[4];
                        let result = todo!();
                        self.memory[1] = result.unwrap_or(0) as u32;
                    }
                    SyscallNum::Bind => {
                        // bind
                        let fd = self.memory[2];
                        let addr = self.memory[3..];
                        let result = self.memory[1] = result.unwrap_or(0) as u32;
                    }
                    SyscallNum::Listen => {
                        // listen
                        let fd = self.memory[2];
                        let backlog = self.memory[3];
                        let result = todo!();
                        self.memory[1] = result.unwrap_or(0) as u32;
                    }
                    SyscallNum::Accept => {
                        // accept
                        let fd = self.memory[2];
                        let addr = self.memory[3..];
                        let result = self.fds[fd as usize].accept();
                        self.memory[1] = result.unwrap_or(0) as u32;
                    }
                    SyscallNum::Close => {
                        // close
                        let fd = self.memory[2];
                        let result = self.fds[fd as usize].close();
                        self.memory[1] = result.unwrap_or(0) as u32;
                    }
                    _ => return Err(VMError::InvalidSyscall),
                }
            }
            _ => self.execute_normal(code)?,
        }
        Ok(())
    }

    // Memory safety methods
    fn check_bounds(&self, ptr: u32) -> VMResult<()> {
        if ptr >= self.memory.len() as u32 {
            Err(VMError::MemoryOutOfBounds)
        } else {
            Ok(())
        }
    }

    fn increment_ptr(&mut self) -> VMResult<()> {
        self.ptr = self.ptr.checked_add(1).ok_or(VMError::MemoryOutOfBounds)?;
        self.check_bounds(self.ptr)
    }

    fn decrement_ptr(&mut self) -> VMResult<()> {
        self.ptr = self.ptr.checked_sub(1).ok_or(VMError::MemoryOutOfBounds)?;
        self.check_bounds(self.ptr)
    }

    fn increment_value(&mut self) -> VMResult<()> {
        self.check_bounds(self.ptr)?;
        self.memory[self.ptr as usize] = self.memory[self.ptr as usize].wrapping_add(1);
        Ok(())
    }

    fn decrement_value(&mut self) -> VMResult<()> {
        self.check_bounds(self.ptr)?;
        self.memory[self.ptr as usize] = self.memory[self.ptr as usize].wrapping_sub(1);
        Ok(())
    }

    fn output_char(&mut self) -> VMResult<()> {
        self.check_bounds(self.ptr)?;
        let value = self.memory[self.ptr as usize];
        if let Some(c) = char::from_u32(value) {
            self.output.push(c);
        }
        Ok(())
    }

    fn input_char(&mut self) -> VMResult<()> {
        self.check_bounds(self.ptr)?;
        let mut buf = [0u8; 1];
        std::io::stdin()
            .read_exact(&mut buf)
            .map_err(VMError::IoError)?;
        self.memory[self.ptr as usize] = buf[0] as u32;
        Ok(())
    }
}
