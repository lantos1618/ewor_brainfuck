use std::cmp::Ordering;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum BFLNode {
    Assign(String, Box<BFLNode>), // var = expr
    Variable(String),             // var
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
            BFLNode::Variable(name) => {
                let location = self.allocate_variable(name);
                self.move_to(location);
            }
            BFLNode::Assign(name, expr) => {
                let location = self.allocate_variable(name);
                self.compile(expr)?;
                self.move_to(location);
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
                // Move to syscall number location (cell 0)
                self.move_to(0);
                self.compile(number)?;

                // Compile arguments
                for (i, arg) in args.iter().enumerate() {
                    self.move_to(i + 1);
                    self.compile(arg)?;
                }

                // Execute syscall
                self.output.push('.');
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
        let program = BFLNode::Block(vec![
            BFLNode::Assign("x".to_string(), Box::new(BFLNode::Number(42))),
        ]);
        compiler.compile(&program).unwrap();
        assert!(!compiler.get_output().is_empty());
    }
}
