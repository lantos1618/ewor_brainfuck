use ewor_brainfuck::bfl::{BFLCompiler, BFLNode};
use ewor_brainfuck::bf::{BF, Mode};

fn main() {
    let mut compiler = BFLCompiler::new();
    let program = BFLNode::Block(vec![
        BFLNode::Assign("msg".to_string(), Box::new(BFLNode::String("Hello, syscall!\n".to_string()))),
        BFLNode::Syscall(
            Box::new(BFLNode::Number(1)), // SYS_WRITE (Linux)
            vec![
                BFLNode::Number(1), // stdout
                BFLNode::Variable("msg".to_string()), // pointer to string
                BFLNode::Number(16), // length
            ],
        ),
    ]);
    
    compiler.compile(&program).unwrap();
    let bf_code = compiler.get_output();
    println!("[DEBUG] About to print generated BF code");
    println!("Generated BF code length: {}", bf_code.len());
    println!("First 100 chars: {}", &bf_code.chars().take(100).collect::<String>());
    println!("Generated BF code:\n{}", bf_code);
    println!("[DEBUG] Finished printing generated BF code");
    let mut bf = BF::new(bf_code, Mode::BFA);
    println!("[DEBUG] Memory before run: {:?}", bf.dump_cells(32));
    println!("Output:");
    bf.run().unwrap();
    println!("[DEBUG] Memory after run: {:?}", bf.dump_cells(32));
} 