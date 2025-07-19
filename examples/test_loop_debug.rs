use ewor_brainfuck::bf::{BF, Mode};
use ewor_brainfuck::bfl::{BFLCompiler, BFLNode};

fn main() {
    let mut compiler = BFLCompiler::new();
    
    // Simple loop that counts from 3 down to 0
    let program = BFLNode::Block(vec![
        BFLNode::Assign("i".to_string(), Box::new(BFLNode::Number(3))),
        BFLNode::While(
            Box::new(BFLNode::Variable("i".to_string())),
            vec![
                // Just decrement i
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
    
    compiler.compile(&program).unwrap();
    let bf_code = compiler.get_output();
    
    println!("Generated BF code:");
    println!("{}", bf_code);
    println!("\nBF code length: {}", bf_code.len());
    
    println!("\nRunning BF code...");
    let mut bf = BF::new(&bf_code, Mode::BFA);
    match bf.run() {
        Ok(_) => println!("Program completed successfully"),
        Err(e) => println!("Error: {}", e),
    }
    
    // Check final value of i
    let i_addr = compiler.get_variable_address("i").unwrap();
    println!("\nFinal value of i at address {}: {}", i_addr, bf.dump_cells(i_addr + 1)[i_addr]);
} 