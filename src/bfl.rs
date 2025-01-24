use std::cmp::Ordering;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum BFLNode {
    Assign(String, Box<BFLNode>), // var = expr
    Variable(String),             // var
    String(String),               // string
    Number(i32),                  // literal number
    Add(Box<BFLNode>, Box<BFLNode>),
    Sub(Box<BFLNode>, Box<BFLNode>),
    If(Box<BFLNode>, Vec<BFLNode>),      // condition, body
    While(Box<BFLNode>, Vec<BFLNode>),   // condition, body
    Syscall(Box<BFLNode>, Vec<BFLNode>), // syscall_number, args
    Block(Vec<BFLNode>),
}

pub struct BFLCompiler {
    variables: HashMap<String, usize>, // Maps variable names to memory locations
    next_var_location: usize,          // Next available memory location
    output: String,                    // Generated brainfuck code
}

impl Default for BFLCompiler {
    fn default() -> Self {
        Self::new()
    }
}

impl BFLCompiler {
    pub fn new() -> Self {
        BFLCompiler {
            variables: HashMap::new(),
            next_var_location: 8, // Start after syscall args area
            output: String::new(),
        }
    }

    // Allocate a new variable location
    fn allocate_variable(&mut self, name: &str) -> usize {
        if let Some(&location) = self.variables.get(name) {
            location
        } else {
            let location = self.next_var_location;
            self.variables.insert(name.to_string(), location);
            self.next_var_location += 1;
            location
        }
    }

    // Move pointer to location
    fn move_to(&mut self, location: usize) {
        let current_pos = self.current_position();
        let diff = location as i32 - current_pos as i32;

        match diff.cmp(&0) {
            Ordering::Greater => self.output.push_str(&">".repeat(diff as usize)),
            Ordering::Less => self.output.push_str(&"<".repeat((-diff) as usize)),
            Ordering::Equal => {}
        }
    }

    // Get current position (by counting > and < in output)
    fn current_position(&self) -> usize {
        let mut pos = 0i32;
        for c in self.output.chars() {
            match c {
                '>' => pos += 1,
                '<' => pos -= 1,
                _ => {}
            }
        }
        pos as usize
    }

    // Compile a node to brainfuck code
    pub fn compile(&mut self, node: &BFLNode) -> Result<(), String> {
        match node {
            BFLNode::Number(n) => {
                // Clear current cell and add number
                self.output.push_str("[-]");
                self.output.push_str(&"+".repeat(*n as usize));
            }
            BFLNode::String(s) => {
                // Store string data at current position
                for c in s.chars() {
                    self.output.push_str("[-]"); // Clear current cell
                    self.output.push_str(&"+".repeat(c as usize)); // Set to ASCII value
                    self.output.push('>'); // Move to next cell
                }
                self.output.push_str("[-]"); // Null terminator
                self.output.push('>'); // Move past null terminator
            }
            BFLNode::Variable(name) => {
                let location = self.allocate_variable(name);
                self.move_to(location);
            }
            BFLNode::Assign(name, expr) => {
                let location = self.allocate_variable(name);

                match expr.as_ref() {
                    BFLNode::String(s) => {
                        // First, store the string data at next available location
                        let str_pos = self.next_var_location;
                        self.move_to(str_pos);
                        self.compile(expr)?;

                        // Now store the pointer in the variable
                        self.move_to(location);
                        self.output.push_str("[-]"); // Clear the variable
                        self.output.push_str(&"+".repeat(str_pos)); // Store pointer to string data

                        // Update next_var_location to after the string data
                        self.next_var_location = str_pos + s.len() + 2; // +1 for null terminator, +1 for next position
                    }
                    _ => {
                        self.move_to(location);
                        self.output.push_str("[-]"); // Clear the target location
                        self.compile(expr)?;
                    }
                }
            }
            BFLNode::Add(left, right) => {
                // Compile left expression
                self.compile(left)?;
                // Store result in temporary location
                self.output.push_str("[->+>+<<]>>[-<<+>>]<<");
                // Compile right expression and add it
                self.compile(right)?;
            }
            BFLNode::Sub(left, right) => {
                // Similar to Add but with subtraction
                self.compile(left)?;
                self.output.push_str("[->+>+<<]>>[-<<+>>]<<");
                self.compile(right)?;
                self.output.push_str("[-]");
            }
            BFLNode::If(condition, body) => {
                self.compile(condition)?;
                self.output.push('[');
                for stmt in body {
                    self.compile(stmt)?;
                }
                self.output.push(']');
            }
            BFLNode::While(condition, body) => {
                self.compile(condition)?;
                self.output.push('[');
                for stmt in body {
                    self.compile(stmt)?;
                }
                self.compile(condition)?;
                self.output.push(']');
            }
            BFLNode::Syscall(number, args) => {
                // Store current position
                let start_pos = self.current_position();

                // First, clear the syscall area (cells 0-7)
                for i in 0..8 {
                    self.move_to(i);
                    self.output.push_str("[-]");
                }

                // Set syscall number first (cell 0)
                self.move_to(0);
                self.compile(number)?;

                // Set up arguments in their cells
                for (i, arg) in args.iter().enumerate() {
                    let arg_pos = i + 1;
                    match arg {
                        BFLNode::Variable(name) => {
                            let var_loc = *self
                                .variables
                                .get(name)
                                .ok_or_else(|| format!("Variable {} not found", name))?;

                            // Move to argument position
                            self.move_to(arg_pos);
                            self.output.push_str("[-]"); // Clear destination

                            // Move to variable location and copy its value
                            self.move_to(var_loc);
                            self.output.push_str("[->+>+<<]"); // Copy to temp and dest
                            self.output.push_str(">>[-<<+>>]<<"); // Move temp back to source
                        }
                        _ => {
                            self.move_to(arg_pos);
                            self.output.push_str("[-]"); // Clear destination
                            self.compile(arg)?;
                        }
                    }
                }

                // Move to syscall number and execute
                self.move_to(0);
                self.output.push('.');

                // Return to original position
                self.move_to(start_pos);
            }
            BFLNode::Block(statements) => {
                for stmt in statements {
                    self.compile(stmt)?;
                }
            }
        }
        Ok(())
    }

    pub fn get_output(&self) -> &str {
        &self.output
    }
}

// Example usage:
#[cfg(test)]
mod tests {
    use super::*;
    use crate::bf::{Mode, BF};

    #[test]
    fn test_simple_assignment() {
        let mut compiler = BFLCompiler::new();
        let program = BFLNode::Assign("x".to_string(), Box::new(BFLNode::Number(42)));
        compiler.compile(&program).unwrap();
        assert!(!compiler.get_output().is_empty());
    }

    #[test]
    fn test_syscall() {
        let mut compiler = BFLCompiler::new();
        // write(1, "A", 1)
        let program = BFLNode::Syscall(
            Box::new(BFLNode::Number(1)), // write syscall
            vec![
                BFLNode::Number(1),  // fd (stdout)
                BFLNode::Number(65), // "A"
                BFLNode::Number(1),  // length
            ],
        );
        compiler.compile(&program).unwrap();
        assert!(!compiler.get_output().is_empty());
    }

    // make a bfl program that reads from stdin and writes to stdout
    #[test]
    fn test_read_write() {
        let mut compiler = BFLCompiler::new();
        let program = BFLNode::Block(vec![BFLNode::Assign(
            "x".to_string(),
            Box::new(BFLNode::Number(42)),
        )]);
        compiler.compile(&program).unwrap();
        assert!(!compiler.get_output().is_empty());
    }

    #[test]
    fn test_hello_world_syscall() {
        let mut compiler = BFLCompiler::new();

        let program = BFLNode::Block(vec![
            // Store the string in a variable
            BFLNode::Assign(
                "msg".to_string(),
                Box::new(BFLNode::String("Hello, World!\n".to_string())),
            ),
            // Write syscall
            BFLNode::Syscall(
                Box::new(BFLNode::Number(1)), // write syscall
                vec![
                    BFLNode::Number(1),                   // stdout
                    BFLNode::Variable("msg".to_string()), // buffer pointer
                    BFLNode::Number(14),                  // length
                ],
            ),
        ]);

        compiler.compile(&program).unwrap();
        let bf_code = compiler.get_output();
        println!("\nGenerated BF code:\n{}", bf_code);

        let mut bf = BF::new(bf_code, Mode::BFA);
        println!("\nMemory before execution:");
        let cells = bf.dump_cells(30);
        for (i, cell) in cells.iter().enumerate() {
            println!("Cell {}: {} ({})", i, cell, *cell as char);
        }

        bf.run().unwrap();

        println!("\nMemory after execution:");
        let cells = bf.dump_cells(30);
        for (i, cell) in cells.iter().enumerate() {
            println!("Cell {}: {} ({})", i, cell, *cell as char);
        }
    }

    #[test]
    #[should_panic(expected = "Permission denied: socket operations not allowed in test mode")]
    fn test_socket_syscall() {
        let mut compiler = BFLCompiler::new();
        let program = BFLNode::Syscall(
            Box::new(BFLNode::Number(2)), // socket syscall
            vec![
                BFLNode::Number(2), // AF_INET
                BFLNode::Number(1), // SOCK_STREAM
                BFLNode::Number(0), // protocol 0
            ],
        );
        compiler.compile(&program).unwrap();
        let bf_code = compiler.get_output();

        let mut bf = BF::new(bf_code, Mode::BFA);
        // Expected to fail in test mode
        bf.run().unwrap();
    }

    #[test]
    fn test_string_storage() {
        let mut compiler = BFLCompiler::new();
        let program = BFLNode::Assign(
            "str".to_string(),
            Box::new(BFLNode::String("A".to_string())),
        );
        compiler.compile(&program).unwrap();
        let bf_code = compiler.get_output();

        // Run the code and check memory
        let mut bf = BF::new(bf_code, Mode::BFA);
        bf.run().unwrap();
        let cells = bf.dump_cells(20);

        // The variable 'str' should be at position 8 (HEAP_START)
        // and should point to position 9 where the actual string data is
        assert_eq!(cells[8], 9); // Variable points to string data
        assert_eq!(cells[9], b'A'); // String data is stored correctly
    }

    #[test]
    fn test_write_syscall_with_string() {
        let mut compiler = BFLCompiler::new();
        let program = BFLNode::Block(vec![
            // First store string in variable
            BFLNode::Assign(
                "msg".to_string(),
                Box::new(BFLNode::String("A".to_string())),
            ),
            // Then do write syscall
            BFLNode::Syscall(
                Box::new(BFLNode::Number(1)), // write syscall
                vec![
                    BFLNode::Number(1),                   // stdout
                    BFLNode::Variable("msg".to_string()), // buffer pointer
                    BFLNode::Number(1),                   // length
                ],
            ),
        ]);

        compiler.compile(&program).unwrap();
        let bf_code = compiler.get_output();
        println!("Generated code for write syscall test:\n{}", bf_code);

        let mut bf = BF::new(bf_code, Mode::BFA);
        println!("Memory before execution:");
        println!("{:?}", bf.dump_cells(20));
        bf.run().unwrap();
    }

    #[test]
    fn test_variable_allocation() {
        let mut compiler = BFLCompiler::new();
        let program = BFLNode::Block(vec![
            BFLNode::Assign("x".to_string(), Box::new(BFLNode::Number(1))),
            BFLNode::Assign("y".to_string(), Box::new(BFLNode::Number(2))),
        ]);
        compiler.compile(&program).unwrap();

        // Check that variables are allocated after HEAP_START
        assert!(compiler.variables["x"] >= 8);
        assert!(compiler.variables["y"] >= 8);
        assert_ne!(compiler.variables["x"], compiler.variables["y"]);
    }

    #[test]
    fn test_string_storage_and_syscall() {
        let mut compiler = BFLCompiler::new();
        let program = BFLNode::Block(vec![
            // Store string in variable
            BFLNode::Assign(
                "msg".to_string(),
                Box::new(BFLNode::String("A".to_string())),
            ),
            // Write syscall
            BFLNode::Syscall(
                Box::new(BFLNode::Number(1)), // write syscall
                vec![
                    BFLNode::Number(1),                   // stdout
                    BFLNode::Variable("msg".to_string()), // buffer pointer
                    BFLNode::Number(1),                   // length
                ],
            ),
        ]);

        compiler.compile(&program).unwrap();
        let bf_code = compiler.get_output();
        println!("\nGenerated BF code:\n{}", bf_code);

        let mut bf = BF::new(bf_code, Mode::BFA);

        // Print initial memory state
        println!("\nMemory before execution:");
        let cells = bf.dump_cells(20);
        for (i, cell) in cells.iter().enumerate() {
            println!("Cell {}: {} ({})", i, cell, *cell as char);
        }

        bf.run().unwrap();

        // Print final memory state
        println!("\nMemory after execution:");
        let cells = bf.dump_cells(20);
        for (i, cell) in cells.iter().enumerate() {
            println!("Cell {}: {} ({})", i, cell, *cell as char);
        }
    }
}
