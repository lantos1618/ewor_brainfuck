use std::cmp::Ordering;
use std::collections::HashMap;

const SCRATCH_1: usize = 30000;
const SCRATCH_2: usize = 30001;
const SCRATCH_3: usize = 30002; // Added for robust clamped subtraction

#[derive(Debug, Clone)]
pub enum BFLNode {
    Assign(String, Box<BFLNode>),
    Variable(String),
    String(String),
    Number(i32),
    Bytes(Vec<u8>),
    Add(Box<BFLNode>, Box<BFLNode>),
    Sub(Box<BFLNode>, Box<BFLNode>),
    If(Box<BFLNode>, Vec<BFLNode>),
    While(Box<BFLNode>, Vec<BFLNode>),
    Syscall(Box<BFLNode>, Vec<BFLNode>),
    Block(Vec<BFLNode>),
}

pub struct BFLCompiler {
    variables: HashMap<String, usize>,
    next_var_location: usize,
    output: String,
    current_ptr: usize,
}

impl Default for BFLCompiler {
    fn default() -> Self {
        Self::new()
    }
}

impl BFLCompiler {
    pub fn new() -> Self {
        let mut variables = HashMap::new();
        variables.insert("_syscall_result".to_string(), 0);
        BFLCompiler {
            variables,
            next_var_location: 8, // Start after syscall reserved area
            output: String::new(),
            current_ptr: 0,
        }
    }

    fn move_to(&mut self, location: usize) {
        if self.current_ptr == location {
            return;
        }
        
        let distance = if location > self.current_ptr {
            location - self.current_ptr
        } else {
            self.current_ptr - location
        };
        
        // For small distances, use direct movement
        if distance <= 100 {
            if location > self.current_ptr {
                self.output.push_str(&">".repeat(distance));
            } else {
                self.output.push_str(&"<".repeat(distance));
            }
        } else {
            // For large distances, use a loop-based approach
            // This is much more efficient than brute force
            let temp_counter = 30000;
            
            // Set temp counter to distance
            self.output.push_str("[-]"); // Clear temp
            for _ in 0..distance {
                self.output.push('+');
            }
            
            // Move while decrementing counter
            self.output.push_str("[");
            if location > self.current_ptr {
                self.output.push('>');
            } else {
                self.output.push('<');
            }
            self.output.push_str("[-]"); // Clear temp
            self.output.push_str("]");
        }
        
        self.current_ptr = location;
    }

    fn copy_value(&mut self, src: usize, dest: usize) {
        if src == dest {
            return;
        }
        
        // Use a simple loop-based copy
        let temp1 = 30000;
        let temp2 = 30001;
        
        // Clear destination
        if dest > self.current_ptr {
            self.output.push_str(&">".repeat(dest - self.current_ptr));
        } else {
            self.output.push_str(&"<".repeat(self.current_ptr - dest));
        }
        self.output.push_str("[-]");
        self.current_ptr = dest;
        
        // Copy source to temp1 and temp2
        if src > self.current_ptr {
            self.output.push_str(&">".repeat(src - self.current_ptr));
        } else {
            self.output.push_str(&"<".repeat(self.current_ptr - src));
        }
        self.current_ptr = src;
        
        self.output.push_str("[");
        // Move to temp1
        if temp1 > self.current_ptr {
            self.output.push_str(&">".repeat(temp1 - self.current_ptr));
        } else {
            self.output.push_str(&"<".repeat(self.current_ptr - temp1));
        }
        self.output.push('+');
        self.current_ptr = temp1;
        
        // Move to temp2
        if temp2 > self.current_ptr {
            self.output.push_str(&">".repeat(temp2 - self.current_ptr));
        } else {
            self.output.push_str(&"<".repeat(self.current_ptr - temp2));
        }
        self.output.push('+');
        self.current_ptr = temp2;
        
        // Move back to src
        if src > self.current_ptr {
            self.output.push_str(&">".repeat(src - self.current_ptr));
        } else {
            self.output.push_str(&"<".repeat(self.current_ptr - src));
        }
        self.current_ptr = src;
        self.output.push_str("-]");
        
        // Copy temp1 back to src
        if temp1 > self.current_ptr {
            self.output.push_str(&">".repeat(temp1 - self.current_ptr));
        } else {
            self.output.push_str(&"<".repeat(self.current_ptr - temp1));
        }
        self.current_ptr = temp1;
        self.output.push_str("[");
        if src > self.current_ptr {
            self.output.push_str(&">".repeat(src - self.current_ptr));
        } else {
            self.output.push_str(&"<".repeat(self.current_ptr - src));
        }
        self.output.push('+');
        self.current_ptr = src;
        if temp1 > self.current_ptr {
            self.output.push_str(&">".repeat(temp1 - self.current_ptr));
        } else {
            self.output.push_str(&"<".repeat(self.current_ptr - temp1));
        }
        self.current_ptr = temp1;
        self.output.push_str("-]");
        
        // Copy temp2 to dest
        if temp2 > self.current_ptr {
            self.output.push_str(&">".repeat(temp2 - self.current_ptr));
        } else {
            self.output.push_str(&"<".repeat(self.current_ptr - temp2));
        }
        self.current_ptr = temp2;
        self.output.push_str("[");
        if dest > self.current_ptr {
            self.output.push_str(&">".repeat(dest - self.current_ptr));
        } else {
            self.output.push_str(&"<".repeat(self.current_ptr - dest));
        }
        self.output.push('+');
        self.current_ptr = dest;
        if temp2 > self.current_ptr {
            self.output.push_str(&">".repeat(temp2 - self.current_ptr));
        } else {
            self.output.push_str(&"<".repeat(self.current_ptr - temp2));
        }
        self.current_ptr = temp2;
        self.output.push_str("-]");
        
        // Move to destination
        if dest > self.current_ptr {
            self.output.push_str(&">".repeat(dest - self.current_ptr));
        } else {
            self.output.push_str(&"<".repeat(self.current_ptr - dest));
        }
        self.current_ptr = dest;
    }

    /// Evaluate an expression into a specific cell, ending with the pointer at that cell
    fn eval_to_cell(&mut self, expr: &BFLNode, cell: usize) -> Result<(), String> {
        self.move_to(cell);
        self.output.push_str("[-]");
        match expr {
            BFLNode::Number(n) => {
                if *n > 0 {
                    self.output.push_str(&"+".repeat(*n as usize));
                }
            }
            BFLNode::Variable(name) => {
                let src = *self.variables.get(name).ok_or(format!("Variable '{}' not found", name))?;
                self.copy_value(src, cell);
            }
            _ => {
                // For complex expressions, evaluate to SCRATCH_2 then copy
                let prev_ptr = self.current_ptr;
                self.move_to(SCRATCH_2);
                self.output.push_str("[-]");
                self.compile(expr)?;
                self.copy_value(self.current_ptr, cell);
                self.move_to(cell);
            }
        }
        Ok(())
    }

    pub fn compile(&mut self, node: &BFLNode) -> Result<(), String> {
        match node {
            BFLNode::Block(statements) => {
                for stmt in statements {
                    self.compile(stmt)?;
                }
            }
            BFLNode::Assign(name, expr) => {
                let location = *self.variables.entry(name.clone()).or_insert_with(|| {
                    let loc = self.next_var_location;
                    self.next_var_location += 1;
                    loc
                });

                match expr.as_ref() {
                    BFLNode::String(s) => {
                        // Store string content directly in the variable location
                        // instead of storing a pointer to the string
                        self.move_to(location);
                        self.output.push_str("[-]");
                        
                        // Store the first character in the variable cell
                        if !s.is_empty() {
                            self.output.push_str(&"+".repeat(s.as_bytes()[0] as usize));
                        }
                        
                        // For multi-character strings, we could store additional chars in adjacent cells
                        // but for now, just store the first character to keep it simple
                    }
                    BFLNode::Bytes(bytes) => {
                        let data_location = self.next_var_location;
                        self.next_var_location += bytes.len();

                        self.move_to(location);
                        self.output.push_str("[-]");
                        self.output.push_str(&"+".repeat(data_location));

                        for (i, byte) in bytes.iter().enumerate() {
                            self.move_to(data_location + i);
                            self.output.push_str("[-]");
                            self.output.push_str(&"+".repeat(*byte as usize));
                        }
                    }
                    BFLNode::Variable(src_name) => {
                        let src_location = self.variables.get(src_name).copied().ok_or(format!("Source variable '{}' not found for assignment", src_name))?;
                        self.copy_value(src_location, location);
                    }
                    BFLNode::Number(n) => {
                        // Direct assignment of number value
                        self.move_to(location);
                        self.output.push_str("[-]");
                        if *n > 0 {
                            self.output.push_str(&"+".repeat(*n as usize));
                        }
                    }
                    BFLNode::Add(lhs, rhs) => {
                        // Special case: variable + number (most common in loops)
                        if let (BFLNode::Variable(var_name), BFLNode::Number(n)) = (lhs.as_ref(), rhs.as_ref()) {
                            let var_location = self.variables.get(var_name).copied()
                                .ok_or(format!("Variable '{}' not found in addition", var_name))?;
                            
                            // Simple increment: just add the number to the variable
                            self.move_to(var_location);
                            for _ in 0..*n {
                                self.output.push('+');
                            }
                            
                            // Copy result to destination
                            self.copy_value(var_location, location);
                        } else if let (BFLNode::Number(n1), BFLNode::Number(n2)) = (lhs.as_ref(), rhs.as_ref()) {
                            // Special case: number + number
                            let result = n1 + n2;
                            self.move_to(location);
                            self.output.push_str("[-]");
                            if result > 0 {
                                self.output.push_str(&"+".repeat(result as usize));
                            }
                        } else {
                            // General case: use the complex logic
                            self.move_to(location);
                            self.output.push_str("[-]");
                            // lhs
                            self.eval_to_cell(lhs, SCRATCH_1)?;
                            // rhs
                            self.eval_to_cell(rhs, SCRATCH_2)?;
                            // Add SCRATCH_1 and SCRATCH_2 into location
                            self.move_to(SCRATCH_1);
                            self.output.push_str("[");
                            self.move_to(location);
                            self.output.push('+');
                            self.move_to(SCRATCH_1);
                            self.output.push('-');
                            self.output.push_str("]");
                            self.move_to(SCRATCH_2);
                            self.output.push_str("[");
                            self.move_to(location);
                            self.output.push('+');
                            self.move_to(SCRATCH_2);
                            self.output.push('-');
                            self.output.push_str("]");
                        }
                    }
                    BFLNode::Sub(lhs, rhs) => {
                        // Special case: variable - number (most common in loops)
                        if let (BFLNode::Variable(var_name), BFLNode::Number(n)) = (lhs.as_ref(), rhs.as_ref()) {
                            let var_location = self.variables.get(var_name).copied()
                                .ok_or(format!("Variable '{}' not found in subtraction", var_name))?;
                            
                            // Simple decrement: just subtract the number from the variable
                            self.move_to(var_location);
                            for _ in 0..*n {
                                self.output.push('-');
                            }
                            
                            // Copy result to destination
                            self.copy_value(var_location, location);
                        } else if let (BFLNode::Number(n1), BFLNode::Number(n2)) = (lhs.as_ref(), rhs.as_ref()) {
                            // Special case: number - number
                            let result = (n1 - n2).max(0); // Clamp at 0
                            self.move_to(location);
                            self.output.push_str("[-]");
                            if result > 0 {
                                self.output.push_str(&"+".repeat(result as usize));
                            }
                        } else {
                            // General case: use the complex logic
                            self.move_to(location);
                            self.output.push_str("[-]");
                            // lhs
                            self.eval_to_cell(lhs, SCRATCH_1)?;
                            // rhs
                            self.eval_to_cell(rhs, SCRATCH_2)?;
                            // Copy SCRATCH_1 to location
                            self.move_to(SCRATCH_1);
                            self.output.push_str("[");
                            self.move_to(location);
                            self.output.push('+');
                            self.move_to(SCRATCH_1);
                            self.output.push('-');
                            self.output.push_str("]");
                            // Clamped subtraction: subtract SCRATCH_2 from location, clamping at 0
                            self.move_to(SCRATCH_2);
                            self.output.push_str("["); // For each unit in SCRATCH_2
                            
                            // Check if location > 0 before decrementing
                            self.move_to(location);
                            self.output.push_str("["); // If location is not zero
                            self.output.push('-'); // Decrement location
                            self.move_to(SCRATCH_3); 
                            self.output.push('+'); // Mark that we decremented
                            self.move_to(location);
                            self.output.push_str("]+"); // End check and restore 1 if we decremented
                            
                            // If we decremented (SCRATCH_3 is set), clear the extra 1 we added
                            self.move_to(SCRATCH_3);
                            self.output.push_str("[-");
                            self.move_to(location);
                            self.output.push('-');
                            self.move_to(SCRATCH_3);
                            self.output.push_str("]");
                            
                            self.move_to(SCRATCH_2);
                            self.output.push('-'); // Decrement SCRATCH_2
                            self.output.push_str("]"); // End loop
                        }
                    }
                    _ => {
                        self.move_to(SCRATCH_2);
                        self.compile(expr)?;
                        self.copy_value(SCRATCH_2, location);
                    }
                }
            }
            BFLNode::Number(n) => {
                self.output.push_str("[-]");
                if *n > 0 {
                    self.output.push_str(&"+".repeat(*n as usize));
                }
            }
            BFLNode::Variable(name) => {
                 let location = self.variables.get(name).copied().ok_or(format!("Variable '{}' not found", name))?;
                 self.copy_value(location, self.current_ptr);
            }
            BFLNode::While(condition, body) => {
                // For simple variable conditions, we can optimize
                match condition.as_ref() {
                    BFLNode::Variable(var_name) => {
                        let var_location = self.variables.get(var_name).copied()
                            .ok_or(format!("Variable '{}' not found in while condition", var_name))?;
                        
                        // Start loop by checking variable directly
                        self.move_to(var_location);
                        self.output.push('[');
                        
                        // Loop body
                        for stmt in body {
                            self.compile(stmt)?;
                        }
                        
                        // Check variable again
                        self.move_to(var_location);
                        self.output.push(']');
                    }
                    _ => {
                        // For complex conditions, use temp cell but more efficiently
                        let temp_cond = self.next_var_location;
                        self.next_var_location += 1;
                        
                        // Evaluate condition into temp cell
                        self.eval_to_cell(condition, temp_cond)?;
                        self.move_to(temp_cond);
                        self.output.push('[');
                        
                        // Loop body
                        for stmt in body {
                            self.compile(stmt)?;
                        }
                        
                        // Re-evaluate condition (needed if body modifies variables)
                        self.eval_to_cell(condition, temp_cond)?;
                        self.move_to(temp_cond);
                        self.output.push(']');
                    }
                }
            }
            BFLNode::If(condition, body) => {
                // Evaluate condition into a temp cell
                let temp_flag = self.next_var_location;
                self.next_var_location += 1;
                // Compile condition at temp location
                self.eval_to_cell(condition, temp_flag)?;
                self.move_to(temp_flag);
                self.output.push('[');
                for stmt in body {
                    self.compile(stmt)?;
                }
                // Ensure pointer is at temp flag cell before clearing it
                self.move_to(temp_flag);
                self.output.push_str("[-]");
                self.output.push(']');
            }
            BFLNode::Syscall(syscall_no, args) => {
                // Compile syscall number into cell 7
                self.move_to(7);
                self.compile(syscall_no)?;

                // Compile arguments into cells 1-6
                for (i, arg) in args.iter().enumerate() {
                    let arg_cell = i + 1;
                    self.move_to(arg_cell);
                    self.compile(arg)?;
                }

                // Execute syscall by moving to any cell and printing '.'
                self.output.push('.');
            }
            // Other nodes like Add, Sub, String, Bytes are handled as part of Assign or other expressions
            _ => return Err(format!("Node type {:?} not implemented at top level", node)),
        }
        Ok(())
    }

    pub fn get_output(&self) -> &str {
        &self.output
    }

    pub fn get_variable_address(&self, name: &str) -> Option<usize> {
        self.variables.get(name).copied()
    }
}
