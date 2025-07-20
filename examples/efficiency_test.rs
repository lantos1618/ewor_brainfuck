use ewor_brainfuck::bfl::{BFLCompiler, BFLNode};

fn main() {
    let mut compiler = BFLCompiler::new();
    
    // Test 1: Simple assignment
    println!("=== Test 1: Simple Assignment ===");
    let simple_program = BFLNode::Block(vec![
        BFLNode::Assign("x".to_string(), Box::new(BFLNode::Number(42))),
    ]);
    
    compiler.compile(&simple_program).unwrap();
    let code = compiler.get_output();
    println!("Simple assignment code length: {}", code.len());
    println!("First 200 chars: {}", &code[..code.len().min(200)]);
    
    // Test 2: Addition
    println!("\n=== Test 2: Addition ===");
    let mut compiler2 = BFLCompiler::new();
    let add_program = BFLNode::Block(vec![
        BFLNode::Assign("result".to_string(), Box::new(BFLNode::Add(
            Box::new(BFLNode::Number(10)),
            Box::new(BFLNode::Number(20)),
        ))),
    ]);
    
    compiler2.compile(&add_program).unwrap();
    let code2 = compiler2.get_output();
    println!("Addition code length: {}", code2.len());
    println!("First 200 chars: {}", &code2[..code2.len().min(200)]);
    
    // Test 3: String assignment
    println!("\n=== Test 3: String Assignment ===");
    let mut compiler3 = BFLCompiler::new();
    let string_program = BFLNode::Block(vec![
        BFLNode::Assign("msg".to_string(), Box::new(BFLNode::String("Hello".to_string()))),
    ]);
    
    compiler3.compile(&string_program).unwrap();
    let code3 = compiler3.get_output();
    println!("String assignment code length: {}", code3.len());
    println!("First 200 chars: {}", &code3[..code3.len().min(200)]);
    
    // Test 4: If statement
    println!("\n=== Test 4: If Statement ===");
    let mut compiler4 = BFLCompiler::new();
    let if_program = BFLNode::Block(vec![
        BFLNode::If(
            Box::new(BFLNode::Number(1)),
            vec![
                BFLNode::Assign("x".to_string(), Box::new(BFLNode::Number(99))),
            ],
        ),
    ]);
    
    compiler4.compile(&if_program).unwrap();
    let code4 = compiler4.get_output();
    println!("If statement code length: {}", code4.len());
    println!("First 200 chars: {}", &code4[..code4.len().min(200)]);
} 