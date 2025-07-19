use ewor_brainfuck::bfl::{BFLCompiler, BFLNode};
use ewor_brainfuck::bf::{BF, Mode};

fn main() {
    let mut compiler = BFLCompiler::new();
    
    // Create socket (AF_INET=2, SOCK_STREAM=1, 0)
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
        
        // Create sockaddr_in structure: [AF_INET=2, port=8080, addr=127.0.0.1, padding]
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
        BFLNode::Assign("status_msg".to_string(), Box::new(BFLNode::Bytes(b"Server listening on port 8080\n".to_vec()))),
        
        // Print status
        BFLNode::Syscall(
            Box::new(BFLNode::Number(1)), // write syscall
            vec![
                BFLNode::Number(1), // stdout
                BFLNode::Variable("status_msg".to_string()),
                BFLNode::Number(28), // length
            ],
        ),
        
        // Accept connection
        BFLNode::Syscall(
            Box::new(BFLNode::Number(43)), // accept syscall
            vec![
                BFLNode::Variable("fd".to_string()),
                BFLNode::Number(0), // addr (NULL)
                BFLNode::Number(0), // addrlen (NULL)
            ],
        ),
        
        // Store client fd
        BFLNode::Assign("client_fd".to_string(), Box::new(BFLNode::Variable("_syscall_result".to_string()))),
        
        // Create read buffer
        BFLNode::Assign("buf".to_string(), Box::new(BFLNode::Bytes(vec![0; 16]))),
        
        // Read data from client
        BFLNode::Syscall(
            Box::new(BFLNode::Number(0)), // read syscall
            vec![
                BFLNode::Variable("client_fd".to_string()),
                BFLNode::Variable("buf".to_string()),
                BFLNode::Number(16), // max bytes to read
            ],
        ),
        
        // Store bytes read
        BFLNode::Assign("bytes_read".to_string(), Box::new(BFLNode::Variable("_syscall_result".to_string()))),
        
        // Echo back the same data
        BFLNode::Syscall(
            Box::new(BFLNode::Number(1)), // write syscall
            vec![
                BFLNode::Variable("client_fd".to_string()),
                BFLNode::Variable("buf".to_string()),
                BFLNode::Variable("bytes_read".to_string()),
            ],
        ),
        
        // Close client connection
        BFLNode::Syscall(
            Box::new(BFLNode::Number(3)), // close syscall
            vec![
                BFLNode::Variable("client_fd".to_string()),
            ],
        ),
        
        // Close server socket
        BFLNode::Syscall(
            Box::new(BFLNode::Number(3)), // close syscall
            vec![
                BFLNode::Variable("fd".to_string()),
            ],
        ),
    ]);
    
    println!("Compiling ping-pong server...");
    compiler.compile(&program).unwrap();
    let bf_code = compiler.get_output();
    println!("Generated {} characters of brainfuck code", bf_code.len());
    
    println!("Running ping-pong server...");
    let mut bf = BF::new(bf_code, Mode::BFA);
    match bf.run() {
        Ok(_) => println!("Server completed successfully"),
        Err(e) => println!("Server error: {}", e),
    }
} 