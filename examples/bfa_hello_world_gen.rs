use ewor_brainfuck::bfl::{BFLCompiler, BFLNode};

fn main() {
    let mut compiler = BFLCompiler::new();
    
    // Create "Hello World!" using BFL with proper variable storage and syscalls
    let hello_world = "Hello World!\n";
    let mut statements = Vec::new();
    
    for (i, &byte) in hello_world.as_bytes().iter().enumerate() {
        let var_name = format!("char_{}", i);
        
        // Store the character in a variable
        statements.push(BFLNode::Assign(
            var_name.clone(),
            Box::new(BFLNode::Number(byte as i32)),
        ));
        
        // Create a syscall to print the character
        statements.push(BFLNode::Syscall(
            Box::new(BFLNode::Number(1)), // write syscall
            vec![
                BFLNode::Number(1), // stdout
                BFLNode::Variable(var_name), // reference to the character variable
                BFLNode::Number(1), // length
            ],
        ));
    }
    
    let program = BFLNode::Block(statements);
    
    println!("Testing BFL compiler with Hello World using variables and syscalls...");
    compiler.compile(&program).unwrap();
    let bf_code = compiler.get_output();
    println!("Generated {} characters of brainfuck code", bf_code.len());
    println!("BFA code: {}", bf_code);
    
    // Write to file
    std::fs::write("bfa_hello_world.bf", &bf_code).unwrap();
    println!("Written to bfa_hello_world.bf");
    
    // Test the generated code with BFA mode
    println!("\nTesting the generated brainfuck code with BFA mode:");
    let mut bf = ewor_brainfuck::bf::BF::new(&bf_code, ewor_brainfuck::bf::Mode::BFA);
    
    if let Err(e) = bf.run() {
        eprintln!("Brainfuck program failed: {}", e);
    } else {
        println!("\nBFL compiler Hello World test completed successfully!");
        println!("The BFL compiler successfully generated brainfuck code that prints 'Hello World!'");
    }
} 