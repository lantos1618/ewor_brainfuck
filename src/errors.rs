use std::io;
use thiserror::Error;

/// VM-specific errors that we want to handle specifically
#[derive(Debug, Error)]
pub enum VMError {
    #[error("Memory access out of bounds")]
    MemoryOutOfBounds,
    #[error("Invalid syscall number")]
    InvalidSyscall,
    #[error("Execution timeout")]
    ExecutionTimeout,
    #[error("IO error: {0}")]
    IoError(#[from] io::Error),
}

/// Compiler-specific errors that we want to handle specifically
#[derive(Debug, Error)]
pub enum CompileError {
    #[error("Variable not found: {0}")]
    VariableNotFound(String),
    #[error("Memory allocation failed")]
    AllocationFailed,
    #[error("Invalid expression")]
    InvalidExpression,
    #[error("Type mismatch")]
    TypeMismatch,
    #[error("Other error: {0}")]
    Other(#[from] anyhow::Error),
}

// For VM operations where we want specific error handling
pub type VMResult<T> = std::result::Result<T, VMError>;

// For compiler operations where we want specific error handling
pub type CompileResult<T> = std::result::Result<T, CompileError>;

// For general operations where we want to propagate errors with anyhow
pub type Result<T> = anyhow::Result<T>;

impl From<VMError> for CompileError {
    fn from(err: VMError) -> Self {
        CompileError::Other(err.into())
    }
}
