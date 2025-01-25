use crate::errors::{CompileError, CompileResult, Result};
use crate::{Syscall, SyscallNum};
use anyhow::Context;

#[derive(Debug, Clone)]
pub enum BFLNode {
    Assign(String, Box<BFLNode>), // var = expr
    Variable(String),             // var
    String(String),               // string
    Number(i32),                  // literal number
    Bytes(Vec<u8>),               // raw bytes
    Add(Box<BFLNode>, Box<BFLNode>),
    Sub(Box<BFLNode>, Box<BFLNode>),
    If(Box<BFLNode>, Vec<BFLNode>),    // condition, body
    While(Box<BFLNode>, Vec<BFLNode>), // condition, body
    Syscall(Syscall),
    Block(Vec<BFLNode>),
}

pub struct MemoryContainer {
    pos: u32,
    size: u32,
}

pub struct BFLCompiler {
    current_ptr_pos: u32,
    output: String,
}

impl BFLCompiler {
    pub fn new() -> Self {
        BFLCompiler {
            current_ptr_pos: 0,
            output: String::new(),
        }
    }

    pub fn compile(&mut self, node: &BFLNode) -> Result<String> {
        match node {
            BFLNode::Assign(var, expr) => self
                .compile_assign(var, expr)
                .context("Failed to compile assignment"),
            BFLNode::Variable(var) => self
                .compile_variable(var)
                .context("Failed to compile variable"),
            BFLNode::String(str) => self.compile_string(str).context("Failed to compile string"),
            BFLNode::Number(num) => self
                .compile_number(*num)
                .context("Failed to compile number"),
            BFLNode::Bytes(bytes) => self.compile_bytes(bytes).context("Failed to compile bytes"),
            BFLNode::Add(left, right) => self
                .compile_add(left, right)
                .context("Failed to compile addition"),
            BFLNode::Sub(left, right) => self
                .compile_sub(left, right)
                .context("Failed to compile subtraction"),
            BFLNode::If(cond, body) => self
                .compile_if(cond, body)
                .context("Failed to compile if statement"),
            BFLNode::While(cond, body) => self
                .compile_while(cond, body)
                .context("Failed to compile while loop"),
            BFLNode::Syscall(syscall) => self
                .compile_syscall(*syscall)
                .context("Failed to compile syscall"),
            BFLNode::Block(body) => self.compile_block(body).context("Failed to compile block"),
        }
    }

    fn allocate_memory(&mut self, size: u32) -> CompileResult<MemoryContainer> {
        let container = MemoryContainer {
            pos: self.current_ptr_pos,
            size,
        };
        self.current_ptr_pos = self
            .current_ptr_pos
            .checked_add(size)
            .ok_or(CompileError::AllocationFailed)?;
        Ok(container)
    }

    fn compile_assign(&mut self, var: &str, expr: &BFLNode) -> CompileResult<String> {
        // Implementation
        todo!()
    }

    fn compile_variable(&mut self, var: &str) -> CompileResult<String> {
        // Implementation
        todo!()
    }

    fn compile_string(&mut self, s: &str) -> CompileResult<String> {
        // Implementation
        todo!()
    }

    fn compile_number(&mut self, num: i32) -> CompileResult<String> {
        // Implementation
        todo!()
    }

    fn compile_bytes(&mut self, bytes: &[u8]) -> CompileResult<String> {
        // Implementation
        todo!()
    }

    fn compile_add(&mut self, left: &BFLNode, right: &BFLNode) -> CompileResult<String> {
        // Implementation
        todo!()
    }

    fn compile_sub(&mut self, left: &BFLNode, right: &BFLNode) -> CompileResult<String> {
        // Implementation
        todo!()
    }

    fn compile_if(&mut self, cond: &BFLNode, body: &[BFLNode]) -> CompileResult<String> {
        // Implementation
        todo!()
    }

    fn compile_while(&mut self, cond: &BFLNode, body: &[BFLNode]) -> CompileResult<String> {
        // Implementation
        todo!()
    }

    fn compile_syscall(&mut self, syscall: Syscall) -> CompileResult<String> {
        // Implementation
        match syscall {
            Syscall::Read { fd, buf } => {
                // emit read syscall

                // emit syscall result
            }
            Syscall::Write { fd, buf } => todo!(),
            Syscall::Socket {
                domain,
                service,
                protocol,
            } => todo!(),
            Syscall::Bind { fd, addr } => todo!(),
            Syscall::Listen { fd, backlog } => todo!(),
            Syscall::Accept { fd } => todo!(),
            Syscall::Close { fd } => todo!(),
        }
    }

    fn compile_block(&mut self, body: &[BFLNode]) -> CompileResult<String> {
        // Implementation
        todo!()
    }
}
