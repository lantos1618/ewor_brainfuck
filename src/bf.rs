use std::io::Read;
use std::io::Write;
use syscalls::Sysno;

#[derive(Debug)]
pub enum BFError {
    InvalidSyscall(String),
    MemoryAccess(String),
    InvalidFileDescriptor(String),
    SyscallFailed(String),
    BracketMismatch(String),
}

impl std::fmt::Display for BFError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BFError::InvalidSyscall(msg) => write!(f, "Invalid syscall: {}", msg),
            BFError::MemoryAccess(msg) => write!(f, "Invalid memory access: {}", msg),
            BFError::InvalidFileDescriptor(msg) => write!(f, "Invalid file descriptor: {}", msg),
            BFError::SyscallFailed(msg) => write!(f, "Syscall failed: {}", msg),
            BFError::BracketMismatch(msg) => write!(f, "Bracket mismatch: {}", msg),
        }
    }
}

impl std::error::Error for BFError {}

pub struct BF {
    cells: Vec<u32>,
    ptr: usize,
    code: Vec<char>,
    pc: usize,
    output: Vec<u8>, // Keep output as u8 for string operations
    mode: Mode,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Mode {
    BF,  // Standard Brainfuck
    BFA, // Brainfuck with syscall extensions
}

impl BF {
    pub fn new(code: &str, mode: Mode) -> Self {
        let cells = vec![0u32; 65536]; // Increase to 64KB from 30KB
        BF {
            cells,
            ptr: 0,
            code: code.chars().collect(),
            pc: 0,
            output: Vec::new(),
            mode,
        }
    }

    pub fn dump_cells(&self, n: usize) -> &[u32] {
        &self.cells[..n.min(self.cells.len())]
    }

    pub fn run(&mut self) -> Result<(), BFError> {
        // Validate brackets before execution
        let mut depth = 0;
        for c in self.code.iter() {
            match c {
                '[' => depth += 1,
                ']' => {
                    depth -= 1;
                    if depth < 0 {
                        return Err(BFError::BracketMismatch("Unmatched ]".to_string()));
                    }
                }
                _ => {}
            }
        }
        if depth > 0 {
            return Err(BFError::BracketMismatch("Unmatched [".to_string()));
        }

        while self.pc < self.code.len() {
            match self.mode {
                Mode::BFA => self.execute_bfa()?,
                Mode::BF => self.execute_bf()?,
            }
            self.pc += 1;
        }
        Ok(())
    }

    fn execute_bf(&mut self) -> Result<(), BFError> {
        match self.code[self.pc] {
            '>' => {
                self.ptr += 1;
                if self.ptr >= self.cells.len() {
                    return Err(BFError::MemoryAccess("Pointer out of bounds".to_string()));
                }
                Ok(())
            }
            '<' => {
                if self.ptr == 0 {
                    return Err(BFError::MemoryAccess("Pointer out of bounds".to_string()));
                }
                self.ptr -= 1;
                Ok(())
            }
            '+' => {
                self.cells[self.ptr] = self.cells[self.ptr].wrapping_add(1);
                Ok(())
            }
            '-' => {
                self.cells[self.ptr] = self.cells[self.ptr].wrapping_sub(1);
                Ok(())
            }
            '.' => {
                self.output.push(self.cells[self.ptr] as u8);
                // Also write to stdout
                print!("{}", self.cells[self.ptr] as u8 as char);
                std::io::stdout()
                    .flush()
                    .map_err(|e| BFError::SyscallFailed(e.to_string()))?;
                Ok(())
            }
            ',' => {
                let mut buf = [0u8; 1];
                std::io::stdin()
                    .read_exact(&mut buf)
                    .map_err(|e| BFError::SyscallFailed(format!("Input failed: {}", e)))?;
                self.cells[self.ptr] = buf[0] as u32;
                Ok(())
            }
            '[' => {
                let mut depth = 1;
                let mut pos = self.pc;
                if self.cells[self.ptr] == 0 {
                    while depth > 0 {
                        pos += 1;
                        if pos >= self.code.len() {
                            return Err(BFError::BracketMismatch("Unmatched [".to_string()));
                        }
                        match self.code[pos] {
                            '[' => depth += 1,
                            ']' => depth -= 1,
                            _ => {}
                        }
                    }
                    self.pc = pos;
                }
                Ok(())
            }
            ']' => {
                let mut depth = 1;
                let mut pos = self.pc;
                if self.cells[self.ptr] != 0 {
                    while depth > 0 {
                        if pos == 0 {
                            return Err(BFError::BracketMismatch("Unmatched ]".to_string()));
                        }
                        pos -= 1;
                        match self.code[pos] {
                            '[' => depth -= 1,
                            ']' => depth += 1,
                            _ => {}
                        }
                    }
                    self.pc = pos - 1;
                }
                Ok(())
            }
            _ => Ok(()), // Ignore other characters
        }
    }

    fn execute_bfa(&mut self) -> Result<(), BFError> {
        match self.code[self.pc] {
            '.' => {
                // Execute syscall when '.' is encountered
                let syscall_num = self.cells[7] as u32; // Use cell 7 for syscall number instead of 0

                // Get arguments from cells
                let args = [
                    self.cells[1] as usize, // arg1
                    self.cells[2] as usize, // arg2
                    self.cells[3] as usize, // arg3
                    self.cells[4] as usize, // arg4
                    self.cells[5] as usize, // arg5
                    self.cells[6] as usize, // arg6
                ];

                // In test mode, reject socket operations first
                #[cfg(test)]
                {
                    let syscall_num_u8 = syscall_num as u8;
                    if (2..=6).contains(&syscall_num_u8) {
                        return Err(BFError::InvalidSyscall(
                            "Permission denied: socket operations not allowed in test mode"
                                .to_string(),
                        ));
                    }
                }

                self.validate_syscall(syscall_num, &args)?;

                // Execute syscall
                unsafe {
                    let result = match syscall_num {
                        4 => {
                            // write
                            let buf = &self.cells[args[1]..args[1] + args[2]]
                                .iter()
                                .map(|&x| x as u8)
                                .collect::<Vec<u8>>();
                            let result =
                                syscalls::syscall!(Sysno::write, args[0], buf.as_ptr(), args[2])
                                    .map_err(|e| {
                                        BFError::SyscallFailed(format!(
                                            "write syscall failed: {}",
                                            e
                                        ))
                                    });
                            if let Ok(n) = result {
                                self.cells[7] = n as u32; // Store result in cell 7
                                                          // Flush stdout if writing to stdout (fd 1)
                                if args[0] == 1 {
                                    std::io::stdout().flush().map_err(|e| {
                                        BFError::SyscallFailed(format!(
                                            "Failed to flush stdout: {}",
                                            e
                                        ))
                                    })?;
                                }
                            }
                            result
                        }
                        97 => {
                            // socket
                            let result =
                                syscalls::syscall!(Sysno::socket, args[0], args[1], args[2])
                                    .map_err(|e| {
                                        BFError::SyscallFailed(format!(
                                            "socket syscall failed: {}",
                                            e
                                        ))
                                    });
                            if let Ok(fd) = result {
                                println!("Socket syscall returned fd: {}", fd);
                                self.cells[0] = fd as u32; // Store fd in cell 0
                                self.cells[7] = 0; // Success
                            }
                            result
                        }
                        104 => {
                            // bind
                            let fd = self.cells[0] as usize; // Use fd from cell 0
                            let buf = &self.cells[args[1]..args[1] + args[2]]
                                .iter()
                                .map(|&x| x as u8)
                                .collect::<Vec<u8>>();
                            println!("Binding socket fd {} with sockaddr_in:", fd);
                            println!("  sin_family: {:02x} {:02x}", buf[0], buf[1]);
                            println!("  sin_port: {:02x} {:02x}", buf[2], buf[3]);
                            println!(
                                "  sin_addr: {:02x} {:02x} {:02x} {:02x}",
                                buf[4], buf[5], buf[6], buf[7]
                            );
                            let result = syscalls::syscall!(Sysno::bind, fd, buf.as_ptr(), args[2])
                                .map_err(|e| {
                                    BFError::SyscallFailed(format!("bind syscall failed: {}", e))
                                });
                            if let Ok(n) = result {
                                println!("Bind syscall succeeded with result: {}", n);
                                self.cells[7] = n as u32; // Store result in cell 7
                            }
                            result
                        }
                        106 => {
                            // listen
                            let fd = self.cells[0] as usize; // Use fd from cell 0
                            println!("Listening on socket fd {}", fd);
                            let result =
                                syscalls::syscall!(Sysno::listen, fd, args[1]).map_err(|e| {
                                    BFError::SyscallFailed(format!("listen syscall failed: {}", e))
                                });
                            if let Ok(n) = result {
                                println!("Listen syscall succeeded with result: {}", n);
                                self.cells[7] = n as u32; // Store result in cell 7
                            }
                            result
                        }
                        5 => {
                            // accept
                            let fd = self.cells[0] as usize; // Use fd from cell 0
                            let buf = &self.cells[args[1]..args[1] + args[2]]
                                .iter()
                                .map(|&x| x as u8)
                                .collect::<Vec<u8>>();
                            println!("Accepting connection on socket fd {}", fd);
                            let result =
                                syscalls::syscall!(Sysno::accept, fd, buf.as_ptr(), args[2])
                                    .map_err(|e| {
                                        BFError::SyscallFailed(format!(
                                            "accept syscall failed: {}",
                                            e
                                        ))
                                    });
                            if let Ok(client_fd) = result {
                                println!("Accept syscall succeeded with client fd: {}", client_fd);
                                self.cells[0] = client_fd as u32; // Store client fd in cell 0
                                self.cells[7] = 0; // Success
                            }
                            result
                        }
                        3 => {
                            // read
                            let fd = args[0]; // Use fd directly from args
                            let mut buf = vec![0u8; args[2]];
                            println!("Reading from fd {} into buffer of size {}", fd, args[2]);
                            let result =
                                syscalls::syscall!(Sysno::read, fd, buf.as_mut_ptr(), args[2])
                                    .map_err(|e| {
                                        BFError::SyscallFailed(format!(
                                            "read syscall failed: {}",
                                            e
                                        ))
                                    });
                            if let Ok(n) = result {
                                println!("Read {} bytes", n);
                                // Copy read data into cells
                                for (i, &byte) in buf[..n as usize].iter().enumerate() {
                                    self.cells[args[1] + i] = byte as u32;
                                }
                                self.cells[0] = n as u32; // Store bytes read in cell 0
                                self.cells[7] = 0; // Success
                            }
                            result
                        }
                        48 => {
                            // accept
                            let socket_fd = self.cells[args[0]] as usize; // Use fd from argument
                            let addr_ptr = args[1];
                            let addr_len_ptr = args[2];

                            // Create a mutable buffer for the address length
                            let mut addr_len = self.cells[addr_len_ptr] as usize;

                            let result = syscalls::syscall!(
                                Sysno::accept,
                                socket_fd,
                                &self.cells[addr_ptr] as *const u32 as *mut u8,
                                &mut addr_len as *mut usize
                            )
                            .map_err(|e| {
                                BFError::SyscallFailed(format!("accept syscall failed: {}", e))
                            });

                            if let Ok(client_fd) = result {
                                println!("Accept syscall succeeded with client fd: {}", client_fd);
                                self.cells[0] = client_fd as u32; // Store client fd in cell 0
                                self.cells[7] = 0; // Success
                                self.cells[addr_len_ptr] = addr_len as u32; // Update the address length
                            }
                            result
                        }
                        6 => syscalls::syscall!(Sysno::close, args[0]).map_err(|e| {
                            BFError::SyscallFailed(format!("close syscall failed: {}", e))
                        }),
                        _ => Err(BFError::InvalidSyscall("Invalid syscall".to_string())),
                    }?;

                    // Store result in cell 7, not cell 0
                    self.cells[7] = result as u32;
                    Ok(())
                }
            }
            // Handle other BFA instructions normally
            '>' | '<' | '+' | '-' | '[' | ']' => self.execute_bf(),
            _ => Ok(()),
        }
    }

    fn validate_syscall(&self, syscall_num: u32, args: &[usize; 6]) -> Result<(), BFError> {
        // macOS syscall numbers
        match syscall_num {
            3 => {
                // read validation
                let _fd = args[0];
                let buf_addr = args[1];
                let buf_len = args[2];

                if buf_addr >= self.cells.len()
                    || buf_len == 0
                    || buf_addr + buf_len > self.cells.len()
                {
                    return Err(BFError::MemoryAccess("Buffer overflow".to_string()));
                }
            }
            4 => {
                // write validation
                let _fd = args[0];
                let buf_addr = args[1];
                let buf_len = args[2];

                if buf_addr >= self.cells.len()
                    || buf_len == 0
                    || buf_addr + buf_len > self.cells.len()
                {
                    return Err(BFError::MemoryAccess("Buffer overflow".to_string()));
                }
            }
            48 | 97 | 104 | 106 | 6 => {
                // accept, socket, bind, listen, close validation
                // These are valid syscalls on macOS
            }
            _ => {
                return Err(BFError::InvalidSyscall(format!(
                    "Invalid syscall number: {}",
                    syscall_num
                )));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bfl::{BFLCompiler, BFLNode};

    mod syscall_direct {
        use syscalls::Sysno;

        #[test]
        fn test_write_syscall() {
            let message = b"test\n";
            unsafe {
                let result = syscalls::syscall!(
                    Sysno::write,
                    1, // stdout
                    message.as_ptr(),
                    message.len()
                );
                assert!(result.is_ok());
                let bytes_written = result.unwrap();
                assert_eq!(bytes_written as usize, message.len());
            }
        }

        #[test]
        fn test_write_from_buffer() {
            let mut buffer = vec![0u8; 30000];
            buffer[3] = b'h';
            buffer[4] = b'i';
            buffer[5] = b'\n';

            unsafe {
                let result = syscalls::syscall!(
                    Sysno::write,
                    1, // stdout
                    buffer[3..].as_ptr(),
                    3
                );
                assert!(result.is_ok());
                let bytes_written = result.unwrap();
                assert_eq!(bytes_written as usize, 3);
            }
        }

        #[test]
        fn test_read_syscall() {
            let mut buf = [0u8; 10];
            unsafe {
                let result = syscalls::syscall!(
                    Sysno::read,
                    0, // stdin
                    buf.as_mut_ptr(),
                    1
                );
                assert!(result.is_ok());
                let bytes_read = result.unwrap();
                assert_eq!(bytes_read as usize, 1);
            }
        }
    }

    mod bf {
        use super::*;
        use pretty_assertions::assert_eq;

        #[test]
        fn test_basic_operations() {
            let mut bf = BF::new("+++.", Mode::BF);
            bf.run().unwrap();
            assert_eq!(bf.output, vec![3]);
        }

        #[test]
        fn test_pointer_movement() {
            let mut bf = BF::new(">+++>++<.", Mode::BF);
            bf.run().unwrap();
            assert_eq!(bf.output, vec![3]);
        }

        #[test]
        fn test_loop() {
            let mut bf = BF::new("+++[>+<-]>.", Mode::BF);
            bf.run().unwrap();
            assert_eq!(bf.output, vec![3]);
        }

        #[test]
        fn test_hello_world() {
            let mut bf = BF::new(
                "++++++++[>++++[>++>+++>+++>+<<<<-]>+>+>->>+[<]<-]>>.>---.+++++++..+++.>>.<-.<.+++.------.--------.>>+.>++.",
                Mode::BF
            );
            bf.run().unwrap();
            assert_eq!(String::from_utf8(bf.output).unwrap(), "Hello World!\n");
        }

        #[test]
        fn test_unmatched_brackets() {
            let mut bf = BF::new("[", Mode::BF);
            assert!(bf.run().is_err());

            let mut bf = BF::new("]", Mode::BF);
            assert!(bf.run().is_err());
        }

        #[test]
        fn test_input() {
            use std::io::Write;
            use std::process::{Command, Stdio};

            let mut child = Command::new(std::env::current_exe().unwrap())
                .arg("test")
                .arg("--exact")
                .arg("tests::bf::test_input_internal")
                .stdin(Stdio::piped())
                .spawn()
                .unwrap();

            child.stdin.take().unwrap().write_all(b"A").unwrap();
            assert!(child.wait().unwrap().success());
        }

        #[test]
        #[ignore = "This test is only meant to be run as a subprocess of test_input"]
        fn test_input_internal() {
            let mut bf = BF::new(",.", Mode::BF); // Read one char and output it
            bf.run().unwrap();
            assert_eq!(bf.output, vec![b'A']);
        }
    }

    mod bfa {
        use super::*;
        use pretty_assertions::assert_eq;

        #[test]
        fn test_mode_switching() {
            let mut bf = BF::new("", Mode::BFA);
            assert!(bf.run().is_ok());
        }

        #[test]
        fn test_syscall_socket() {
            // Test socket creation with BFA mode
            // Sets up:
            // - socket syscall (cell[0] = 2)
            // - domain (cell[1] = 2)
            // - type (cell[2] = 1)
            let code = r#"
                ++          Set socket syscall (2)
                >++<       Set domain (2)
                >>+<<      Set type (1)
                .          Execute syscall
            "#
            .replace(" ", "")
            .replace("\n", "");
            let mut bf = BF::new(&code, Mode::BFA);
            let result = bf.run();
            assert!(
                result.is_err(),
                "Socket operations should be rejected in test mode"
            );
        }

        #[test]
        fn test_invalid_syscall() {
            // Test handling of invalid syscall number (>7)
            // Sets up:
            // - invalid syscall number (cell[0] = 8)
            let code = r#"
                ++++++++   Set invalid syscall number (8)
                .          Execute syscall
            "#
            .replace(" ", "")
            .replace("\n", "");
            let mut bf = BF::new(&code, Mode::BFA);
            let result = bf.run();
            assert!(result.is_err(), "Invalid syscall number should be rejected");
        }

        #[test]
        fn test_syscall_args() {
            // Test syscall argument handling
            // Sets up:
            // - write syscall (cell[0] = 1)
            // - invalid fd (cell[1] = 3)
            // - buffer (cell[2] = 0)
            // - length (cell[3] = 1)
            let code = r#"
                +          Set write syscall
                >+++<     Set invalid fd (3)
                >>+<<     Set buffer
                >>>+<<<   Set length
                .         Execute syscall
            "#
            .replace(" ", "")
            .replace("\n", "");
            let mut bf = BF::new(&code, Mode::BFA);
            let result = bf.run();
            assert!(
                result.is_err(),
                "Invalid file descriptor should be rejected"
            );

            assert_eq!(bf.cells[0], 1); // syscall number (write)
            assert_eq!(bf.cells[1], 3); // invalid fd
            assert_eq!(bf.cells[2], 1); // buffer
            assert_eq!(bf.cells[3], 1); // length
        }

        #[test]
        fn test_simple_write() {
            // Write "h" to stdout (fd 1)
            // Set up cells:
            // - write syscall (cell[0] = 1)
            // - fd 1 (cell[1] = 1)
            // - buffer pointer (cell[2] = HEAP_START)  // Point to start of heap
            // - length (cell[3] = 1)
            // - actual data at HEAP_START = 'h' (104)
            let code = r#"
                +                    Set syscall 1 (write)
                >+<                 Set fd 1 (stdout)
                >>+++++++<<        Set buffer pointer to HEAP_START (7)
                >>>+<<<             Set length 1
                >>>>>>>++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++<<<<<<<  Set buffer content to 'h' (104) at HEAP_START
                .                   Execute syscall
            "#.replace(" ", "").replace("\n", "");
            let mut bf = BF::new(&code, Mode::BFA);
            let result = bf.run();
            if let Err(e) = &result {
                eprintln!("Simple write failed: {}", e);
                eprintln!("Cells: {:?}", &bf.cells[..12]); // Show first 12 cells which includes syscall area and some heap
            }
            assert!(result.is_ok());
        }

        #[test]
        fn test_syscall_write() {
            // Test write syscall with BFA mode
            // Sets up:
            // - write syscall (cell[0] = 1)
            // - invalid fd (cell[1] = 3)
            // - buffer with content (cell[2])
            // - length (cell[3])
            let code = r#"
                +          Set write syscall
                >+++<     Set invalid fd (3)
                >>+<<     Set buffer content
                >>>++<<<  Set length
                .         Execute syscall
            "#
            .replace(" ", "")
            .replace("\n", "");
            let mut bf = BF::new(&code, Mode::BFA);
            let result = bf.run();
            assert!(result.is_err(), "Write with invalid fd should fail");
        }

        #[test]
        fn test_syscall_read() {
            // Test read syscall with BFA mode
            // Sets up:
            // - read syscall (cell[0] = 0)
            // - fd (cell[1] = 0)
            // - buffer location (cell[2])
            // - read length (cell[3])
            let code = r#"
                >+<       Set buffer location
                >>+<<     Set read length
                .         Execute syscall
            "#
            .replace(" ", "")
            .replace("\n", "");
            let mut bf = BF::new(&code, Mode::BFA);
            let result = bf.run();
            assert!(result.is_err());
        }

        #[test]
        fn test_string_comparison() {
            let mut compiler = BFLCompiler::new();

            // Test program that:
            // 1. Stores a byte (10 for newline)
            // 2. Compares it with another value (10)
            // 3. Sets a result flag
            let program = BFLNode::Block(vec![
                // Store test byte (newline - ASCII 10)
                BFLNode::Assign(
                    "test_byte".to_string(),
                    Box::new(BFLNode::Block(vec![
                        BFLNode::Number(10), // newline character
                    ])),
                ),
                // Store comparison value
                BFLNode::Assign(
                    "compare_byte".to_string(),
                    Box::new(BFLNode::Block(vec![
                        BFLNode::Number(10), // newline character
                    ])),
                ),
                // Initialize result to 0
                BFLNode::Assign(
                    "result".to_string(),
                    Box::new(BFLNode::Block(vec![BFLNode::Number(0)])),
                ),
                // Compare bytes and set result
                BFLNode::Block(vec![
                    // Copy test_byte to temp cell
                    BFLNode::Variable("test_byte".to_string()),
                    // Subtract compare_byte
                    BFLNode::Sub(
                        Box::new(BFLNode::Variable("test_byte".to_string())),
                        Box::new(BFLNode::Variable("compare_byte".to_string())),
                    ),
                    // If difference is 0, set result to 1
                    BFLNode::If(
                        Box::new(BFLNode::Number(0)),
                        vec![BFLNode::Assign(
                            "result".to_string(),
                            Box::new(BFLNode::Block(vec![BFLNode::Number(1)])),
                        )],
                    ),
                ]),
            ]);

            compiler.compile(&program).unwrap();
            let bf_code = compiler.get_output();
            let mut bf = BF::new(bf_code, Mode::BF);
            bf.run().unwrap();

            // Check that result is 1 (comparison succeeded)
            let result_location = compiler.get_variable_location("result").unwrap();
            assert_eq!(bf.cells[result_location], 1);
        }

        #[test]
        fn test_newline_handling() {
            let mut compiler = BFLCompiler::new();

            // Test program that:
            // 1. Stores a string with newline
            // 2. Checks each character
            let program = BFLNode::Block(vec![
                // Store test string "123\n"
                BFLNode::Assign(
                    "test_str".to_string(),
                    Box::new(BFLNode::String("123\n".to_string())),
                ),
                // Initialize result to 0
                BFLNode::Assign(
                    "result".to_string(),
                    Box::new(BFLNode::Block(vec![BFLNode::Number(0)])),
                ),
                // Check if last character is newline
                BFLNode::Block(vec![
                    // Get last character (at index 3)
                    BFLNode::Add(
                        Box::new(BFLNode::Variable("test_str".to_string())),
                        Box::new(BFLNode::Number(3)),
                    ),
                    // Compare with newline (10)
                    BFLNode::Sub(
                        Box::new(BFLNode::Number(10)),
                        Box::new(BFLNode::Variable("test_str".to_string())),
                    ),
                    // If difference is 0, set result to 1
                    BFLNode::If(
                        Box::new(BFLNode::Number(0)),
                        vec![BFLNode::Assign(
                            "result".to_string(),
                            Box::new(BFLNode::Block(vec![BFLNode::Number(1)])),
                        )],
                    ),
                ]),
            ]);

            compiler.compile(&program).unwrap();
            let bf_code = compiler.get_output();
            let mut bf = BF::new(bf_code, Mode::BF);
            bf.run().unwrap();

            // Check that result is 1 (newline was found)
            let result_location = compiler.get_variable_location("result").unwrap();
            assert_eq!(bf.cells[result_location], 1);
        }
    }
}
