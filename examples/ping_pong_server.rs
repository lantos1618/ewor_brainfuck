use ewor_brainfuck::bfl::{BFLCompiler, BFLNode};
use ewor_brainfuck::bf::{BF, Mode};

fn main() {
    let mut compiler = BFLCompiler::new();

    let status_msg = b"Server listening on port 8080\n";
    let client_msg = b"Client connected on port 8080\n";

    let program = BFLNode::Block(vec![
        // Create socket
        BFLNode::Syscall(
            Box::new(BFLNode::Number(41)), // socket syscall
            vec![
                BFLNode::Number(2),  // AF_INET
                BFLNode::Number(1),  // SOCK_STREAM
                BFLNode::Number(0),  // protocol
            ],
        ),
        // Store socket fd
        BFLNode::Assign("fd".to_string(), Box::new(BFLNode::Variable("_syscall_result".to_string()))),
        // Also store server_fd for tracking (copy immediately to avoid corruption)
        BFLNode::Assign("server_fd".to_string(), Box::new(BFLNode::Variable("fd".to_string()))),
        // Create sockaddr_in structure
        BFLNode::Assign("addr".to_string(), Box::new(BFLNode::Bytes(vec![
            2, 0,     // AF_INET
            31, 144,  // port 8080 (network byte order)
            127, 0, 0, 1,  // 127.0.0.1
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0  // padding
        ]))),
        // Bind socket
        BFLNode::Syscall(
            Box::new(BFLNode::Number(49)), // bind syscall
            vec![
                BFLNode::Variable("fd".to_string()),
                BFLNode::Variable("addr".to_string()),
                BFLNode::Number(16), // addrlen
            ],
        ),
        // Listen for connections
        BFLNode::Syscall(
            Box::new(BFLNode::Number(50)), // listen syscall
            vec![
                BFLNode::Variable("fd".to_string()),
                BFLNode::Number(1), // backlog
            ],
        ),
        // Create status message
        BFLNode::Assign("status_msg".to_string(), Box::new(BFLNode::Bytes(status_msg.to_vec()))),
        // Print status
        BFLNode::Syscall(
            Box::new(BFLNode::Number(1)), // write syscall
            vec![
                BFLNode::Number(1), // stdout
                BFLNode::Variable("status_msg".to_string()),
                BFLNode::Number(status_msg.len() as i32), // length
            ],
        ),
        // Accept connection using server_fd (once, outside the data loop)
        BFLNode::Syscall(
            Box::new(BFLNode::Number(43)), // accept syscall
            vec![
                BFLNode::Variable("server_fd".to_string()),
                BFLNode::Number(0), // addr (NULL)
                BFLNode::Number(0), // addrlen (NULL)
            ],
        ),
        // Store client fd
        BFLNode::Assign("client_fd".to_string(), Box::new(BFLNode::Variable("_syscall_result".to_string()))),
        // Print client connected message
        BFLNode::Assign("client_msg".to_string(), Box::new(BFLNode::Bytes(client_msg.to_vec()))),
        BFLNode::Syscall(
            Box::new(BFLNode::Number(1)), // write syscall
            vec![
                BFLNode::Number(1), // stdout
                BFLNode::Variable("client_msg".to_string()),
                BFLNode::Number(client_msg.len() as i32), // length
            ],
        ),
        // Allocate buffer with proper size (1024 bytes)
        BFLNode::Assign("buf".to_string(), Box::new(BFLNode::Bytes(vec![0; 1024]))),
        // Main data loop: handle data from the same client
        BFLNode::While(
            Box::new(BFLNode::Number(1)), // infinite loop
            vec![
                // Print at start of loop
                BFLNode::Assign("loop_msg".to_string(), Box::new(BFLNode::Bytes(b"[LOOP] Entered loop\n".to_vec()))),
                BFLNode::Syscall(
                    Box::new(BFLNode::Number(1)),
                    vec![
                        BFLNode::Number(1),
                        BFLNode::Variable("loop_msg".to_string()),
                        BFLNode::Number(18),
                    ],
                ),
                // Read data from client
                BFLNode::Syscall(
                    Box::new(BFLNode::Number(0)), // read syscall
                    vec![
                        BFLNode::Variable("client_fd".to_string()),
                        BFLNode::Variable("buf".to_string()),
                        BFLNode::Number(1024), // max bytes to read
                    ],
                ),
                // Print after read syscall
                BFLNode::Assign("after_read_msg".to_string(), Box::new(BFLNode::Bytes(b"[LOOP] After read\n".to_vec()))),
                BFLNode::Syscall(
                    Box::new(BFLNode::Number(1)),
                    vec![
                        BFLNode::Number(1),
                        BFLNode::Variable("after_read_msg".to_string()),
                        BFLNode::Number(18),
                    ],
                ),
                // Store bytes read
                BFLNode::Assign("bytes_read".to_string(), Box::new(BFLNode::Variable("_syscall_result".to_string()))),
                // Check if bytes_read > 0 (client sent data) and echo back
                BFLNode::If(
                    Box::new(BFLNode::Variable("bytes_read".to_string())),
                    vec![
                        // Debug: Print a fixed message to show we received data
                        BFLNode::Assign("debug_msg".to_string(), Box::new(BFLNode::Bytes(b"Received data: ".to_vec()))),
                        BFLNode::Syscall(
                            Box::new(BFLNode::Number(1)), // write syscall
                            vec![
                                BFLNode::Number(1), // stdout
                                BFLNode::Variable("debug_msg".to_string()),
                                BFLNode::Number(14), // length of "Received data: "
                            ],
                        ),
                        // Debug: Print what we received to stdout
                        BFLNode::Syscall(
                            Box::new(BFLNode::Number(1)), // write syscall
                            vec![
                                BFLNode::Number(1), // stdout
                                BFLNode::Variable("buf".to_string()),
                                BFLNode::Variable("bytes_read".to_string()),
                            ],
                        ),
                        // Debug: Print bytes_read value as a decimal string
                        BFLNode::Assign("bytes_read_msg".to_string(), Box::new(BFLNode::Bytes(b"bytes_read: ".to_vec()))),
                        BFLNode::Syscall(
                            Box::new(BFLNode::Number(1)), // write syscall
                            vec![
                                BFLNode::Number(1), // stdout
                                BFLNode::Variable("bytes_read_msg".to_string()),
                                BFLNode::Number(12), // length of "bytes_read: "
                            ],
                        ),
                        // Print the value of bytes_read (as a single byte, not a full decimal string)
                        BFLNode::Syscall(
                            Box::new(BFLNode::Number(1)), // write syscall
                            vec![
                                BFLNode::Number(1), // stdout
                                BFLNode::Variable("bytes_read".to_string()),
                                BFLNode::Number(1), // just print the raw byte value
                            ],
                        ),
                        // Print newline
                        BFLNode::Assign("newline2".to_string(), Box::new(BFLNode::Bytes(b"\n".to_vec()))),
                        BFLNode::Syscall(
                            Box::new(BFLNode::Number(1)), // write syscall
                            vec![
                                BFLNode::Number(1), // stdout
                                BFLNode::Variable("newline2".to_string()),
                                BFLNode::Number(1), // length
                            ],
                        ),
                        // Echo back the received data to client
                        BFLNode::Syscall(
                            Box::new(BFLNode::Number(1)), // write syscall
                            vec![
                                BFLNode::Variable("client_fd".to_string()),
                                BFLNode::Variable("buf".to_string()),
                                BFLNode::Variable("bytes_read".to_string()),
                            ],
                        ),
                    ]
                ),
            ]
        ),
        // Close client connection
        BFLNode::Syscall(
            Box::new(BFLNode::Number(3)), // close syscall
            vec![
                BFLNode::Variable("client_fd".to_string()),
            ],
        ),
        // Close server socket (unreachable)
        BFLNode::Syscall(
            Box::new(BFLNode::Number(3)), // close syscall
            vec![
                BFLNode::Variable("server_fd".to_string()),
            ],
        ),
    ]);
    
    println!("Compiling ping-pong server...");
    println!("Program: {:?}", program);

    compiler.compile(&program).unwrap();
    let bf_code = compiler.get_output();
    // Print all variable addresses for debugging
    for var in ["fd", "server_fd", "addr", "status_msg", "client_fd", "client_msg", "buf", "bytes_read"].iter() {
        if let Some(addr) = compiler.get_variable_address(var) {
            println!("[DEBUG] Variable {} address: {}", var, addr);
        } else {
            println!("[DEBUG] Variable {} not found", var);
        }
    }
    println!("Generated {} characters of brainfuck code", bf_code.len());
    println!("Generated BF code:\n{}", bf_code);
    println!("Running ping-pong server...");
    let mut bf = BF::new(bf_code, Mode::BFA);
    match bf.run() {
        Ok(_) => println!("Server completed successfully"),
        Err(e) => println!("Server error: {}", e),
    }
} 