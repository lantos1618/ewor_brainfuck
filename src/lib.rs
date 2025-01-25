// Re-export modules
pub mod bfl;
pub mod bfvm;
pub mod errors;

pub use errors::{Result, VMError, VMResult};

/// Represents system calls available in the BF virtual machine
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyscallNum {
    Read = 0,
    Write = 1,
    Socket = 2,
    Bind = 3,
    Listen = 4,
    Accept = 5,
    Close = 6,
}

impl TryFrom<u32> for SyscallNum {
    type Error = VMError;

    fn try_from(value: u32) -> std::result::Result<SyscallNum, errors::VMError> {
        Ok(match value {
            0 => SyscallNum::Read,
            1 => SyscallNum::Write,
            2 => SyscallNum::Socket,
            3 => SyscallNum::Bind,
            4 => SyscallNum::Listen,
            5 => SyscallNum::Accept,
            6 => SyscallNum::Close,
            _ => return Err(VMError::InvalidSyscall.into()),
        })
    }
}

impl From<SyscallNum> for u32 {
    fn from(value: SyscallNum) -> Self {
        value as u32
    }
}

#[derive(Debug, Clone)]
pub enum Syscall {
    Read {
        fd: u32,
        buf: Vec<u8>,
    },
    Write {
        fd: u32,
        buf: Vec<u8>,
    },
    Socket {
        domain: u32,
        service: u32,
        protocol: u32,
    },
    Bind {
        fd: u32,
        addr: Vec<u8>,
    },
    Listen {
        fd: u32,
        backlog: u32,
    },
    Accept {
        fd: u32,
    },
    Close {
        fd: u32,
    },
}
