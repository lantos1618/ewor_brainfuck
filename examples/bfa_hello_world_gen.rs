use ewor_brainfuck::bfl::{BFLCompiler, BFLNode};

fn main() {
    let mut compiler = BFLCompiler::new();
    
    // Create "Hello, World!\n" using BFL
    let program = BFLNode::Block(vec![
        // Assign string to variable
        BFLNode::Assign("msg".to_string(), Box::new(BFLNode::Bytes(b"Hello, World!\n".to_vec()))),
        
        // Print the string
        BFLNode::Syscall(
            Box::new(BFLNode::Number(1)), // write syscall
            vec![
                BFLNode::Number(1), // stdout
                BFLNode::Variable("msg".to_string()),
                BFLNode::Number(14), // length
            ],
        ),
    ]);
    
    println!("Compiling BFA hello world...");
    compiler.compile(&program).unwrap();
    let bf_code = compiler.get_output();
    println!("Generated {} characters of brainfuck code", bf_code.len());
    println!("BFA code: {}", bf_code);
    
    // Write to file
    std::fs::write("bfa_hello_world.bf", bf_code).unwrap();
    println!("Written to bfa_hello_world.bf");
} 