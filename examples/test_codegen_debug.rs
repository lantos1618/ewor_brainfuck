use ewor_brainfuck::bfl::{BFLCompiler, BFLNode};

fn main() {
    // Test 1: Just assign a number
    println!("=== Test 1: Number assignment ===");
    let mut compiler1 = BFLCompiler::new();
    let program1 = BFLNode::Block(vec![
        BFLNode::Assign("x".to_string(), Box::new(BFLNode::Number(42))),
    ]);
    compiler1.compile(&program1).unwrap();
    let code1 = compiler1.get_output();
    let optimized1 = compiler1.get_optimized_output_copy();
    println!("Number assignment: {} chars (optimized: {} chars)", code1.len(), optimized1.len());
    println!("Code: {}", &code1[..code1.len().min(200)]);
    
    // Test 2: Assign a string
    println!("\n=== Test 2: String assignment ===");
    let mut compiler2 = BFLCompiler::new();
    let program2 = BFLNode::Block(vec![
        BFLNode::Assign("msg".to_string(), Box::new(BFLNode::String("Hi".to_string()))),
    ]);
    compiler2.compile(&program2).unwrap();
    let code2 = compiler2.get_output();
    let optimized2 = compiler2.get_optimized_output_copy();
    println!("String assignment: {} chars (optimized: {} chars)", code2.len(), optimized2.len());
    println!("Code: {}", &code2[..code2.len().min(200)]);
    
    // Test 3: Simple loop with number
    println!("\n=== Test 3: Simple loop with number ===");
    let mut compiler3 = BFLCompiler::new();
    let program3 = BFLNode::Block(vec![
        BFLNode::Assign("i".to_string(), Box::new(BFLNode::Number(3))),
        BFLNode::While(
            Box::new(BFLNode::Variable("i".to_string())),
            vec![
                BFLNode::Assign(
                    "i".to_string(),
                    Box::new(BFLNode::Sub(
                        Box::new(BFLNode::Variable("i".to_string())),
                        Box::new(BFLNode::Number(1)),
                    )),
                ),
            ],
        ),
    ]);
    compiler3.compile(&program3).unwrap();
    let code3 = compiler3.get_output();
    let optimized3 = compiler3.get_optimized_output_copy();
    println!("Simple loop: {} chars (optimized: {} chars)", code3.len(), optimized3.len());
    println!("Code: {}", &code3[..code3.len().min(200)]);
    
    // Test 4: Loop with string assignment inside
    println!("\n=== Test 4: Loop with string inside ===");
    let mut compiler4 = BFLCompiler::new();
    let program4 = BFLNode::Block(vec![
        BFLNode::Assign("i".to_string(), Box::new(BFLNode::Number(3))),
        BFLNode::While(
            Box::new(BFLNode::Variable("i".to_string())),
            vec![
                BFLNode::Assign("msg".to_string(), Box::new(BFLNode::String("Hi".to_string()))),
                BFLNode::Assign(
                    "i".to_string(),
                    Box::new(BFLNode::Sub(
                        Box::new(BFLNode::Variable("i".to_string())),
                        Box::new(BFLNode::Number(1)),
                    )),
                ),
            ],
        ),
    ]);
    compiler4.compile(&program4).unwrap();
    let code4 = compiler4.get_output();
    let optimized4 = compiler4.get_optimized_output_copy();
    println!("Loop with string: {} chars (optimized: {} chars)", code4.len(), optimized4.len());
    println!("Code: {}", &code4[..code4.len().min(200)]);
} 