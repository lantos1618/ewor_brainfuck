use ewor_brainfuck::bf::{Mode, BF};
use ewor_brainfuck::bfl::{BFLCompiler, BFLNode};

fn main() -> Result<(), String> {
    let mut compiler = BFLCompiler::new();

    // lets make an echo server
    let program = BFLNode::Block(vec![
        // Create socket (syscall 97 -> 97)
        // AF_INET = 2, SOCK_STREAM = 1, protocol = 0
        BFLNode::Syscall(
            Box::new(BFLNode::Number(97)), // socket syscall
            vec![
                BFLNode::Number(2), // AF_INET
                BFLNode::Number(1), // SOCK_STREAM
                BFLNode::Number(0), // protocol
            ],
        ),
        // Store socket fd in a variable
        BFLNode::Assign(
            "socket_fd".to_string(),
            Box::new(BFLNode::Block(vec![
                BFLNode::Number(0), // Copy value from cell[0] where syscall result is stored
            ])),
        ),
        // Initialize sockaddr_in structure in memory
        BFLNode::Assign(
            "sockaddr".to_string(),
            Box::new(BFLNode::Block(vec![
                BFLNode::Number(2),    // sin_family: AF_INET = 2 (low byte)
                BFLNode::Number(0),    // sin_family: high byte = 0
                BFLNode::Number(0x1F), // sin_port: high byte (8080)
                BFLNode::Number(0x90), // sin_port: low byte (8080)
                BFLNode::Number(0),    // sin_addr[0] = 0.0.0.0
                BFLNode::Number(0),    // sin_addr[1] = 0.0.0.0
                BFLNode::Number(0),    // sin_addr[2] = 0.0.0.0
                BFLNode::Number(0),    // sin_addr[3] = 0.0.0.0
                BFLNode::Number(0),    // sin_zero[0]
                BFLNode::Number(0),    // sin_zero[1]
                BFLNode::Number(0),    // sin_zero[2]
                BFLNode::Number(0),    // sin_zero[3]
                BFLNode::Number(0),    // sin_zero[4]
                BFLNode::Number(0),    // sin_zero[5]
                BFLNode::Number(0),    // sin_zero[6]
                BFLNode::Number(0),    // sin_zero[7]
            ])),
        ),
        // Bind socket (syscall 104 -> 104)
        BFLNode::Syscall(
            Box::new(BFLNode::Number(104)), // bind syscall
            vec![
                BFLNode::Variable("socket_fd".to_string()),
                BFLNode::Variable("sockaddr".to_string()),
                BFLNode::Number(16), // size of sockaddr_in
            ],
        ),
        // Listen (syscall 106 -> 106)
        BFLNode::Syscall(
            Box::new(BFLNode::Number(106)), // listen syscall
            vec![
                BFLNode::Variable("socket_fd".to_string()),
                BFLNode::Number(2), // backlog of 2 connections
            ],
        ),
        // Print "Server started on port 8080" message
        BFLNode::Assign(
            "msg".to_string(),
            Box::new(BFLNode::String("Server started on port 8080\n".to_string())),
        ),
        BFLNode::Syscall(
            Box::new(BFLNode::Number(4)), // write syscall
            vec![
                BFLNode::Number(1), // stdout
                BFLNode::Variable("msg".to_string()),
                BFLNode::Number(28), // length of message (including newline)
            ],
        ),
        // Initialize client sockaddr structure for accept
        BFLNode::Assign(
            "client_addr".to_string(),
            Box::new(BFLNode::Block(vec![
                BFLNode::Number(0); 16 // Allocate space for sockaddr_in
            ])),
        ),
        // Initialize address length variable
        BFLNode::Assign(
            "addr_len".to_string(),
            Box::new(BFLNode::Block(vec![
                BFLNode::Number(16), // Size of sockaddr_in
            ])),
        ),
        // Initialize buffer for reading client data
        BFLNode::Assign(
            "buffer".to_string(),
            Box::new(BFLNode::Block(vec![BFLNode::Number(0); 64])), // 64 byte buffer
        ),
        // Main accept loop
        BFLNode::While(
            Box::new(BFLNode::Number(1)), // Infinite loop
            vec![
                // Print "Waiting for connection..." message
                BFLNode::Assign(
                    "msg".to_string(),
                    Box::new(BFLNode::String("Waiting for connection...\n".to_string())),
                ),
                BFLNode::Syscall(
                    Box::new(BFLNode::Number(4)), // write syscall
                    vec![
                        BFLNode::Number(1), // stdout
                        BFLNode::Variable("msg".to_string()),
                        BFLNode::Number(25), // length of message (including newline)
                    ],
                ),
                // Accept connection (syscall 48 -> accept)
                BFLNode::Syscall(
                    Box::new(BFLNode::Number(48)), // accept syscall
                    vec![
                        BFLNode::Variable("socket_fd".to_string()),
                        BFLNode::Variable("client_addr".to_string()),
                        BFLNode::Variable("addr_len".to_string()),
                    ],
                ),
                // Store client socket fd
                BFLNode::Assign(
                    "client_fd".to_string(),
                    Box::new(BFLNode::Block(vec![
                        BFLNode::Number(0), // Copy value from cell[0] where syscall result is stored
                    ])),
                ),
                // Print "Client connected" message
                BFLNode::Assign(
                    "msg".to_string(),
                    Box::new(BFLNode::String("Client connected\n".to_string())),
                ),
                BFLNode::Syscall(
                    Box::new(BFLNode::Number(4)), // write syscall
                    vec![
                        BFLNode::Number(1), // stdout
                        BFLNode::Variable("msg".to_string()),
                        BFLNode::Number(17), // length of message
                    ],
                ),
                // Read/echo loop for this client
                BFLNode::While(
                    Box::new(BFLNode::Number(1)), // Loop until client disconnects
                    vec![
                        // Read one byte
                        BFLNode::Syscall(
                            Box::new(BFLNode::Number(3)), // read syscall
                            vec![
                                BFLNode::Number(0), // Use client_fd directly from cell 0
                                BFLNode::Variable("buffer".to_string()),
                                BFLNode::Number(1), // read 1 byte
                            ],
                        ),
                        // Store bytes read
                        BFLNode::Assign(
                            "bytes_read".to_string(),
                            Box::new(BFLNode::Block(vec![
                                BFLNode::Number(0), // Copy value from cell[0] where syscall result is stored
                            ])),
                        ),
                        // Echo data back to client if we have any bytes
                        BFLNode::If(
                            Box::new(BFLNode::Variable("bytes_read".to_string())),
                            vec![
                                // Write the buffer
                                BFLNode::Syscall(
                                    Box::new(BFLNode::Number(4)), // write syscall
                                    vec![
                                        BFLNode::Number(0), // Use client_fd directly from cell 0
                                        BFLNode::Variable("buffer".to_string()),
                                        BFLNode::Number(1), // write 1 byte
                                    ],
                                ),
                                // Check if we got a newline
                                BFLNode::If(
                                    Box::new(BFLNode::Variable("buffer".to_string())), // Check the byte we read
                                    vec![
                                        // Break if newline (ASCII 10)
                                        BFLNode::If(
                                            Box::new(BFLNode::Number(10)), // Compare with newline
                                            vec![
                                                // Set bytes_read to 0 to break outer loop
                                                BFLNode::Assign(
                                                    "bytes_read".to_string(),
                                                    Box::new(BFLNode::Block(vec![
                                                        BFLNode::Number(0),
                                                    ])),
                                                ),
                                            ],
                                        ),
                                    ],
                                ),
                            ],
                        ),
                    ],
                ),
                // Close client socket (syscall 6 -> close)
                BFLNode::Syscall(
                    Box::new(BFLNode::Number(6)), // close syscall
                    vec![BFLNode::Variable("client_fd".to_string())],
                ),
            ],
        ),
    ]);

    compiler.compile(&program)?;
    let bf_code = compiler.get_output();
    println!("\nGenerated BF code:\n{}", bf_code);

    let mut bf = BF::new(bf_code, Mode::BFA);
    if let Err(e) = bf.run() {
        return Err(e.to_string());
    }

    Ok(())
}
