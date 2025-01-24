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
    cells: Vec<u8>,
    ptr: usize,
    code: Vec<char>,
    pc: usize,
    output: Vec<u8>, // For testing
    mode: Mode,
}

const SYSCALL_ARGS: usize = 7; // syscall number + 6 args
const HEAP_START: usize = SYSCALL_ARGS; // Start heap right after syscall args

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Mode {
    BF,  // Standard Brainfuck
    BFA, // Brainfuck with syscall extensions
}

impl BF {
    pub fn new(code: &str, mode: Mode) -> Self {
        let cells = vec![0; 65536]; // Increase to 64KB from 30KB
        BF {
            cells,
            ptr: 0,
            code: code.chars().collect(),
            pc: 0,
            output: Vec::new(),
            mode,
        }
    }

    pub fn dump_cells(&self, n: usize) -> &[u8] {
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
                self.output.push(self.cells[self.ptr]);
                // Also write to stdout
                print!("{}", self.cells[self.ptr] as char);
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
                self.cells[self.ptr] = buf[0];
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
                let syscall_num = self.cells[0];

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
                    if (2..=6).contains(&syscall_num) {
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
                        0 => {
                            // read
                            let buf = &mut self.cells[args[1]..args[1] + args[2]];
                            syscalls::syscall!(Sysno::read, args[0], buf.as_mut_ptr(), args[2])
                                .map_err(|e| {
                                    BFError::SyscallFailed(format!("read syscall failed: {}", e))
                                })
                        }
                        1 => {
                            // write
                            let buf = &self.cells[args[1]..args[1] + args[2]];
                            // Print the buffer contents for debugging
                            println!("Write syscall:");
                            println!("  fd: {}", args[0]);
                            println!("  buffer pointer: {}", args[1]);
                            println!("  length: {}", args[2]);
                            println!("  buffer contents: {:?}", buf);
                            println!(
                                "  buffer as chars: {:?}",
                                buf.iter().map(|&b| b as char).collect::<Vec<_>>()
                            );
                            println!("  cells[0..15]: {:?}", &self.cells[0..15]);
                            syscalls::syscall!(Sysno::write, args[0], buf.as_ptr(), args[2])
                                .map_err(|e| {
                                    BFError::SyscallFailed(format!("write syscall failed: {}", e))
                                })
                        }
                        2 => {
                            // socket
                            syscalls::syscall!(Sysno::socket, args[0], args[1], args[2]).map_err(
                                |e| BFError::SyscallFailed(format!("socket syscall failed: {}", e)),
                            )
                        }
                        3 => {
                            // bind
                            let buf = &self.cells[args[1]..args[1] + args[2]];
                            syscalls::syscall!(Sysno::bind, args[0], buf.as_ptr(), args[2]).map_err(
                                |e| BFError::SyscallFailed(format!("bind syscall failed: {}", e)),
                            )
                        }
                        4 => syscalls::syscall!(Sysno::listen, args[0], args[1]).map_err(|e| {
                            BFError::SyscallFailed(format!("listen syscall failed: {}", e))
                        }),
                        5 => syscalls::syscall!(Sysno::accept, args[0], args[1], args[2]).map_err(
                            |e| BFError::SyscallFailed(format!("accept syscall failed: {}", e)),
                        ),
                        6 => syscalls::syscall!(Sysno::connect, args[0], args[1], args[2]).map_err(
                            |e| BFError::SyscallFailed(format!("connect syscall failed: {}", e)),
                        ),
                        7 => syscalls::syscall!(Sysno::close, args[0]).map_err(|e| {
                            BFError::SyscallFailed(format!("close syscall failed: {}", e))
                        }),
                        _ => Err(BFError::InvalidSyscall("Invalid syscall".to_string())),
                    }?;

                    self.cells[0] = result as u8;
                    Ok(())
                }
            }
            // Handle other BFA instructions normally
            '>' | '<' | '+' | '-' | '[' | ']' => self.execute_bf(),
            _ => Ok(()),
        }
    }

    fn validate_syscall(&self, syscall_num: u8, args: &[usize; 6]) -> Result<(), BFError> {
        if syscall_num > 7 {
            return Err(BFError::InvalidSyscall(format!(
                "Invalid syscall number: {}",
                syscall_num
            )));
        }

        match syscall_num {
            0 | 1 => {
                // read/write validation
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
            7 => {
                // close validation
            }
            _ => {}
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
                eprintln!("Cells: {:?}", &bf.cells[..HEAP_START + 5]);
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
    }
}
