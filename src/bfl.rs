use std::collections::HashMap;

// Use cells closer to variables for better efficiency
const SCRATCH_1: usize = 100;
const SCRATCH_2: usize = 101; // Adjacent to SCRATCH_1 for efficient copying

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
        // The return value of a syscall is always in cell 0
        variables.insert("_syscall_result".to_string(), 0);
        BFLCompiler {
            variables,
            next_var_location: 8, // Start user variables after syscall reserved area
            output: String::new(),
            current_ptr: 0,
        }
    }

    // A clean, simple, and correct pointer movement function.
    fn move_to(&mut self, target: usize) {
        if self.current_ptr == target {
            return;
        }
        if target > self.current_ptr {
            self.output.push_str(&">".repeat(target - self.current_ptr));
        } else {
            self.output.push_str(&"<".repeat(self.current_ptr - target));
        }
        self.current_ptr = target;
    }

    // Optimized copy for adjacent cells
    fn copy_adjacent(&mut self, src: usize, dest: usize) {
        if src == dest {
            return;
        }
        
        // For adjacent cells, use the standard BF copy pattern: [->+<]
        if src + 1 == dest {
            // src and dest are adjacent, src < dest
            self.move_to(src);
            self.output.push_str("[->+<]");
            self.current_ptr = dest;
        } else if dest + 1 == src {
            // src and dest are adjacent, dest < src
            self.move_to(dest);
            self.output.push_str("[->+<]");
            self.current_ptr = src;
        } else {
            // Not adjacent, fall back to general copy
            self.copy_value(src, dest);
        }
    }

    // Non-destructive copy from src to dest. Pointer ends at dest.
    // This version is much more efficient than the original.
    fn copy_value(&mut self, src: usize, dest: usize) {
        if src == dest {
            self.move_to(dest);
            return;
        }

        // 1. Clear destination and scratch cell
        self.move_to(dest);
        self.output.push_str("[-]");
        self.move_to(SCRATCH_1);
        self.output.push_str("[-]");

        // 2. Move value from src to dest and scratch
        self.move_to(src);
        self.output.push_str("["); // while src is not zero
        self.move_to(dest);
        self.output.push('+'); // dest++
        self.move_to(SCRATCH_1);
        self.output.push('+'); // scratch++
        self.move_to(src);
        self.output.push('-'); // src--
        self.output.push_str("]");

        // 3. Restore value from scratch to src
        self.move_to(SCRATCH_1);
        self.output.push_str("["); // while scratch is not zero
        self.move_to(src);
        self.output.push('+'); // src++
        self.move_to(SCRATCH_1);
        self.output.push('-'); // scratch--
        self.output.push_str("]");

        // 4. Ensure pointer ends at dest
        self.move_to(dest);
    }

    /// Peephole optimizer to remove redundant sequences
    fn optimize_output(&mut self) {
        let mut optimized = String::new();
        let chars: Vec<char> = self.output.chars().collect();
        let mut i = 0;
        
        while i < chars.len() {
            // Remove redundant pointer movements: >< or <>
            if i + 1 < chars.len() {
                match (chars[i], chars[i + 1]) {
                    ('>', '<') | ('<', '>') => {
                        i += 2; // Skip both characters
                        continue;
                    }
                    _ => {}
                }
            }
            
            // Remove redundant increments/decrements: +- or -+
            if i + 1 < chars.len() {
                match (chars[i], chars[i + 1]) {
                    ('+', '-') | ('-', '+') => {
                        i += 2; // Skip both characters
                        continue;
                    }
                    _ => {}
                }
            }
            
            // Remove redundant loops: [][]
            if i + 3 < chars.len() && chars[i] == '[' && chars[i + 1] == ']' && chars[i + 2] == '[' && chars[i + 3] == ']' {
                i += 4; // Skip all four characters
                continue;
            }
            
            optimized.push(chars[i]);
            i += 1;
        }
        
        self.output = optimized;
    }

    /// Return an optimized version of the output without modifying internal state
    pub fn get_optimized_output_copy(&self) -> String {
        let mut optimized = String::new();
        let chars: Vec<char> = self.output.chars().collect();
        let mut i = 0;
        
        while i < chars.len() {
            // Remove redundant pointer movements: >< or <>
            if i + 1 < chars.len() {
                match (chars[i], chars[i + 1]) {
                    ('>', '<') | ('<', '>') => {
                        i += 2; // Skip both characters
                        continue;
                    }
                    _ => {}
                }
            }
            
            // Remove redundant increments/decrements: +- or -+
            if i + 1 < chars.len() {
                match (chars[i], chars[i + 1]) {
                    ('+', '-') | ('-', '+') => {
                        i += 2; // Skip both characters
                        continue;
                    }
                    _ => {}
                }
            }
            
            // Remove redundant loops: [][]
            if i + 3 < chars.len() && chars[i] == '[' && chars[i + 1] == ']' && chars[i + 2] == '[' && chars[i + 3] == ']' {
                i += 4; // Skip all four characters
                continue;
            }
            
            optimized.push(chars[i]);
            i += 1;
        }
        
        optimized
    }

    /// Evaluate an expression, storing its final value in the specified cell.
    /// The pointer will end at the `dest` cell.
    fn eval_to_cell(&mut self, expr: &BFLNode, dest: usize) -> Result<(), String> {
        match expr {
            BFLNode::Number(n) => {
                self.move_to(dest);
                self.output.push_str("[-]"); // Clear cell
                if *n > 0 {
                    self.output.push_str(&"+".repeat(*n as usize));
                }
            }
            BFLNode::Variable(name) => {
                let src = *self.variables.get(name).ok_or(format!("Variable '{}' not found", name))?;
                // Use optimized copy for adjacent cells, fall back to general copy
                if (src == SCRATCH_1 && dest == SCRATCH_2) || (src == SCRATCH_2 && dest == SCRATCH_1) {
                    // Special case for scratch cells - they're adjacent
                    self.copy_adjacent(src, dest);
                } else {
                    self.copy_value(src, dest);
                }
            }
            BFLNode::String(s) => {
                self.eval_to_cell(&BFLNode::Bytes(s.as_bytes().to_vec()), dest)?;
            }
            BFLNode::Bytes(bytes) => {
                let data_location = self.next_var_location;
                self.next_var_location += bytes.len();

                // Store pointer to data in the dest cell
                self.move_to(dest);
                self.output.push_str("[-]");
                self.output.push_str(&"+".repeat(data_location));

                // Write the actual bytes to memory
                for (i, byte) in bytes.iter().enumerate() {
                    self.move_to(data_location + i);
                    self.output.push_str("[-]");
                    self.output.push_str(&"+".repeat(*byte as usize));
                }
                self.move_to(dest); // Leave pointer at the destination (which now holds the address)
            }
            BFLNode::Add(lhs, rhs) => {
                self.eval_to_cell(lhs, dest)?; // Evaluate LHS into dest
                self.eval_to_cell(rhs, SCRATCH_1)?; // Evaluate RHS into scratch
                
                // Add SCRATCH_1 to dest using optimized pattern
                self.move_to(SCRATCH_1);
                self.output.push_str("["); // while scratch is not zero
                self.move_to(dest);
                self.output.push('+'); // dest++
                self.move_to(SCRATCH_1);
                self.output.push('-'); // scratch--
                self.output.push_str("]");
                self.move_to(dest);
            }
            BFLNode::Sub(lhs, rhs) => {
                self.eval_to_cell(lhs, dest)?; // Evaluate LHS into dest
                self.eval_to_cell(rhs, SCRATCH_1)?; // Evaluate RHS into scratch
                
                // Subtract SCRATCH_1 from dest (clamping at 0)
                self.move_to(SCRATCH_1);
                self.output.push_str("["); // while scratch is not zero
                self.move_to(dest);
                self.output.push('-'); // dest--
                self.move_to(SCRATCH_1);
                self.output.push('-'); // scratch--
                self.output.push_str("]");
                self.move_to(dest);
            }
            _ => return Err(format!("Cannot evaluate this node type directly: {:?}", expr)),
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
                self.eval_to_cell(expr, location)?;
            }
            BFLNode::While(condition, body) => {
                let cond_loc = SCRATCH_2;
                self.eval_to_cell(condition, cond_loc)?; // Initial condition check
                self.move_to(cond_loc);
                self.output.push('['); // Loop while condition is non-zero
                
                for stmt in body {
                    self.compile(stmt)?;
                }
                
                self.eval_to_cell(condition, cond_loc)?; // Re-evaluate condition at the end of the loop
                self.move_to(cond_loc);
                self.output.push(']');
            }
            BFLNode::If(condition, body) => {
                let cond_loc = SCRATCH_2;
                self.eval_to_cell(condition, cond_loc)?;
                self.move_to(cond_loc);
                self.output.push('['); // If condition is non-zero
                
                for stmt in body {
                    self.compile(stmt)?;
                }
                
                // Clear the flag to ensure the 'if' block runs only once
                self.move_to(cond_loc);
                self.output.push_str("[-]");
                self.output.push(']');
            }
            BFLNode::Syscall(syscall_no, args) => {
                // Evaluate syscall number into cell 7
                self.eval_to_cell(syscall_no, 7)?;

                // Evaluate arguments into cells 1-6
                for (i, arg) in args.iter().enumerate() {
                    if i >= 6 {
                        return Err("Too many syscall arguments (max 6)".to_string());
                    }
                    let arg_cell = i + 1;
                    self.eval_to_cell(arg, arg_cell)?;
                }

                // Execute syscall
                self.output.push('.');
            }
            // Expressions are handled by `eval_to_cell` and shouldn't be top-level statements
            _ => return Err(format!("Node type {:?} cannot be a top-level statement", node)),
        }
        Ok(())
    }

    pub fn get_output(&self) -> &str {
        &self.output
    }

    pub fn get_optimized_output(&mut self) -> &str {
        self.optimize_output();
        &self.output
    }

    pub fn get_variable_address(&self, name: &str) -> Option<usize> {
        self.variables.get(name).copied()
    }
}
