use ewor_brainfuck::bfl::{BFLCompiler, BFLNode};
use ewor_brainfuck::bf::{BF, Mode};

fn main() {
    let mut compiler = BFLCompiler::new();
    
    // Simple test: just create a socket and close it
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
        
        // Create message buffer
        BFLNode::Assign("msg".to_string(), Box::new(BFLNode::Bytes(b"Socket test completed\n".to_vec()))),
        
        // Print success message
        BFLNode::Syscall(
            Box::new(BFLNode::Number(1)), // write syscall
            vec![
                BFLNode::Number(1), // stdout
                BFLNode::Variable("msg".to_string()),
                BFLNode::Number(22), // length
            ],
        ),
        
        // Close socket
        BFLNode::Syscall(
            Box::new(BFLNode::Number(3)), // close syscall
            vec![
                BFLNode::Variable("fd".to_string()),
            ],
        ),
    ]);
    
    println!("Compiling simple socket test...");
    compiler.compile(&program).unwrap();
    let bf_code = compiler.get_output();
    println!("Generated {} characters of brainfuck code", bf_code.len());
    
    println!("Running socket test...");
    let mut bf = BF::new(bf_code, Mode::BFA);
    match bf.run() {
        Ok(_) => println!("Test completed successfully"),
        Err(e) => println!("Test error: {}", e),
    }
} 