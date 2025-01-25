use crate::errors::{CompileError, CompileResult, Result};
use crate::{Syscall, SyscallNum};
use anyhow::Context;
use std::collections::HashMap;

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

#[derive(Debug)]
struct Variable {
    pos: u32,
    size: u32,
    is_array: bool,
}

pub struct BFLCompiler {
    current_ptr_pos: u32,
    output: String,
    variables: HashMap<String, Variable>,
    temp_counter: u32,
}

impl BFLCompiler {
    pub fn new() -> Self {
        BFLCompiler {
            current_ptr_pos: 0,
            output: String::new(),
            variables: HashMap::new(),
            temp_counter: 0,
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
                .compile_syscall(&syscall)
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
        let mut output = String::new();

        // Compile the expression
        output.push_str(&self.compile(expr)?);

        // Store the result in the variable's memory location
        let var_pos = self.get_variable_position(var)?;
        output.push_str(&self.move_to(var_pos));

        Ok(output)
    }

    fn compile_variable(&mut self, var: &str) -> CompileResult<String> {
        let var_pos = self.get_variable_position(var)?;
        Ok(self.move_to(var_pos))
    }

    fn compile_string(&mut self, s: &str) -> CompileResult<String> {
        let mut output = String::new();
        let start_pos = self.current_ptr_pos;

        // Allocate memory for the string
        let container = self.allocate_memory(s.len() as u32 + 1)?; // +1 for null terminator

        // Write string characters to memory
        for (i, byte) in s.bytes().enumerate() {
            output.push_str(&self.move_to(container.pos + i as u32));
            output.push_str(&format!("{:+}", byte as i32));
        }

        // Add null terminator
        output.push_str(&self.move_to(container.pos + s.len() as u32));
        output.push_str("[-]"); // Clear cell

        // Move back to start
        output.push_str(&self.move_to(start_pos));

        Ok(output)
    }

    fn compile_number(&mut self, num: i32) -> CompileResult<String> {
        let mut output = String::new();

        // Clear current cell
        output.push_str("[-]");

        // Add or subtract to reach the target number
        if num >= 0 {
            output.push_str(&"+".repeat(num as usize));
        } else {
            output.push_str(&"-".repeat(-num as usize));
        }

        Ok(output)
    }

    fn compile_bytes(&mut self, bytes: &[u8]) -> CompileResult<String> {
        let mut output = String::new();
        let start_pos = self.current_ptr_pos;

        // Allocate memory for the bytes
        let container = self.allocate_memory(bytes.len() as u32)?;

        // Write bytes to memory
        for (i, &byte) in bytes.iter().enumerate() {
            output.push_str(&self.move_to(container.pos + i as u32));
            output.push_str(&format!("{:+}", byte as i32));
        }

        // Move back to start
        output.push_str(&self.move_to(start_pos));

        Ok(output)
    }

    fn compile_add(&mut self, left: &BFLNode, right: &BFLNode) -> CompileResult<String> {
        let mut output = String::new();

        // Compile left operand
        output.push_str(&self.compile(left)?);

        // Save left result
        let temp = self.allocate_memory(1)?;
        output.push_str(&self.move_to(temp.pos));
        output.push_str("[->+<]"); // Move value to next cell

        // Compile right operand
        output.push_str(&self.compile(right)?);

        // Add left value to right value
        output.push_str(&self.move_to(temp.pos));
        output.push_str("[>+<-]"); // Add saved value to result

        Ok(output)
    }

    fn compile_sub(&mut self, left: &BFLNode, right: &BFLNode) -> CompileResult<String> {
        let mut output = String::new();

        // Compile left operand
        output.push_str(&self.compile(left)?);

        // Save left result
        let temp = self.allocate_memory(1)?;
        output.push_str(&self.move_to(temp.pos));
        output.push_str("[->+<]"); // Move value to next cell

        // Compile right operand
        output.push_str(&self.compile(right)?);

        // Subtract right value from left value
        output.push_str(&self.move_to(temp.pos));
        output.push_str("[>-<-]"); // Subtract saved value from result

        Ok(output)
    }

    fn compile_if(&mut self, cond: &BFLNode, body: &[BFLNode]) -> CompileResult<String> {
        let mut output = String::new();

        // Compile condition
        output.push_str(&self.compile(cond)?);

        // Start if block
        output.push_str("[");

        // Compile body
        for node in body {
            output.push_str(&self.compile(node)?);
        }

        // End if block
        output.push_str("]");

        Ok(output)
    }

    fn compile_while(&mut self, cond: &BFLNode, body: &[BFLNode]) -> CompileResult<String> {
        let mut output = String::new();

        // Compile condition
        output.push_str(&self.compile(cond)?);

        // Start while loop
        output.push_str("[");

        // Compile body
        for node in body {
            output.push_str(&self.compile(node)?);
        }

        // Recompile condition
        output.push_str(&self.compile(cond)?);

        // End while loop
        output.push_str("]");

        Ok(output)
    }

    fn compile_syscall(&mut self, syscall: &Syscall) -> CompileResult<String> {
        let mut output = String::new();

        // Move to syscall result position and clear it
        output.push_str(&self.move_to(0));
        output.push_str(&self.clear_cell());

        match syscall {
            Syscall::Read { fd, buf } => {
                // Set syscall number (0 for read)
                output.push_str(&self.move_to(1));
                output.push_str(&self.set_cell(0));

                // Set fd
                output.push_str(&self.move_to(2));
                output.push_str(&self.set_signed_number(*fd as i32));

                // Set buffer size
                output.push_str(&self.move_to(3));
                output.push_str(&self.set_signed_number(buf.len() as i32));

                // Copy buffer to memory starting at position 4
                for (i, &byte) in buf.iter().enumerate() {
                    output.push_str(&self.move_to(4 + i as u32));
                    output.push_str(&self.set_signed_number(byte as i32));
                }
            }
            Syscall::Write { fd, buf } => {
                // Set syscall number (1 for write)
                output.push_str(&self.move_to(1));
                output.push_str(&self.set_cell(1));

                // Set fd
                output.push_str(&self.move_to(2));
                output.push_str(&self.set_signed_number(*fd as i32));

                // Set buffer size
                output.push_str(&self.move_to(3));
                output.push_str(&self.set_signed_number(buf.len() as i32));

                // Copy buffer to memory starting at position 4
                for (i, &byte) in buf.iter().enumerate() {
                    output.push_str(&self.move_to(4 + i as u32));
                    output.push_str(&self.set_signed_number(byte as i32));
                }
            }
            Syscall::Socket {
                domain,
                service,
                protocol,
            } => {
                // Set syscall number (2 for socket)
                output.push_str(&self.move_to(1));
                output.push_str(&self.set_cell(2));

                // Set domain
                output.push_str(&self.move_to(2));
                output.push_str(&self.set_signed_number(*domain as i32));

                // Set service type
                output.push_str(&self.move_to(3));
                output.push_str(&self.set_signed_number(*service as i32));

                // Set protocol
                output.push_str(&self.move_to(4));
                output.push_str(&self.set_signed_number(*protocol as i32));
            }
            Syscall::Bind { fd, addr } => {
                // Set syscall number (3 for bind)
                output.push_str(&self.move_to(1));
                output.push_str(&self.set_cell(3));

                // Set fd
                output.push_str(&self.move_to(2));
                output.push_str(&self.set_signed_number(*fd as i32));

                // Set address size
                output.push_str(&self.move_to(3));
                output.push_str(&self.set_signed_number(addr.len() as i32));

                // Copy address to memory starting at position 4
                for (i, &byte) in addr.iter().enumerate() {
                    output.push_str(&self.move_to(4 + i as u32));
                    output.push_str(&self.set_signed_number(byte as i32));
                }
            }
            Syscall::Listen { fd, backlog } => {
                // Set syscall number (4 for listen)
                output.push_str(&self.move_to(1));
                output.push_str(&self.set_cell(4));

                // Set fd
                output.push_str(&self.move_to(2));
                output.push_str(&self.set_signed_number(*fd as i32));

                // Set backlog
                output.push_str(&self.move_to(3));
                output.push_str(&self.set_signed_number(*backlog as i32));
            }
            Syscall::Accept { fd } => {
                // Set syscall number (5 for accept)
                output.push_str(&self.move_to(1));
                output.push_str(&self.set_cell(5));

                // Set fd
                output.push_str(&self.move_to(2));
                output.push_str(&self.set_signed_number(*fd as i32));
            }
            Syscall::Close { fd } => {
                // Set syscall number (6 for close)
                output.push_str(&self.move_to(1));
                output.push_str(&self.set_cell(6));

                // Set fd
                output.push_str(&self.move_to(2));
                output.push_str(&self.set_signed_number(*fd as i32));
            }
        }

        // Execute syscall
        output.push_str(".");

        Ok(output)
    }

    fn move_to(&self, target: u32) -> String {
        if target == self.current_ptr_pos {
            return String::new();
        }

        let diff = target as i32 - self.current_ptr_pos as i32;
        if diff > 0 {
            ">".repeat(diff as usize)
        } else {
            "<".repeat(-diff as usize)
        }
    }

    fn compile_block(&mut self, body: &[BFLNode]) -> CompileResult<String> {
        let mut output = String::new();

        for node in body {
            output.push_str(&self.compile(node)?);
        }

        Ok(output)
    }

    fn get_variable_position(&mut self, var: &str) -> CompileResult<u32> {
        if let Some(variable) = self.variables.get(var) {
            Ok(variable.pos)
        } else {
            // Allocate new variable
            let pos = self.current_ptr_pos;
            self.current_ptr_pos = self
                .current_ptr_pos
                .checked_add(1)
                .ok_or(CompileError::AllocationFailed)?;

            self.variables.insert(
                var.to_string(),
                Variable {
                    pos,
                    size: 1,
                    is_array: false,
                },
            );

            Ok(pos)
        }
    }

    fn clear_cell(&self) -> String {
        "[-]".to_string() // Loops until cell is zero
    }

    fn set_cell(&self, value: i32) -> String {
        let mut output = String::new();
        output.push_str(&self.clear_cell()); // First clear the cell

        // Then add the desired value
        if value >= 0 {
            output.push_str(&"+".repeat(value as usize));
        } else {
            output.push_str(&"-".repeat(-value as usize));
        }
        output
    }

    // Format a number with explicit sign and set cell to that value
    fn set_signed_number(&self, value: i32) -> String {
        format!("{:+}", value) // Always shows + or - sign
    }

    // Copy value from current cell to target cell
    fn copy_value(&self, target_offset: i32) -> String {
        // [->+>+<<]  pattern copies current cell to two cells right
        format!(
            "[{}+{}+{}]",
            ">".repeat(target_offset.abs() as usize), // Move right
            "<".repeat(target_offset.abs() as usize), // Move back
            "<".repeat(target_offset.abs() as usize)  // Move back again
        )
    }

    // Move value from current cell to target cell
    fn move_value(&self, target_offset: i32) -> String {
        // [->+<] pattern moves current cell one cell right
        format!(
            "[{}+{}]",
            ">".repeat(target_offset.abs() as usize), // Move right
            "<".repeat(target_offset.abs() as usize)  // Move back
        )
    }

    // Multiply current cell by a constant
    fn multiply_by(&self, factor: i32) -> String {
        let mut output = String::new();
        // Save original value
        output.push_str("["); // While current cell is not zero
                              // Add factor copies
        for _ in 0..factor.abs() {
            output.push_str(">+"); // Add 1 to next cell
        }
        output.push_str("<-]"); // Move back and decrement
        output
    }

    // Get a temporary memory location for intermediate calculations
    fn get_temp(&mut self) -> CompileResult<u32> {
        let pos = self.current_ptr_pos;
        self.temp_counter += 1;
        self.current_ptr_pos = self
            .current_ptr_pos
            .checked_add(1)
            .ok_or(CompileError::AllocationFailed)?;
        Ok(pos)
    }

    // Release a temporary memory location
    fn release_temp(&mut self) {
        if self.temp_counter > 0 {
            self.temp_counter -= 1;
            self.current_ptr_pos -= 1;
        }
    }

    // Declare an array variable
    fn declare_array(&mut self, name: &str, size: u32) -> CompileResult<()> {
        let pos = self.current_ptr_pos;
        self.current_ptr_pos = self
            .current_ptr_pos
            .checked_add(size)
            .ok_or(CompileError::AllocationFailed)?;

        self.variables.insert(
            name.to_string(),
            Variable {
                pos,
                size,
                is_array: true,
            },
        );

        Ok(())
    }

    // Array operations
    fn array_index(&mut self, array: &str, index: &BFLNode) -> CompileResult<String> {
        let mut output = String::new();

        // Get array info first before any mutable operations
        let array_info = self
            .variables
            .get(array)
            .ok_or_else(|| CompileError::VariableNotFound(array.to_string()))?;
        let base_pos = array_info.pos;
        if !array_info.is_array {
            return Err(CompileError::TypeMismatch);
        }

        // Compile index expression
        output.push_str(&self.compile(index)?);

        // Add base offset
        output.push_str(&self.move_to(base_pos));

        // Now we're at array[index]
        Ok(output)
    }

    // Arithmetic operations
    fn divide(&mut self, numerator: &BFLNode, denominator: &BFLNode) -> CompileResult<String> {
        let mut output = String::new();

        // Compile numerator
        output.push_str(&self.compile(numerator)?);

        // Save numerator to temp
        let temp1 = self.get_temp()?;
        output.push_str(&self.move_to(temp1));
        output.push_str(&self.copy_value(1));

        // Compile denominator
        output.push_str(&self.compile(denominator)?);

        // Save denominator to temp
        let temp2 = self.get_temp()?;
        output.push_str(&self.move_to(temp2));
        output.push_str(&self.copy_value(1));

        // Division algorithm:
        // while (numerator >= denominator) {
        //   numerator -= denominator;
        //   result++;
        // }
        let result = self.get_temp()?;
        output.push_str(&self.move_to(result));
        output.push_str(&self.clear_cell());

        // Move to numerator and start division loop
        output.push_str(&self.move_to(temp1));
        output.push_str("["); // While numerator > 0

        // Copy denominator to working area
        output.push_str(&self.move_to(temp2));
        output.push_str(&self.copy_value(1));

        // Subtract denominator from numerator
        output.push_str(&self.move_to(temp1));
        output.push_str("-");

        // If we could subtract, increment result
        output.push_str(&self.move_to(result));
        output.push_str("+");

        // Move back to numerator to continue loop
        output.push_str(&self.move_to(temp1));
        output.push_str("]");

        // Cleanup
        self.release_temp(); // temp2
        self.release_temp(); // temp1

        Ok(output)
    }

    // Comparison operations
    fn equals(&mut self, left: &BFLNode, right: &BFLNode) -> CompileResult<String> {
        let mut output = String::new();

        // Compile left operand
        output.push_str(&self.compile(left)?);

        // Save to temp
        let temp1 = self.get_temp()?;
        output.push_str(&self.move_to(temp1));
        output.push_str(&self.copy_value(1));

        // Compile right operand
        output.push_str(&self.compile(right)?);

        // Subtract and check if zero
        output.push_str(&self.move_to(temp1));
        output.push_str("[->-<]"); // Subtract
        output.push_str(">"); // Move to result
        output.push_str("[[-]<+>]"); // If non-zero, result is 0
        output.push_str("<[[-]>+<]"); // If zero, result is 1

        self.release_temp();

        Ok(output)
    }

    // Multiply two numbers
    fn multiply(&mut self, left: &BFLNode, right: &BFLNode) -> CompileResult<String> {
        let mut output = String::new();

        // Compile left operand
        output.push_str(&self.compile(left)?);

        // Save left value
        let temp1 = self.get_temp()?;
        output.push_str(&self.move_to(temp1));
        output.push_str(&self.copy_value(1));

        // Compile right operand
        output.push_str(&self.compile(right)?);

        // Save right value
        let temp2 = self.get_temp()?;
        output.push_str(&self.move_to(temp2));
        output.push_str(&self.copy_value(1));

        // Clear result cell
        let result = self.get_temp()?;
        output.push_str(&self.move_to(result));
        output.push_str(&self.clear_cell());

        // For each value in left operand
        output.push_str(&self.move_to(temp1));
        output.push_str("[");

        // Add right operand to result
        output.push_str(&self.move_to(temp2));
        output.push_str(&self.copy_value(1));
        output.push_str(&self.move_to(result));
        output.push_str(&self.move_value(1));

        // Decrement left operand and continue loop
        output.push_str(&self.move_to(temp1));
        output.push_str("-");
        output.push_str("]");

        // Cleanup
        self.release_temp(); // temp2
        self.release_temp(); // temp1

        Ok(output)
    }

    // Modulo operation (remainder after division)
    fn modulo(&mut self, left: &BFLNode, right: &BFLNode) -> CompileResult<String> {
        let mut output = String::new();

        // Compile left operand (dividend)
        output.push_str(&self.compile(left)?);

        // Save dividend
        let temp1 = self.get_temp()?;
        output.push_str(&self.move_to(temp1));
        output.push_str(&self.copy_value(1));

        // Compile right operand (divisor)
        output.push_str(&self.compile(right)?);

        // Save divisor
        let temp2 = self.get_temp()?;
        output.push_str(&self.move_to(temp2));
        output.push_str(&self.copy_value(1));

        // While dividend >= divisor
        output.push_str(&self.move_to(temp1));
        output.push_str("[");

        // Try to subtract divisor
        output.push_str(&self.move_to(temp2));
        output.push_str(&self.copy_value(1));
        output.push_str(&self.move_to(temp1));
        output.push_str("-");

        // Continue loop if we could subtract
        output.push_str("]");

        // Cleanup
        self.release_temp(); // temp2
        self.release_temp(); // temp1

        Ok(output)
    }

    // Greater than comparison
    fn greater_than(&mut self, left: &BFLNode, right: &BFLNode) -> CompileResult<String> {
        let mut output = String::new();

        // Compile left operand
        output.push_str(&self.compile(left)?);

        // Save left value
        let temp1 = self.get_temp()?;
        output.push_str(&self.move_to(temp1));
        output.push_str(&self.copy_value(1));

        // Compile right operand
        output.push_str(&self.compile(right)?);

        // Subtract right from left, if something remains then left > right
        output.push_str(&self.move_to(temp1));
        output.push_str("[->-<]"); // Subtract
        output.push_str(">"); // Move to result
        output.push_str("[[-]<+>]"); // If something remains, result is 1

        self.release_temp();

        Ok(output)
    }

    // Utility function to print a number as ASCII digits
    fn print_number(&mut self, num: &BFLNode) -> CompileResult<String> {
        let mut output = String::new();

        // Compile number
        output.push_str(&self.compile(num)?);

        // Convert to ASCII digits and print
        let temp = self.get_temp()?;
        output.push_str(&self.move_to(temp));

        // Add ASCII '0' (48) to each digit
        output.push_str(&self.multiply_by(48));
        output.push_str("."); // Print

        self.release_temp();

        Ok(output)
    }

    // Utility function to read a number from input
    fn read_number(&mut self) -> CompileResult<String> {
        let mut output = String::new();

        // Get temp cell for input
        let temp = self.get_temp()?;
        output.push_str(&self.move_to(temp));

        // Read character
        output.push_str(",");

        // Subtract ASCII '0' (48) to get actual number
        output.push_str(&self.set_cell(-48));

        self.release_temp();

        Ok(output)
    }
}
