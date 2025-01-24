use std::cmp::Ordering;
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
            println!("Using existing variable {} at location {}", name, location);
            location
        } else {
            let location = self.next_var_location;
            println!("Allocating new variable {} at location {}", name, location);
            self.variables.insert(name.to_string(), location);
            self.next_var_location = location + 1;
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
                    let ascii_val = c as u8;
                    self.output.push_str(&"+".repeat(ascii_val as usize)); // Set to ASCII value
                    self.output.push('>'); // Move to next cell
                }
                // Remove the last '>' since we don't want to move past the last character
                if !s.is_empty() {
                    self.output.pop();
                }
            }
            BFLNode::Bytes(bytes) => {
                // Store bytes data at current position
                for &byte in bytes {
                    self.output.push_str("[-]"); // Clear current cell
                    self.output.push_str(&"+".repeat(byte as usize)); // Set to byte value
                    self.output.push('>'); // Move to next cell
                }
                // Remove the last '>' since we don't want to move past the last byte
                if !bytes.is_empty() {
                    self.output.pop();
                }
            }
            BFLNode::Variable(name) => {
                let location = self.allocate_variable(name);
                self.move_to(location);
                // For now, just move to the variable location
                // Dereferencing will be handled by the parent node if needed
            }
            BFLNode::Assign(name, expr) => {
                let location = self.allocate_variable(name);

                match expr.as_ref() {
                    BFLNode::String(s) => {
                        // Store string data right after the variable's location
                        let str_start = location + 1;

                        // Store the pointer in the variable
                        self.move_to(location);
                        self.output.push_str("[-]"); // Clear the variable
                        self.output.push_str(&"+".repeat(str_start)); // Store pointer to string data

                        // Store the string data
                        self.move_to(str_start);
                        for c in s.chars() {
                            self.output.push_str("[-]"); // Clear current cell
                            let ascii_val = c as u8;
                            // Print debug info
                            println!(
                                "Storing character '{}' with ASCII value {} at cell {}",
                                c, ascii_val, str_start
                            );
                            self.output.push_str(&"+".repeat(ascii_val as usize)); // Set to ASCII value
                            self.output.push('>'); // Move to next cell
                        }
                        // Remove the last '>' since we don't want to move past the last character
                        if !s.is_empty() {
                            self.output.pop();
                        }

                        // Update next_var_location to after the string data
                        self.next_var_location = str_start + s.len() + 1;
                    }
                    BFLNode::Bytes(bytes) => {
                        // Store bytes data right after the variable's location
                        let bytes_start = location + 1;

                        // Store the pointer in the variable
                        self.move_to(location);
                        self.output.push_str("[-]"); // Clear the variable
                        self.output.push_str(&"+".repeat(bytes_start)); // Store pointer to bytes data

                        // Store the bytes data
                        self.move_to(bytes_start);
                        for &byte in bytes {
                            self.output.push_str("[-]"); // Clear current cell
                            println!("Storing byte value {} at cell {}", byte, bytes_start);
                            self.output.push_str(&"+".repeat(byte as usize)); // Set to byte value
                            self.output.push('>'); // Move to next cell
                        }
                        // Remove the last '>' since we don't want to move past the last byte
                        if !bytes.is_empty() {
                            self.output.pop();
                        }

                        // Update next_var_location to after the bytes data
                        self.next_var_location = bytes_start + bytes.len() + 1;
                    }
                    BFLNode::Block(statements) => {
                        // For blocks, treat each statement as a sequential byte
                        let data_start = location + 1;

                        // Store the pointer in the variable
                        self.move_to(location);
                        self.output.push_str("[-]"); // Clear the variable
                        self.output.push_str(&"+".repeat(data_start)); // Store pointer to data

                        // Store each byte
                        for (offset, stmt) in statements.iter().enumerate() {
                            self.move_to(data_start + offset);
                            self.output.push_str("[-]"); // Clear cell
                            match stmt {
                                BFLNode::Number(n) => {
                                    // For numbers, just set the value directly
                                    if *n > 0 {
                                        self.output.push_str(&"+".repeat(*n as usize));
                                    }
                                }
                                _ => {
                                    self.compile(stmt)?;
                                }
                            }
                        }

                        // Update next_var_location to after the data
                        self.next_var_location = data_start + statements.len() + 1;
                    }
                    _ => {
                        self.move_to(location);
                        self.output.push_str("[-]"); // Clear the target location
                        self.compile(expr)?;
                    }
                }
            }
            BFLNode::Add(left, right) => {
                match (left.as_ref(), right.as_ref()) {
                    (BFLNode::Variable(name), BFLNode::Number(offset)) => {
                        // Special case: adding offset to variable - get pointer and add offset
                        let location = self.allocate_variable(name);
                        self.move_to(location);
                        // Copy the pointer value
                        self.output.push_str("[->+>+<<]>>[-<<+>>]<<");
                        // Add the offset
                        self.output.push_str(&"+".repeat(*offset as usize));
                        // Move to the pointed location
                        self.output.push_str("[>+<-]>");
                    }
                    _ => {
                        // Normal addition
                        self.compile(left)?;
                        self.output.push_str("[->+>+<<]>>[-<<+>>]<<");
                        self.compile(right)?;
                    }
                }
            }
            BFLNode::Sub(left, right) => {
                match (left.as_ref(), right.as_ref()) {
                    (BFLNode::Variable(name), _) => {
                        // When subtracting from a variable, dereference it first
                        let location = self.allocate_variable(name);
                        self.move_to(location);
                        // Copy and dereference the pointer
                        self.output.push_str("[->+>+<<]>>[-<<+>>]<<[>+<-]>");
                        // Now subtract the right operand
                        self.compile(right)?;
                        self.output.push_str("[-<->]<");
                    }
                    _ => {
                        // Normal subtraction
                        self.compile(left)?;
                        self.output.push_str("[->+>+<<]>>[-<<+>>]<<");
                        self.compile(right)?;
                        self.output.push_str("[-<->]<");
                    }
                }
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
                // Store the current position to return to it after the loop
                let start_pos = self.current_position();

                // Compile condition
                self.compile(condition)?;

                // Start loop
                self.output.push('[');

                // Compile loop body
                for stmt in body {
                    self.compile(stmt)?;
                }

                // Return to condition cell
                self.move_to(start_pos);

                // End loop
                self.output.push(']');
            }
            BFLNode::Syscall(number, args) => {
                // Store current position
                let start_pos = self.current_position();

                // First, clear ONLY the syscall area (cells 1-7), preserving cell 0 for fd
                for i in 1..8 {
                    self.move_to(i);
                    self.output.push_str("[-]");
                }

                // Set syscall number in cell 7
                self.move_to(7);
                match number.as_ref() {
                    BFLNode::Number(n) => {
                        // For numbers, just set the value directly
                        if *n > 0 {
                            self.output.push_str(&"+".repeat(*n as usize));
                        }
                    }
                    _ => {
                        self.compile(number)?;
                    }
                }

                // Set up arguments in their cells
                let mut current_pos = 1;
                for arg in args.iter() {
                    match arg {
                        BFLNode::Number(0) => {
                            // Special case: if we're using fd from cell 0, don't copy it
                            // Just increment current_pos since we'll use cell 0 directly
                            current_pos += 1;
                        }
                        BFLNode::Block(statements) => {
                            // For blocks, treat each statement as a sequential byte
                            for stmt in statements {
                                self.move_to(current_pos);
                                self.output.push_str("[-]"); // Clear cell
                                match stmt {
                                    BFLNode::Number(n) => {
                                        // For numbers, just set the value directly
                                        if *n > 0 {
                                            self.output.push_str(&"+".repeat(*n as usize));
                                        }
                                    }
                                    _ => {
                                        self.compile(stmt)?;
                                    }
                                }
                                current_pos += 1;
                            }
                        }
                        BFLNode::Variable(name) => {
                            let var_loc = *self
                                .variables
                                .get(name)
                                .ok_or_else(|| format!("Variable {} not found", name))?;

                            // Move to argument position and clear it
                            self.move_to(current_pos);
                            self.output.push_str("[-]");

                            // Move to variable location and copy its value to the argument position
                            self.move_to(var_loc);
                            // Copy the value (which might be a pointer) to the argument position
                            self.output.push_str("[->+>+<<]>>[-<<+>>]<<"); // Copy to temp cells
                            self.output.push_str("[>+<-]>"); // Move to pointed location if this is a pointer
                                                             // Now copy the actual value to the argument position
                            self.output.push_str("[->+>+<<]>>[-<<+>>]<<"); // Copy to temp cells
                            self.output.push_str("[-<+>]<"); // Move back to original position

                            current_pos += 1;
                        }
                        BFLNode::Number(n) => {
                            // For numbers, just set the value directly
                            self.move_to(current_pos);
                            self.output.push_str("[-]"); // Clear cell
                            if *n > 0 {
                                self.output.push_str(&"+".repeat(*n as usize));
                            }
                            current_pos += 1;
                        }
                        _ => {
                            self.move_to(current_pos);
                            self.output.push_str("[-]"); // Clear destination
                            self.compile(arg)?;
                            current_pos += 1;
                        }
                    }
                }

                // Move to syscall number and execute
                self.move_to(7);
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

    pub fn get_variable_location(&self, name: &str) -> Option<usize> {
        self.variables.get(name).copied()
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
    fn test_hello_world() {
        let mut compiler = BFLCompiler::new();
        println!("\n=== Starting Hello World Test ===\n");

        let program = BFLNode::Block(vec![
            // Store "Hello, World!\n" in a variable
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
        println!("Generated BF code:\n{}", bf_code);

        let mut bf = BF::new(bf_code, Mode::BFA);
        println!("\n--- Memory before execution: ---");
        let cells = bf.dump_cells(30);
        for (i, cell) in cells.iter().enumerate() {
            if *cell != 0 {
                println!(
                    "Cell {}: {} ({})",
                    i,
                    cell,
                    char::from_u32(*cell).unwrap_or(' ')
                );
            }
        }

        println!("\n--- Program Output: ---");
        bf.run().unwrap();

        println!("\n--- Memory after execution: ---");
        let cells = bf.dump_cells(30);
        for (i, cell) in cells.iter().enumerate() {
            if *cell != 0 {
                println!(
                    "Cell {}: {} ({})",
                    i,
                    cell,
                    char::from_u32(*cell).unwrap_or(' ')
                );
            }
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
        assert_eq!(cells[9], u32::from(b'A')); // String data is stored correctly
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
            println!(
                "Cell {}: {} ({})",
                i,
                cell,
                char::from_u32(*cell).unwrap_or(' ')
            );
        }

        bf.run().unwrap();

        // Print final memory state
        println!("\nMemory after execution:");
        let cells = bf.dump_cells(20);
        for (i, cell) in cells.iter().enumerate() {
            println!(
                "Cell {}: {} ({})",
                i,
                cell,
                char::from_u32(*cell).unwrap_or(' ')
            );
        }
    }

    #[test]
    fn test_simple_hello() {
        let mut compiler = BFLCompiler::new();
        println!("\n=== Starting Simple Hello Test ===\n");

        let program = BFLNode::Block(vec![
            // Store string "Hi\n" in a variable
            BFLNode::Assign(
                "msg".to_string(),
                Box::new(BFLNode::String("Hi\n".to_string())),
            ),
            // Write syscall
            BFLNode::Syscall(
                Box::new(BFLNode::Number(1)), // write syscall
                vec![
                    BFLNode::Number(1),                   // stdout
                    BFLNode::Variable("msg".to_string()), // buffer pointer
                    BFLNode::Number(3),                   // length
                ],
            ),
        ]);

        compiler.compile(&program).unwrap();
        let bf_code = compiler.get_output();
        println!("Generated BF code:\n{}", bf_code);

        let mut bf = BF::new(bf_code, Mode::BFA);
        println!("\n--- Memory before execution: ---");
        let cells = bf.dump_cells(15);
        for (i, cell) in cells.iter().enumerate() {
            if *cell != 0 {
                println!(
                    "Cell {}: {} ({})",
                    i,
                    cell,
                    char::from_u32(*cell).unwrap_or(' ')
                );
            }
        }

        println!("\n--- Program Output: ---");
        bf.run().unwrap();

        println!("\n--- Memory after execution: ---");
        let cells = bf.dump_cells(15);
        for (i, cell) in cells.iter().enumerate() {
            if *cell != 0 {
                println!(
                    "Cell {}: {} ({})",
                    i,
                    cell,
                    char::from_u32(*cell).unwrap_or(' ')
                );
            }
        }
        println!("\n=== End of Simple Hello Test ===\n");
    }

    #[test]
    fn test_string_storage_basic() {
        let mut compiler = BFLCompiler::new();
        println!("\n=== Starting String Storage Basic Test ===\n");

        // Just store a string and verify memory layout
        let program = BFLNode::Block(vec![BFLNode::Assign(
            "str".to_string(),
            Box::new(BFLNode::String("A".to_string())),
        )]);

        compiler.compile(&program).unwrap();
        let bf_code = compiler.get_output();
        println!("Generated BF code:\n{}", bf_code);

        let mut bf = BF::new(bf_code, Mode::BFA);

        // Run the code to set up memory
        bf.run().unwrap();

        println!("\n--- Memory after string storage: ---");
        let cells = bf.dump_cells(15);
        for (i, cell) in cells.iter().enumerate() {
            if *cell != 0 {
                println!(
                    "Cell {}: {} ({})",
                    i,
                    cell,
                    char::from_u32(*cell).unwrap_or(' ')
                );
            }
        }
        // Cell 8 (variable) should contain 9 (pointer to string)
        // Cell 9 should contain 65 ('A')
        assert_eq!(cells[8], 9, "Variable should point to string location");
        assert_eq!(cells[9], 65, "String content should be 'A'");
    }

    #[test]
    fn test_string_write_single_char() {
        let mut compiler = BFLCompiler::new();
        println!("\n=== Starting Single Char Write Test ===\n");

        let program = BFLNode::Block(vec![
            // Store just "A" in a variable
            BFLNode::Assign("ch".to_string(), Box::new(BFLNode::String("A".to_string()))),
            // Write syscall
            BFLNode::Syscall(
                Box::new(BFLNode::Number(1)), // write syscall
                vec![
                    BFLNode::Number(1),                  // stdout
                    BFLNode::Variable("ch".to_string()), // buffer pointer
                    BFLNode::Number(1),                  // length
                ],
            ),
        ]);

        compiler.compile(&program).unwrap();
        let bf_code = compiler.get_output();
        println!("Generated BF code:\n{}", bf_code);

        let mut bf = BF::new(bf_code, Mode::BFA);
        println!("\n--- Memory before execution: ---");
        let cells = bf.dump_cells(15);
        for (i, cell) in cells.iter().enumerate() {
            if *cell != 0 {
                println!(
                    "Cell {}: {} ({})",
                    i,
                    cell,
                    char::from_u32(*cell).unwrap_or(' ')
                );
            }
        }

        println!("\n--- Program Output: ---");
        bf.run().unwrap();

        println!("\n--- Memory after execution: ---");
        let cells = bf.dump_cells(15);
        for (i, cell) in cells.iter().enumerate() {
            if *cell != 0 {
                println!(
                    "Cell {}: {} ({})",
                    i,
                    cell,
                    char::from_u32(*cell).unwrap_or(' ')
                );
            }
        }
    }

    #[test]
    fn test_string_write_multiple_chars() {
        let mut compiler = BFLCompiler::new();
        println!("\n=== Starting Multiple Chars Write Test ===\n");

        let program = BFLNode::Block(vec![
            // Store "ABC" in a variable
            BFLNode::Assign(
                "str".to_string(),
                Box::new(BFLNode::String("ABC".to_string())),
            ),
            // Write syscall
            BFLNode::Syscall(
                Box::new(BFLNode::Number(1)), // write syscall
                vec![
                    BFLNode::Number(1),                   // stdout
                    BFLNode::Variable("str".to_string()), // buffer pointer
                    BFLNode::Number(3),                   // length
                ],
            ),
        ]);

        compiler.compile(&program).unwrap();
        let bf_code = compiler.get_output();
        println!("Generated BF code:\n{}", bf_code);

        let mut bf = BF::new(bf_code, Mode::BFA);
        println!("\n--- Memory before execution: ---");
        let cells = bf.dump_cells(15);
        for (i, cell) in cells.iter().enumerate() {
            if *cell != 0 {
                println!(
                    "Cell {}: {} ({})",
                    i,
                    cell,
                    char::from_u32(*cell).unwrap_or(' ')
                );
            }
        }

        println!("\n--- Program Output: ---");
        bf.run().unwrap();

        println!("\n--- Memory after execution: ---");
        let cells = bf.dump_cells(15);
        for (i, cell) in cells.iter().enumerate() {
            if *cell != 0 {
                println!(
                    "Cell {}: {} ({})",
                    i,
                    cell,
                    char::from_u32(*cell).unwrap_or(' ')
                );
            }
        }
    }

    #[test]
    fn test_single_char_write() {
        let mut compiler = BFLCompiler::new();
        println!("\n=== Starting Single Char Write Test ===\n");

        let program = BFLNode::Block(vec![
            // Store just "A" in a variable
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
        println!("Generated BF code:\n{}", bf_code);

        let mut bf = BF::new(bf_code, Mode::BFA);
        println!("\n--- Memory before execution: ---");
        let cells = bf.dump_cells(15);
        for (i, cell) in cells.iter().enumerate() {
            println!(
                "Cell {}: {} ({})",
                i,
                cell,
                char::from_u32(*cell).unwrap_or(' ')
            );
        }

        println!("\n--- Program Output: ---");
        bf.run().unwrap();

        println!("\n--- Memory after execution: ---");
        let cells = bf.dump_cells(15);
        for (i, cell) in cells.iter().enumerate() {
            println!(
                "Cell {}: {} ({})",
                i,
                cell,
                char::from_u32(*cell).unwrap_or(' ')
            );
        }
    }

    #[test]
    fn test_syscall_bytes() {
        let mut compiler = BFLCompiler::new();
        println!("\n=== Starting Syscall Bytes Test ===\n");

        // Create a simple syscall that uses a byte array argument
        let program = BFLNode::Block(vec![
            // First store bytes in a variable
            BFLNode::Assign(
                "sockaddr".to_string(),
                Box::new(BFLNode::Bytes(vec![2, 0, 0x1F, 0x90])),
            ),
            // Then use the variable in a syscall
            BFLNode::Syscall(
                Box::new(BFLNode::Number(1)), // write syscall
                vec![
                    BFLNode::Number(1), // stdout
                    BFLNode::Variable("sockaddr".to_string()),
                    BFLNode::Number(4), // length
                ],
            ),
        ]);

        compiler.compile(&program).unwrap();
        let bf_code = compiler.get_output();
        println!("Generated BF code:\n{}", bf_code);

        let mut bf = BF::new(bf_code, Mode::BFA);
        println!("\n--- Memory before execution: ---");
        let cells = bf.dump_cells(15);
        for (i, cell) in cells.iter().enumerate() {
            println!("Cell {}: {} (0x{:02x})", i, cell, cell);
        }

        // Run the program
        bf.run().unwrap();

        println!("\n--- Memory after execution: ---");
        let cells = bf.dump_cells(15);
        for (i, cell) in cells.iter().enumerate() {
            println!("Cell {}: {} (0x{:02x})", i, cell, cell);
        }

        // Verify the bytes were set correctly in the variable
        assert_eq!(cells[9], 2, "First byte should be 2");
        assert_eq!(cells[10], 0, "Second byte should be 0");
        assert_eq!(cells[11], 0x1F, "Third byte should be 0x1F");
        assert_eq!(cells[12], 0x90, "Fourth byte should be 0x90");
    }

    #[test]
    fn test_syscall_block() {
        let mut compiler = BFLCompiler::new();
        println!("\n=== Starting Syscall Block Test ===\n");

        // Create a syscall that uses a block argument to set sequential bytes
        let program = BFLNode::Block(vec![
            // First store bytes in a variable
            BFLNode::Assign(
                "sockaddr".to_string(),
                Box::new(BFLNode::Block(vec![
                    BFLNode::Number(2),    // First byte
                    BFLNode::Number(0),    // Second byte
                    BFLNode::Number(0x1F), // Third byte
                    BFLNode::Number(0x90), // Fourth byte
                ])),
            ),
            // Then use the variable in a syscall
            BFLNode::Syscall(
                Box::new(BFLNode::Number(1)), // write syscall
                vec![
                    BFLNode::Number(1), // stdout
                    BFLNode::Variable("sockaddr".to_string()),
                    BFLNode::Number(4), // length
                ],
            ),
        ]);

        compiler.compile(&program).unwrap();
        let bf_code = compiler.get_output();
        println!("Generated BF code:\n{}", bf_code);

        let mut bf = BF::new(bf_code, Mode::BFA);
        println!("\n--- Memory before execution: ---");
        let cells = bf.dump_cells(15);
        for (i, cell) in cells.iter().enumerate() {
            println!("Cell {}: {} (0x{:02x})", i, cell, cell);
        }

        // Run the program
        bf.run().unwrap();

        println!("\n--- Memory after execution: ---");
        let cells = bf.dump_cells(15);
        for (i, cell) in cells.iter().enumerate() {
            println!("Cell {}: {} (0x{:02x})", i, cell, cell);
        }

        // Verify the bytes were set correctly in the variable
        assert_eq!(cells[9], 2, "First byte should be 2");
        assert_eq!(cells[10], 0, "Second byte should be 0");
        assert_eq!(cells[11], 0x1F, "Third byte should be 0x1F");
        assert_eq!(cells[12], 0x90, "Fourth byte should be 0x90");
    }
}
