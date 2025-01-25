use crate::{
    errors::{Result, VMError, VMResult},
    SyscallNum,
};
use anyhow::Context;
use nix::{
    sys::socket::{self, AddressFamily, SockFlag, SockType, SockaddrIn},
    unistd,
};
use std::{
    io::Read,
    net::Ipv4Addr,
    os::fd::{AsFd, OwnedFd},
    os::unix::io::{AsRawFd, FromRawFd},
};

// Memory layout:
// | syscall_result (4 bytes) | syscall_number (4 bytes) | syscall_args | heap... |
const SYSCALL_RESULT_OFFSET: usize = 0;
const SYSCALL_NUMBER_OFFSET: usize = 4;
const SYSCALL_ARGS_OFFSET: usize = 8;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BFMode {
    Normal,
    Syscall,
}

pub struct BFVM {
    mode: BFMode,
    memory: Vec<u8>,
    ptr: u32,
    output: String,
    pc: usize,
    execution_steps: u64,
    max_steps: u64,
    fds: Vec<OwnedFd>,
}

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
                let syscall_number = self.get_syscall_number();
                let syscall_number = SyscallNum::try_from(syscall_number)?;

                match syscall_number {
                    SyscallNum::Read => {
                        let fd = self.get_syscall_arg(0) as usize;
                        let buf_size = self.get_syscall_arg(1) as usize;

                        if fd >= self.fds.len() {
                            self.set_syscall_result(u32::MAX);
                            return Ok(());
                        }

                        let mut buf = vec![0u8; buf_size];
                        match unistd::read(self.fds[fd].as_raw_fd(), &mut buf) {
                            Ok(n) => {
                                // Copy read bytes directly to memory after syscall args
                                let data_offset = SYSCALL_ARGS_OFFSET + 8; // After fd and buf_size
                                for (i, &byte) in buf[..n].iter().enumerate() {
                                    if i + data_offset >= self.memory.len() {
                                        break;
                                    }
                                    self.memory[i + data_offset] = byte;
                                }
                                self.set_syscall_result(n as u32);
                            }
                            Err(_) => {
                                self.set_syscall_result(u32::MAX);
                            }
                        }
                    }
                    SyscallNum::Write => {
                        let fd = self.get_syscall_arg(0) as usize;
                        let buf_size = self.get_syscall_arg(1) as usize;

                        if fd >= self.fds.len() {
                            self.set_syscall_result(u32::MAX);
                            return Ok(());
                        }

                        // Use memory bytes directly
                        let buf = &self.memory[SYSCALL_ARGS_OFFSET..SYSCALL_ARGS_OFFSET + buf_size];

                        match unistd::write(self.fds[fd].as_fd(), buf) {
                            Ok(n) => {
                                self.set_syscall_result(n as u32);
                            }
                            Err(_) => {
                                self.set_syscall_result(u32::MAX);
                            }
                        }
                    }
                    SyscallNum::Socket => {
                        let domain = self.get_syscall_arg(0) as i32;
                        let sock_type = self.get_syscall_arg(1) as i32;
                        let protocol = self.get_syscall_arg(2) as i32;

                        let domain = match domain {
                            2 => AddressFamily::Inet,
                            _ => AddressFamily::Unix,
                        };

                        let sock_type = match sock_type {
                            1 => SockType::Stream,
                            2 => SockType::Datagram,
                            _ => SockType::Raw,
                        };

                        match socket::socket(domain, sock_type, SockFlag::empty(), None) {
                            Ok(fd) => {
                                self.fds.push(fd);
                                self.set_syscall_result((self.fds.len() - 1) as u32);
                            }
                            Err(_) => {
                                self.set_syscall_result(u32::MAX);
                            }
                        }
                    }
                    SyscallNum::Bind => {
                        let fd = self.get_syscall_arg(0) as usize;
                        let port = self.get_syscall_arg(1) as u16;

                        if fd >= self.fds.len() {
                            self.set_syscall_result(u32::MAX);
                            return Ok(());
                        }

                        let addr = SockaddrIn::from(std::net::SocketAddrV4::new(
                            Ipv4Addr::UNSPECIFIED,
                            port,
                        ));

                        match socket::bind(self.fds[fd].as_raw_fd(), &addr) {
                            Ok(()) => {
                                self.set_syscall_result(0);
                            }
                            Err(_) => {
                                self.set_syscall_result(u32::MAX);
                            }
                        }
                    }
                    SyscallNum::Listen => {
                        let fd = self.get_syscall_arg(0) as usize;
                        let backlog = socket::Backlog::new(self.get_syscall_arg(1) as i32)
                            .unwrap_or(socket::Backlog::MAXCONN);

                        if fd >= self.fds.len() {
                            self.set_syscall_result(u32::MAX);
                            return Ok(());
                        }

                        match socket::listen(&self.fds[fd], backlog) {
                            Ok(()) => {
                                self.set_syscall_result(0);
                            }
                            Err(_) => {
                                self.set_syscall_result(u32::MAX);
                            }
                        }
                    }
                    SyscallNum::Accept => {
                        let fd = self.get_syscall_arg(0) as usize;

                        if fd >= self.fds.len() {
                            self.set_syscall_result(u32::MAX);
                            return Ok(());
                        }

                        match socket::accept(self.fds[fd].as_raw_fd()) {
                            Ok(new_fd) => {
                                self.fds.push(unsafe { OwnedFd::from_raw_fd(new_fd) });
                                self.set_syscall_result((self.fds.len() - 1) as u32);
                            }
                            Err(_) => {
                                self.set_syscall_result(u32::MAX);
                            }
                        }
                    }
                    SyscallNum::Close => {
                        let fd = self.get_syscall_arg(0) as usize;

                        if fd >= self.fds.len() {
                            self.set_syscall_result(u32::MAX);
                            return Ok(());
                        }

                        // Remove and drop the FD, which automatically closes it
                        self.fds.remove(fd);
                        self.set_syscall_result(0);
                    }
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
        if let Some(c) = char::from_u32(value as u32) {
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
        self.memory[self.ptr as usize] = buf[0];
        Ok(())
    }

    // Helper methods to access syscall values from memory
    fn get_syscall_result(&self) -> u32 {
        u32::from_ne_bytes(
            self.memory[SYSCALL_RESULT_OFFSET..SYSCALL_RESULT_OFFSET + 4]
                .try_into()
                .unwrap(),
        )
    }

    fn set_syscall_result(&mut self, value: u32) {
        self.memory[SYSCALL_RESULT_OFFSET..SYSCALL_RESULT_OFFSET + 4]
            .copy_from_slice(&value.to_ne_bytes());
    }

    fn get_syscall_number(&self) -> u32 {
        u32::from_ne_bytes(
            self.memory[SYSCALL_NUMBER_OFFSET..SYSCALL_NUMBER_OFFSET + 4]
                .try_into()
                .unwrap(),
        )
    }

    fn get_syscall_arg(&self, arg_index: usize) -> u32 {
        let offset = SYSCALL_ARGS_OFFSET + (arg_index * 4);
        u32::from_ne_bytes(self.memory[offset..offset + 4].try_into().unwrap())
    }
}
