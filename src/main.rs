use anyhow::Result;
use std::collections::HashMap;
use std::fmt;
use syscalls::{syscall, SyscallArgs, Sysno};
use thiserror::Error;

/* -----------------------------------------------------------------------------------
 * Errors
 * -----------------------------------------------------------------------------------
 */

#[derive(Error, Debug)]
pub enum BfGenError {
    #[error("Invalid syscall number")]
    InvalidSyscallNumber,
    #[error("Invalid variable name")]
    InvalidVarName,
    #[error("Invalid memory access")]
    InvalidMemoryAccess,
    #[error("Invalid assign value")]
    InvalidAssignValue,
}

#[derive(Error, Debug)]
pub enum BfVmError {
    #[error("Syscall failed")]
    SyscallFailed,
    #[error("Invalid memory access")]
    InvalidMemoryAccess,
    #[error("Invalid syscall number")]
    InvalidSyscallNumber,
}

/* -----------------------------------------------------------------------------------
 * AST Nodes
 * -----------------------------------------------------------------------------------
 *
 * BfNode is our AST (Abstract Syntax Tree) node type. We keep it flexible and
 * easy to extend. Each variant captures a language construct we need to generate.
 */

#[derive(Debug, Clone)]
pub enum BfNode {
    // Single byte literal
    Byte(u8),

    // Arbitrary bytes literal
    Bytes(Vec<u8>),

    // Variable reference
    Var(String),

    // Assignment: variable name, expression
    Assign(String, Box<BfNode>),

    // Copy: copies 'size' bytes from source var to destination var
    // Copy(source_var, dest_var, size)
    Copy(String, String, usize),

    // Pointer manipulation (e.g., we might offset from a known var)
    // Ptr(var_name, offset)
    Ptr(String, isize),

    // Syscall: first node is the syscall number, followed by arguments
    Syscall(Box<BfNode>, Vec<BfNode>),

    // Control flow: if cond then block
    If(Box<BfNode>, Box<BfNode>),

    // Control flow: while cond do block
    While(Box<BfNode>, Box<BfNode>),

    // Equality check: eq(a, b)
    Eq(Box<BfNode>, Box<BfNode>),

    // Less-than check: lt(a, b)
    Lt(Box<BfNode>, Box<BfNode>),

    // A sequence of nodes
    Block(Vec<BfNode>),
}

/* -----------------------------------------------------------------------------------
 * Internal Memory Representation for Variables
 * -----------------------------------------------------------------------------------
 *
 * We track memory positions for variables in a conceptual linear memory array.
 * This array is mapped onto Brainfuck cells. MemoryCell tracks where each var
 * is placed, how big it is, etc.
 */

#[derive(Clone)]
pub struct MemoryCell {
    pub ptr: usize,  // Where in the BF cell tape the variable resides
    pub size: usize, // How many cells (bytes) it occupies
}

/* -----------------------------------------------------------------------------------
 * Code Generation Output
 * -----------------------------------------------------------------------------------
 *
 * We keep a list of (bf_code, debug_message, indentation) so we can either
 * produce raw BF code or produce debug lines. This helps with debugging or
 * analyzing generated BF code.
 */

pub struct CGOutput {
    pub bf: String,
    pub debug: String,
    pub indent: usize,
}

impl fmt::Display for CGOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.bf)
    }
}

impl CGOutput {
    pub fn debug_output(&self) -> String {
        // Optionally indent for readability
        let indent_str = " ".repeat(self.indent * 4);
        format!("{}{} // {}", indent_str, self.bf, self.debug)
    }
}

/* -----------------------------------------------------------------------------------
 * Brainfuck Code Generator
 * -----------------------------------------------------------------------------------
 *
 * The code generator walks the AST and generates Brainfuck instructions that
 * produce the desired behavior. We keep track of a conceptual 'current_ptr'
 * that indicates where our BF pointer currently is, so we can generate < or >
 * instructions to move it around as needed.
 */

pub struct BfCodeGenerator {
    pub output: Vec<CGOutput>,
    pub debug: bool,
    pub indent_level: usize,

    // Conceptual memory and variable->cell mapping
    pub memory: Vec<u8>, // Not strictly necessary, but can help track content
    pub cells: HashMap<String, MemoryCell>,
    pub current_ptr: usize, // Where the BF tape pointer is

    // Keep track of the next free cell for "heap" usage
    // Start after syscall memory zone (0-7)
    pub next_free_cell: usize,
}

impl BfCodeGenerator {
    pub fn new(debug: bool) -> Self {
        Self {
            output: Vec::new(),
            debug,
            indent_level: 0,
            memory: vec![],
            cells: HashMap::new(),
            current_ptr: 0,
            next_free_cell: 8, // Start allocations after syscall memory zone
        }
    }

    /* -------------------------------------------------------------------------------
     * Memory Allocation
     * -------------------------------------------------------------------------------
     *
     * We'll keep it simple: we just push memory onto the end of our conceptual array.
     * The BF code generator doesn't necessarily need the real data, but we track it
     * for completeness. This also ensures we know how many cells we've used.
     */

    pub fn alloc_cells(&mut self, size: usize) -> Result<(usize, usize)> {
        let start = self.next_free_cell;
        self.next_free_cell += size;

        // Extend memory array if needed
        if self.memory.len() < self.next_free_cell {
            self.memory.resize(self.next_free_cell, 0);
        }
        Ok((start, size))
    }

    /* -------------------------------------------------------------------------------
     * Output Recording
     * -------------------------------------------------------------------------------
     *
     * We capture small chunks of BF code as CGOutput. This helps us interleave
     * debug messages with raw BF instructions.
     */

    fn push_output(&mut self, bf: String, debug: String) {
        self.output.push(CGOutput {
            bf,
            debug,
            indent: self.indent_level,
        });
    }

    /* -------------------------------------------------------------------------------
     * Pointer Movement
     * -------------------------------------------------------------------------------
     *
     * Moves the BF pointer from self.current_ptr to 'target'.
     */

    fn generate_ptr_move(&mut self, target: usize) -> CGOutput {
        let from = self.current_ptr;
        let bf = if target > from {
            ">".repeat(target - from)
        } else {
            "<".repeat(from - target)
        };
        self.current_ptr = target;

        CGOutput {
            bf,
            debug: format!("move ptr from {} to {}", from, target),
            indent: self.indent_level,
        }
    }

    /* -------------------------------------------------------------------------------
     * Common BF Routines
     * -------------------------------------------------------------------------------
     *
     * - generate_clear_cell: sets current cell to 0, i.e. "[-]"
     * - generate_number: increments current cell to a given value
     */

    fn generate_clear_cell(&self) -> CGOutput {
        CGOutput {
            bf: "[-]".to_string(),
            debug: "clear cell to 0".to_string(),
            indent: self.indent_level,
        }
    }

    fn generate_number(&self, value: u8) -> CGOutput {
        CGOutput {
            bf: "+".repeat(value as usize),
            debug: format!("set cell to {}", value),
            indent: self.indent_level,
        }
    }

    /* -------------------------------------------------------------------------------
     * Simple Byte or Bytes Generation
     * -------------------------------------------------------------------------------
     */

    fn set_byte(&mut self, cell_pos: usize, value: u8) -> Vec<CGOutput> {
        let mut ret = vec![];
        // Move pointer to cell_pos
        ret.push(self.generate_ptr_move(cell_pos));
        // Clear cell
        ret.push(self.generate_clear_cell());
        // Increment to the desired value
        ret.push(self.generate_number(value));
        ret
    }

    fn set_bytes(&mut self, bytes: &[u8]) -> Vec<CGOutput> {
        let mut ret = vec![];
        let (start_pos, size) = self
            .alloc_cells(bytes.len()) // might fail in real code, ignoring
            .expect("Failed to alloc for set_bytes");

        // For each byte, set it
        for (i, &byte) in bytes.iter().enumerate() {
            ret.extend(self.set_byte(start_pos + i, byte));
        }
        ret
    }

    /* -------------------------------------------------------------------------------
     * Copy Bytes (source_var -> dest_var)
     * -------------------------------------------------------------------------------
     *
     * This is a standard Brainfuck trick:
     *   We'll do two passes:
     *    - a "transfer" loop that drains 'source' into 'dest' and a 'temp' cell
     *    - a "restore" loop that moves from 'temp' back into 'source'
     */

    fn copy_bytes_impl(&mut self, src_pos: usize, dst_pos: usize, size: usize) -> Result<()> {
        // For each byte
        for offset in 0..size {
            let from = src_pos + offset;
            let to = dst_pos + offset;
            // We'll use to+1 as our temp cell
            let temp = to + 1;

            self.indent_level += 1;

            // Store the move result first
            let move_result = self.generate_ptr_move(from);
            self.output.push(move_result);

            // Transfer loop: moves from 'from' to 'to' and 'temp'
            //  [-  (while source not zero)
            //    >... move pointer to 'to', increment
            //    >... move pointer to 'temp', increment
            //    <... back to 'from'
            //  ]
            {
                let mut bf = String::new();
                bf.push_str("[-"); // start loop
                                   // move to 'to'
                if to >= from {
                    bf.push_str(&">".repeat(to - from));
                } else {
                    bf.push_str(&"<".repeat(from - to));
                }
                bf.push('+'); // increment 'to'

                // move to 'temp'
                bf.push('>');
                bf.push('+'); // increment 'temp'

                // move back to 'from'
                if to >= from {
                    bf.push_str(&"<".repeat(to - from + 1));
                } else {
                    bf.push_str(&">".repeat(from - to - 1));
                }

                bf.push(']'); // end loop

                self.push_output(
                    bf,
                    format!("copy (first pass) from cell {} to cell {}", from, to),
                );
            }

            // Move to 'temp' - store intermediate results
            let move_to_from = self.generate_ptr_move(from);
            self.output.push(move_to_from);
            let move_to_temp = self.generate_ptr_move(temp);
            self.output.push(move_to_temp);

            // Restoration loop: move from 'temp' back into 'from'
            //  [-  (while temp not zero)
            //    >... move to 'from', increment
            //    <... back to 'temp'
            //  ]
            {
                let mut bf = String::new();
                bf.push_str("[-");
                // move to 'from'
                if from >= temp {
                    bf.push_str(&">".repeat(from - temp));
                } else {
                    bf.push_str(&"<".repeat(temp - from));
                }
                bf.push('+'); // increment 'from'

                // move back to 'temp'
                if from >= temp {
                    bf.push_str(&"<".repeat(from - temp));
                } else {
                    bf.push_str(&">".repeat(temp - from));
                }
                bf.push(']'); // end loop

                self.push_output(bf, format!("restore source at cell {}", from));
            }

            // Move pointer to 'to' as the final position - store intermediate results
            let move_to_temp_final = self.generate_ptr_move(temp);
            self.output.push(move_to_temp_final);
            let move_to_to = self.generate_ptr_move(to);
            self.output.push(move_to_to);
            self.indent_level -= 1;
        }

        Ok(())
    }

    /* -------------------------------------------------------------------------------
     * Master Generate Function
     * -------------------------------------------------------------------------------
     *
     * Walk the BfNode and produce Brainfuck instructions. We rely on helper
     * methods for smaller tasks (bytes, copy, etc.).
     */

    pub fn generate(&mut self, node: &BfNode) -> Result<()> {
        match node {
            BfNode::Byte(value) => {
                let (pos, _) = self.alloc_cells(1)?;
                let code = self.set_byte(pos, *value);
                self.output.extend(code);
            }

            BfNode::Bytes(bytes) => {
                let code = self.set_bytes(bytes);
                self.output.extend(code);
            }

            BfNode::Var(name) => {
                // Get cell info first
                let cell = self
                    .cells
                    .get(name)
                    .ok_or_else(|| BfGenError::InvalidVarName)?
                    .clone(); // Clone to avoid borrow issues

                // Allocate new region of same size
                let (new_pos, _) = self.alloc_cells(cell.size)?;
                // Perform the copy
                self.copy_bytes_impl(cell.ptr, new_pos, cell.size)?;
            }

            BfNode::Assign(var_name, expr) => {
                // Evaluate 'expr' into newly allocated memory
                let before_alloc = self.next_free_cell;
                self.generate(expr)?; // This will allocate
                let after_alloc = self.next_free_cell;

                // The new expression's data is presumably in the range [before_alloc, after_alloc)
                let size = after_alloc - before_alloc;
                let cell_ptr = before_alloc;

                // Register it in cells
                self.cells.insert(
                    var_name.clone(),
                    MemoryCell {
                        ptr: cell_ptr,
                        size,
                    },
                );
            }

            BfNode::Copy(src_var, dst_var, size) => {
                // Get source and destination info first
                let src_cell = self
                    .cells
                    .get(src_var)
                    .ok_or_else(|| BfGenError::InvalidVarName)?
                    .clone(); // Clone to avoid borrow issues
                let dst_cell = self
                    .cells
                    .get(dst_var)
                    .ok_or_else(|| BfGenError::InvalidVarName)?
                    .clone(); // Clone to avoid borrow issues

                if src_cell.size < *size || dst_cell.size < *size {
                    return Err(BfGenError::InvalidMemoryAccess.into());
                }

                self.copy_bytes_impl(src_cell.ptr, dst_cell.ptr, *size)?;
            }

            BfNode::Ptr(var_name, offset) => {
                // Get base cell info first
                let base_cell = self
                    .cells
                    .get(var_name)
                    .ok_or_else(|| BfGenError::InvalidVarName)?
                    .clone(); // Clone to avoid borrow issues

                // The final cell is base_cell.ptr + offset
                let final_ptr = if offset.is_negative() {
                    base_cell.ptr.checked_sub(offset.unsigned_abs())
                } else {
                    base_cell.ptr.checked_add(*offset as usize)
                }
                .ok_or_else(|| BfGenError::InvalidMemoryAccess)?;

                // Allocate and store pointer value
                let (pos, _) = self.alloc_cells(1)?;
                let move_result = self.generate_ptr_move(pos);
                self.output.push(move_result);
                self.output.push(self.generate_clear_cell());

                // Not realistic to produce final_ptr increments if final_ptr is large, but for demo:
                let increments = final_ptr.min(255);
                self.output.push(CGOutput {
                    bf: "+".repeat(increments),
                    debug: format!("(demo) set pointer address = {}", final_ptr),
                    indent: self.indent_level,
                });
            }

            BfNode::Syscall(num_expr, args_exprs) => {
                // 1) Evaluate the syscall number into a new cell
                let before_num = self.next_free_cell;
                self.generate(num_expr)?;
                let syscall_num_cell = before_num;

                // 2) Evaluate each argument
                let mut arg_cells = vec![];
                for arg_expr in args_exprs {
                    let before_arg = self.next_free_cell;
                    self.generate(arg_expr)?;
                    let after_arg = self.next_free_cell;
                    arg_cells.push(before_arg..after_arg);
                }

                // 3) Set up syscall
                {
                    // Move pointer to 0 and set marker
                    let move_to_zero = self.generate_ptr_move(0);
                    self.output.push(move_to_zero);
                    self.output.push(self.generate_clear_cell());
                    self.output.push(self.generate_number(255));

                    // Copy the syscall number to cell 1
                    self.copy_bytes_impl(syscall_num_cell, 1, 1)?;

                    // Copy each argument to subsequent cells
                    let mut cell_idx = 2;
                    for rng in arg_cells {
                        let len = rng.end - rng.start;
                        self.copy_bytes_impl(rng.start, cell_idx, len)?;
                        cell_idx += len;
                    }

                    // Trigger syscall
                    self.push_output(".".to_string(), "trigger syscall".to_string());
                }
            }

            BfNode::If(cond_expr, body_expr) => {
                // Evaluate the condition into a cell
                let before_cond = self.next_free_cell;
                self.generate(cond_expr)?;
                let cond_cell = before_cond;

                // Move to condition cell and start loop
                let move_result = self.generate_ptr_move(cond_cell);
                self.output.push(move_result);
                self.push_output("[".to_string(), "if: start loop".to_string());

                // Indent and generate body
                self.indent_level += 1;
                self.generate(body_expr)?;

                // Clear condition and end loop
                let move_back = self.generate_ptr_move(cond_cell);
                self.output.push(move_back);
                self.output.push(self.generate_clear_cell());

                self.indent_level -= 1;
                self.push_output("]".to_string(), "if: end loop".to_string());
            }

            BfNode::While(cond_expr, body_expr) => {
                // Evaluate condition first
                let before_cond = self.next_free_cell;
                self.generate(cond_expr)?;
                let cond_cell = before_cond;

                // Move to condition and start loop
                let move_result = self.generate_ptr_move(cond_cell);
                self.output.push(move_result);
                self.push_output("[".to_string(), "while: start loop".to_string());

                self.indent_level += 1;
                self.generate(body_expr)?;
                self.generate(cond_expr)?;
                self.indent_level -= 1;

                self.push_output("]".to_string(), "while: end loop".to_string());
            }

            BfNode::Eq(a_expr, b_expr) => {
                // eq(a, b) sets up a cell that is 1 if a == b, 0 otherwise
                let a_start = self.next_free_cell;
                self.generate(a_expr)?;
                let a_end = self.next_free_cell;
                let a_size = a_end - a_start;

                let b_start = self.next_free_cell;
                self.generate(b_expr)?;
                let b_end = self.next_free_cell;
                let b_size = b_end - b_start;

                if a_size == 1 && b_size == 1 {
                    let a_cell = a_start;
                    let b_cell = b_start;

                    let (result_cell, _) = self.alloc_cells(1)?;
                    self.copy_bytes_impl(a_cell, result_cell, 1)?;

                    let move_result = self.generate_ptr_move(b_cell);
                    self.output.push(move_result);

                    let mut bf = String::new();
                    bf.push_str("[-");
                    if result_cell >= b_cell {
                        bf.push_str(&">".repeat(result_cell - b_cell));
                    } else {
                        bf.push_str(&"<".repeat(b_cell - result_cell));
                    }
                    bf.push('-');
                    if result_cell >= b_cell {
                        bf.push_str(&"<".repeat(result_cell - b_cell));
                    } else {
                        bf.push_str(&">".repeat(b_cell - result_cell));
                    }
                    bf.push(']');
                    self.push_output(bf, "subtract b from result".to_string());
                } else {
                    self.push_output(
                        "".to_string(),
                        "eq on multi-byte: not implemented".to_string(),
                    );
                }
            }

            BfNode::Lt(a_expr, b_expr) => {
                // We'll do a minimal version: single-byte (a < b)
                let a_start = self.next_free_cell;
                self.generate(a_expr)?;
                let a_end = self.next_free_cell;

                let b_start = self.next_free_cell;
                self.generate(b_expr)?;
                let b_end = self.next_free_cell;

                if (a_end - a_start) == 1 && (b_end - b_start) == 1 {
                    // We would do a < b. For a real solution, we might create code to do so.
                    self.push_output("".to_string(), "lt(a,b) single-byte: stub".to_string());
                } else {
                    self.push_output(
                        "".to_string(),
                        "lt on multi-byte: not implemented".to_string(),
                    );
                }
            }

            BfNode::Block(nodes) => {
                // Just generate code for each child in order
                let block_start = CGOutput {
                    bf: "".to_string(),
                    debug: "begin block".to_string(),
                    indent: self.indent_level,
                };
                self.output.push(block_start);

                self.indent_level += 1;
                for n in nodes {
                    self.generate(n)?;
                }
                self.indent_level -= 1;

                let block_end = CGOutput {
                    bf: "".to_string(),
                    debug: "end block".to_string(),
                    indent: self.indent_level,
                };
                self.output.push(block_end);
            }
        }
        Ok(())
    }

    /* -------------------------------------------------------------------------------
     * Final Output
     * -------------------------------------------------------------------------------
     *
     * If debug mode is ON, we produce lines with comments.
     * Otherwise, we produce raw BF instructions concatenated.
     */

    pub fn get_formatted_output(&self) -> String {
        if self.debug {
            self.output
                .iter()
                .map(|o| o.debug_output())
                .collect::<Vec<_>>()
                .join("\n")
        } else {
            self.output
                .iter()
                .map(|o| o.bf.as_str())
                .collect::<Vec<_>>()
                .join("")
        }
    }
}

/* -----------------------------------------------------------------------------------
 * Brainfuck VM
 * -----------------------------------------------------------------------------------
 *
 * A simplistic BF interpreter. We skip implementing loops fully for brevity; if
 * you'd like a fully functional BF interpreter, you'd implement bracket matching,
 * etc. The key piece is that we detect '.' with memory[self.ptr] == 255 as a marker
 * for a syscall. Everything else is standard BF steps.
 */

pub struct BrainfuckVM {
    pub memory: Vec<u8>,
    pub ptr: usize,
    pub debug: bool,
    // Memory layout:
    // [0]: syscall marker (255 indicates syscall)
    // [1]: syscall number
    // [2-7]: syscall arguments (up to 6)
    // [8+]: heap space for program use
    pub heap_start: usize,
}

impl BrainfuckVM {
    pub fn new(debug: bool, memory_size: usize) -> Self {
        Self {
            memory: vec![0; memory_size],
            ptr: 0,
            debug,
            heap_start: 8, // First 8 cells (0-7) reserved for syscall
        }
    }

    fn move_ptr(&mut self, offset: isize) -> Result<()> {
        let new_ptr = if offset >= 0 {
            self.ptr.checked_add(offset as usize)
        } else {
            self.ptr.checked_sub((-offset) as usize)
        };
        match new_ptr {
            Some(np) if np < self.memory.len() => {
                self.ptr = np;
                Ok(())
            }
            _ => Err(BfVmError::InvalidMemoryAccess.into()),
        }
    }

    pub fn run(&mut self, code: &str) -> Result<()> {
        let mut pc = 0;
        let instructions: Vec<char> = code.chars().collect();

        while pc < instructions.len() {
            match instructions[pc] {
                '>' => self.move_ptr(1)?,
                '<' => self.move_ptr(-1)?,
                '+' => {
                    self.memory[self.ptr] = self.memory[self.ptr].wrapping_add(1);
                }
                '-' => {
                    self.memory[self.ptr] = self.memory[self.ptr].wrapping_sub(1);
                }
                '.' => {
                    // If we see '.' and the cell has 255 => syscall
                    if self.memory[self.ptr] == 255 {
                        self.handle_syscall()?;
                    } else {
                        // For normal '.' we output the character in memory
                        print!("{}", self.memory[self.ptr] as char);
                    }
                }
                ',' => {
                    // Basic input read
                    let mut input = String::new();
                    std::io::stdin().read_line(&mut input)?;
                    if let Some(ch) = input.chars().next() {
                        self.memory[self.ptr] = ch as u8;
                    }
                }
                '[' | ']' => {
                    // Not fully implemented. In real code, you'd implement bracket matching
                    // and jumps. We'll skip it for brevity.
                    // This means "If" / "While" won't truly work in this basic VM unless
                    // you do bracket matching, etc.
                }
                _ => { /* ignore any other chars */ }
            }
            pc += 1;
        }
        Ok(())
    }

    fn handle_syscall(&mut self) -> Result<()> {
        // We assume memory layout:
        // cell[0] = 255 indicates do_syscall
        // cell[1] = syscall number
        // cell[2..8] = up to 6 arguments
        let syscall_num = self.memory[1];
        let args: Vec<usize> = self.memory[2..8].iter().map(|&x| x as usize).collect();

        // Attempt the syscall
        let nr = Sysno::from(syscall_num as i32);
        let sa = SyscallArgs::new(args[0], args[1], args[2], args[3], args[4], args[5]);

        let result = unsafe {
            match syscall(nr, &sa) {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("Syscall failed: {}", e);
                    return Err(BfVmError::SyscallFailed.into());
                }
            }
        };

        // Store result in cell[0] for demonstration
        self.memory[0] = result as u8;
        Ok(())
    }
}

/* -----------------------------------------------------------------------------------
 * Example Main
 * -----------------------------------------------------------------------------------
 *
 * This demonstrates building a small AST and generating BF code, then running it.
 */

fn main() -> Result<()> {
    // "Hello World!"
    let message = "Hello World!".as_bytes().to_vec();

    // Example program:
    //   var message = "Hello World!"
    //   syscall(1, message, message.len())
    let program = BfNode::Block(vec![
        BfNode::Assign(
            "message".to_string(),
            Box::new(BfNode::Bytes(message.clone())),
        ),
        BfNode::Syscall(
            // Syscall number 1 (on many systems = write)
            Box::new(BfNode::Byte(1)),
            vec![
                BfNode::Byte(1),
                // Arg0 => pointer to 'message'
                BfNode::Ptr("message".to_string(), 0),
                // Arg1 => length of 'message'
                BfNode::Byte(14),
            ],
        ),
    ]);

    let mut gen = BfCodeGenerator::new(true);
    gen.generate(&program)?;

    println!("=== Generated Brainfuck Code (Debug Mode) ===");
    println!("{}", gen.get_formatted_output());
    println!("\n=== Execution Result ===");

    let mut vm = BrainfuckVM::new(true, 1024 * 1024);
    vm.run(&gen.get_formatted_output())?;

    println!("{:?}", vm.memory[0..32].to_vec());

    Ok(())
}
